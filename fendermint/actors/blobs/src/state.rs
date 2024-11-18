// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Bound::{Included, Unbounded};

use fendermint_actor_blobs_shared::params::GetStatsReturn;
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, CreditApproval, Hash, PublicKey, Subscription,
};
use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::{BigInt, BigUint};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use log::{debug, warn};
use num_traits::{Signed, ToPrimitive, Zero};

/// The minimum epoch duration a blob can be stored.
const MIN_TTL: ChainEpoch = 3600; // one hour
/// The rolling epoch duration used for non-expiring blobs.
const AUTO_TTL: ChainEpoch = 3600; // one hour

/// The state represents all accounts and stored blobs.
/// TODO: use raw HAMTs
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total storage capacity of the subnet.
    pub capacity_total: BigInt,
    /// The total used storage capacity of the subnet.
    pub capacity_used: BigInt,
    /// The total number of credits sold in the subnet.
    pub credit_sold: BigInt,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: BigInt,
    /// The total number of credits debited in the subnet.
    pub credit_debited: BigInt,
    /// The byte-blocks per atto token rate set at genesis.
    pub credit_debit_rate: u64,
    /// Map containing all accounts by robust (non-ID) actor address.
    pub accounts: HashMap<Address, Account>,
    /// Map containing all blobs.
    pub blobs: HashMap<Hash, Blob>,
    /// Map of expiries to blob hashes.
    pub expiries: BTreeMap<ChainEpoch, HashMap<Address, HashMap<Hash, bool>>>,
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: BTreeMap<Hash, HashSet<(Address, PublicKey)>>,
    /// Map of blob hashes that are not yet added to the validator's resolve pool.
    pub added: BTreeMap<Hash, HashSet<(Address, PublicKey)>>,
}

/// Helper for handling credit approvals.
struct CreditDelegation<'a> {
    /// The address that is submitting the transaction to add this blob
    pub origin: Address,
    ///  address that is calling into the blob actor.  If the blobs actor is
    ///  accessed directly, this will be the same as "origin". But most of the time this will be
    ///  the address of the actor instance that is calling into the blobs actor. I.e a specific
    ///  Bucket or Timehub instance.
    pub caller: Address,
    /// Information about the approval that allows "origin" to use credits via "caller".
    /// Note that the Address that has issued this approval (the subscriber/sponsor), and whose
    /// credits are being allowed to be used, are not stored internal to this struct.
    pub approval: &'a mut CreditApproval,
}

impl<'a> CreditDelegation<'a> {
    pub fn new(origin: Address, caller: Address, approval: &'a mut CreditApproval) -> Self {
        Self {
            origin,
            caller,
            approval,
        }
    }

    /// Tuple of (Origin, Caller) addresses
    pub fn addresses(&self) -> (Address, Address) {
        (self.origin, self.caller)
    }
}

impl State {
    pub fn new(capacity: u64, credit_debit_rate: u64) -> Self {
        Self {
            capacity_total: BigInt::from(capacity),
            capacity_used: BigInt::zero(),
            credit_sold: BigInt::zero(),
            credit_committed: BigInt::zero(),
            credit_debited: BigInt::zero(),
            credit_debit_rate,
            accounts: HashMap::new(),
            blobs: HashMap::new(),
            expiries: BTreeMap::new(),
            pending: BTreeMap::new(),
            added: BTreeMap::new(),
        }
    }

    pub fn get_stats(&self, balance: TokenAmount) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(),
            capacity_used: self.capacity_used.clone(),
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            credit_debit_rate: self.credit_debit_rate,
            num_accounts: self.accounts.len() as u64,
            num_blobs: self.blobs.len() as u64,
            num_resolving: self.pending.len() as u64,
            bytes_resolving: self.pending.keys().map(|hash| self.blobs[hash].size).sum(),
            num_added: self.added.len() as u64,
            bytes_added: self.added.keys().map(|hash| self.blobs[hash].size).sum(),
        }
    }

    pub fn buy_credit(
        &mut self,
        recipient: Address,
        amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<Account, ActorError> {
        let credits = self.credit_debit_rate * amount.atto();
        // Don't sell credits if we're at storage capacity
        if self.capacity_available().is_zero() {
            return Err(ActorError::forbidden(
                "credits not available (subnet has reached storage capacity)".into(),
            ));
        }
        self.credit_sold += &credits;
        let account = self
            .accounts
            .entry(recipient)
            .and_modify(|a| a.credit_free += &credits)
            .or_insert(Account::new(credits.clone(), current_epoch));
        Ok(account.clone())
    }

    pub fn approve_credit(
        &mut self,
        from: Address,
        to: Address,
        require_caller: Option<Address>,
        current_epoch: ChainEpoch,
        limit: Option<BigUint>,
        ttl: Option<ChainEpoch>,
    ) -> anyhow::Result<CreditApproval, ActorError> {
        let limit = limit.map(BigInt::from);
        if let Some(ttl) = ttl {
            if ttl < MIN_TTL {
                return Err(ActorError::illegal_argument(format!(
                    "minimum approval TTL is {}",
                    MIN_TTL
                )));
            }
        }
        let expiry = ttl.map(|t| t + current_epoch);
        let account = self
            .accounts
            .entry(from)
            .or_insert(Account::new(BigInt::zero(), current_epoch));
        // Get or add a new approval
        let caller = require_caller.unwrap_or(to);
        let approval = account
            .approvals
            .entry(to)
            .or_default()
            .entry(caller)
            .or_insert(CreditApproval {
                limit: limit.clone(),
                committed: BigInt::zero(),
                expiry,
            });
        // Validate approval changes
        if let Some(limit) = limit.clone() {
            if approval.committed > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already spent credits ({})",
                    approval.committed
                )));
            }
        }
        approval.limit = limit;
        approval.expiry = expiry;
        Ok(approval.clone())
    }

    /// Returns the CreditApproval if one exists from the given address, to the given address,
    /// for the given caller, or None if no approval exists.
    pub fn get_credit_approval(
        &self,
        from: Address,
        receiver: Address,
        caller: Address,
    ) -> Option<CreditApproval> {
        let account = match self.accounts.get(&from) {
            None => return None,
            Some(account) => account,
        };
        // First look for an approval for "to" keyed by "to", which denotes it's valid for
        // any caller.
        // Second look for an approval for the supplied caller.
        let approval = if let Some(approvals) = account.approvals.get(&receiver) {
            if let Some(approval) = approvals.get(&receiver) {
                Some(approval)
            } else {
                approvals.get(&caller)
            }
        } else {
            None
        };
        approval.cloned()
    }

    pub fn revoke_credit(
        &mut self,
        from: Address,
        to: Address,
        require_caller: Option<Address>,
    ) -> anyhow::Result<(), ActorError> {
        let account = self
            .accounts
            .get_mut(&from)
            .ok_or(ActorError::not_found(format!("account {} not found", from)))?;
        let caller = require_caller.unwrap_or(to);
        if let Some(approvals) = account.approvals.get_mut(&to) {
            approvals.remove(&caller);
            if approvals.is_empty() {
                account.approvals.remove(&to);
            }
        }
        Ok(())
    }

    pub fn get_account(&self, address: Address) -> Option<Account> {
        self.accounts.get(&address).cloned()
    }

    #[allow(clippy::type_complexity)]
    pub fn debit_accounts(
        &mut self,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<HashSet<Hash>, ActorError> {
        // Delete expired subscriptions
        let mut delete_from_disc = HashSet::new();
        let expiries: Vec<(ChainEpoch, HashMap<Address, HashMap<Hash, bool>>)> = self
            .expiries
            .range((Unbounded, Included(current_epoch)))
            .map(|(expiry, entry)| (*expiry, entry.clone()))
            .collect();
        let mut num_renewed = 0;
        let mut num_deleted = 0;
        for (_, entry) in expiries {
            for (subscriber, subs) in entry {
                for (hash, auto_renew) in subs {
                    if auto_renew {
                        if let Err(e) = self.renew_blob(subscriber, current_epoch, hash) {
                            // Warn and skip down to delete
                            warn!("failed to renew blob {} for {}: {}", hash, subscriber, e);
                        } else {
                            num_renewed += 1;
                            continue;
                        }
                    }
                    match self.delete_blob(subscriber, subscriber, subscriber, current_epoch, hash)
                    {
                        Ok(from_disc) => {
                            num_deleted += 1;
                            if from_disc {
                                delete_from_disc.insert(hash);
                            }
                        }
                        Err(e) => {
                            warn!("failed to delete blob {} for {}: {}", hash, subscriber, e)
                        }
                    }
                }
            }
        }
        debug!("renewed {} expired subscriptions", num_renewed);
        debug!("deleted {} expired subscriptions", num_deleted);
        debug!(
            "{} blobs marked for deletion from disc",
            delete_from_disc.len()
        );
        // Debit for existing usage
        for (address, account) in self.accounts.iter_mut() {
            let debit_blocks = current_epoch - account.last_debit_epoch;
            let debit = debit_blocks as u64 * &account.capacity_used;
            self.credit_debited += &debit;
            self.credit_committed -= &debit;
            account.credit_committed -= &debit;
            account.last_debit_epoch = current_epoch;
            debug!("debited {} credits from {}", debit, address);
        }
        Ok(delete_from_disc)
    }

    /// Add a blob
    ///
    /// @param origin - the address that is submitting the transaction to add this blob
    /// @param caller - the address that is calling into this function.  If the blobs actor is
    ///   accessed directly, this will be the same as "origin". But most of the time this will be
    ///   the address of the actor instance that is calling into the blobs actor. I.e a specific
    ///   Bucket or Timehub instance.
    /// @param subscriber - the address responsible for the subscription to keep this blob around.
    ///  This is whose credits will be spent by this transaction, and going forward to continue to
    ///  pay for the blob over time. Generally this is the owner of the wrapping Actor
    ///  (e.g. Buckets, Timehub).
    #[allow(clippy::too_many_arguments)]
    pub fn add_blob(
        &mut self,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        metadata_hash: Hash,
        size: u64,
        ttl: Option<ChainEpoch>,
        source: PublicKey,
        tokens_received: TokenAmount,
    ) -> anyhow::Result<(Subscription, TokenAmount), ActorError> {
        let (ttl, auto_renew) = accept_ttl(ttl)?;
        let account = self
            .accounts
            .entry(subscriber)
            .or_insert(Account::new(BigInt::zero(), current_epoch));
        let delegation = if origin != subscriber {
            // First look for an approval for origin keyed by origin, which denotes it's valid for
            // any caller.
            // Second look for an approval for the supplied caller.
            let approval = if let Some(approvals) = account.approvals.get_mut(&origin) {
                if let Some(approval) = approvals.get_mut(&origin) {
                    Some(approval)
                } else {
                    approvals.get_mut(&caller)
                }
            } else {
                None
            };
            let approval = approval.ok_or(ActorError::forbidden(format!(
                "approval from {} to {} via caller {} not found",
                subscriber, origin, caller
            )))?;
            Some(CreditDelegation::new(origin, caller, approval))
        } else {
            None
        };
        // Capacity updates and required credit depend on whether the subscriber is already
        // subcribing to this blob
        let size = BigInt::from(size);
        let expiry = current_epoch + ttl;
        let mut new_capacity = BigInt::zero();
        let mut new_account_capacity = BigInt::zero();
        let credit_required: BigInt;
        // Like cashback but for sending unspent tokens back
        let tokens_unspent: TokenAmount;

        let sub = if let Some(blob) = self.blobs.get_mut(&hash) {
            let sub = if let Some(sub) = blob.subs.get_mut(&subscriber) {
                // Required credit can be negative if subscriber is reducing expiry
                credit_required = (expiry - sub.expiry) as u64 * &size;

                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &tokens_received,
                    self.credit_debit_rate,
                    &subscriber,
                    current_epoch,
                    &delegation,
                )?;
                // Update expiry index
                if expiry != sub.expiry {
                    update_expiry_index(
                        &mut self.expiries,
                        subscriber,
                        hash,
                        Some((expiry, auto_renew)),
                        Some(sub.expiry),
                    );
                }
                sub.expiry = expiry;
                sub.auto_renew = auto_renew;
                // Overwrite source allows subscriber to retry resolving
                sub.source = source;
                sub.delegate = delegation.as_ref().map(|d| d.addresses());
                debug!("updated subscription to {} for {}", hash, subscriber);
                sub.clone()
            } else {
                new_account_capacity = size.clone();
                // One or more accounts have already committed credit.
                // However, we still need to reserve the full required credit from the new
                // subscriber, as the existing account(s) may decide to change the
                // expiry or cancel.
                credit_required = ttl as u64 * &size;
                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &tokens_received,
                    self.credit_debit_rate,
                    &subscriber,
                    current_epoch,
                    &delegation,
                )?;
                // Add new subscription
                let sub = Subscription {
                    added: current_epoch,
                    expiry,
                    auto_renew,
                    source,
                    delegate: delegation.as_ref().map(|d| d.addresses()),
                };
                blob.subs.insert(subscriber, sub.clone());
                debug!("created new subscription to {} for {}", hash, subscriber);
                // Update expiry index
                update_expiry_index(
                    &mut self.expiries,
                    subscriber,
                    hash,
                    Some((expiry, auto_renew)),
                    None,
                );
                sub
            };
            if !matches!(blob.status, BlobStatus::Failed) {
                // It's pending or failed, reset to "added"
                blob.status = BlobStatus::Added;
                // Add/update "added" with hash and its source
                self.added
                    .entry(hash)
                    .and_modify(|sources| {
                        sources.insert((subscriber, source));
                    })
                    .or_insert(HashSet::from([(subscriber, source)]));
            }
            sub
        } else {
            new_account_capacity = size.clone();
            // New blob increases network capacity as well.
            // Ensure there is enough capacity available.
            let available_capacity = &self.capacity_total - &self.capacity_used;

            if size > available_capacity {
                return Err(ActorError::forbidden(format!(
                    "subnet has insufficient storage capacity (available: {}; required: {})",
                    available_capacity, size
                )));
            }
            new_capacity = size.clone();
            credit_required = ttl as u64 * &size;
            tokens_unspent = ensure_credit_or_buy(
                &mut account.credit_free,
                &mut self.credit_sold,
                &credit_required,
                &tokens_received,
                self.credit_debit_rate,
                &subscriber,
                current_epoch,
                &delegation,
            )?;
            // Create new blob
            let sub = Subscription {
                added: current_epoch,
                expiry,
                auto_renew,
                source,
                delegate: delegation.as_ref().map(|d| d.addresses()),
            };
            let blob = Blob {
                size: size.to_u64().unwrap(),
                subs: HashMap::from([(subscriber, sub.clone())]),
                status: BlobStatus::Added,
                metadata_hash,
            };
            self.blobs.insert(hash, blob);
            debug!("created new blob {}", hash);
            debug!("created new subscription to {} for {}", hash, subscriber);
            // Update expiry index
            update_expiry_index(
                &mut self.expiries,
                subscriber,
                hash,
                Some((expiry, auto_renew)),
                None,
            );
            // Add to "added" map
            self.added
                .insert(hash, HashSet::from([(subscriber, source)]));
            sub
        };
        // Account capacity is changing, debit for existing usage
        let debit_blocks = current_epoch - account.last_debit_epoch;
        let debit = debit_blocks as u64 * &account.capacity_used;
        self.credit_debited += &debit;
        self.credit_committed -= &debit;
        account.credit_committed -= &debit;
        account.last_debit_epoch = current_epoch;
        debug!("debited {} credits from {}", debit, subscriber);
        // Account for new size and move free credit to committed credit
        self.capacity_used += &new_capacity;
        debug!("used {} bytes from subnet", new_account_capacity);
        account.capacity_used += &new_account_capacity;
        debug!("used {} bytes from {}", new_account_capacity, subscriber);
        self.credit_committed += &credit_required;
        account.credit_committed += &credit_required;
        account.credit_free -= &credit_required;
        // Update credit approval
        if let Some(delegation) = delegation {
            delegation.approval.committed += &credit_required;
        }
        if credit_required.is_positive() {
            debug!("committed {} credits from {}", credit_required, subscriber);
        } else {
            debug!(
                "released {} credits to {}",
                credit_required.magnitude(),
                subscriber
            );
        }
        Ok((sub, tokens_unspent))
    }

    fn renew_blob(
        &mut self,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
    ) -> anyhow::Result<Account, ActorError> {
        let account = self
            .accounts
            .entry(subscriber)
            .or_insert(Account::new(BigInt::zero(), current_epoch));
        let blob = self
            .blobs
            .get_mut(&hash)
            .ok_or(ActorError::not_found(format!("blob {} not found", hash)))?;
        if matches!(blob.status, BlobStatus::Failed) {
            // Do not renew failed blobs.
            return Err(ActorError::illegal_state(format!(
                "cannot renew failed blob {}",
                hash
            )));
        }
        let sub = blob
            .subs
            .get_mut(&subscriber)
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, hash
            )))?;
        let delegation = if let Some((origin, caller)) = sub.delegate {
            // First look for an approval for origin keyed by origin, which denotes it's valid for
            // any caller.
            // Second look for an approval for the supplied caller.
            let approval = if let Some(approvals) = account.approvals.get_mut(&origin) {
                if let Some(approval) = approvals.get_mut(&origin) {
                    Some(approval)
                } else {
                    approvals.get_mut(&caller)
                }
            } else {
                None
            };
            let approval = approval.ok_or(ActorError::forbidden(format!(
                "approval from {} to {} via caller {} not found",
                subscriber, origin, caller
            )))?;
            Some(CreditDelegation::new(origin, caller, approval))
        } else {
            None
        };
        let size = BigInt::from(blob.size);
        // Since the charge will be for all the account's blobs, we can only
        // account for capacity up to this blob's expiry if it is less than
        // the current epoch.
        let debit_epoch = sub.expiry.min(current_epoch);
        // Account capacity is not changing, but we debit for existing usage here because the
        // subscriber might have a refund that could impact their ability to renew.
        // Debit for existing usage up to old expiry.
        // It could be possible that old expiry is less than the last debit,
        // in which case we need to refund for that duration.
        if debit_epoch > account.last_debit_epoch {
            let debit_blocks = debit_epoch - account.last_debit_epoch;
            let debit = debit_blocks as u64 * &account.capacity_used;
            self.credit_debited += &debit;
            self.credit_committed -= &debit;
            account.credit_committed -= &debit;
            account.last_debit_epoch = debit_epoch;
            debug!("debited {} credits from {}", debit, subscriber);
        } else {
            // The account was debited after this blob's expiry
            let refund_blocks = account.last_debit_epoch - sub.expiry;
            let refund = refund_blocks as u64 * &size;
            account.credit_free += &refund; // re-mint spent credit
            self.credit_debited -= &refund;
            debug!("refunded {} credits to {}", refund, subscriber);
        }
        // Ensure subscriber still has enough credits.
        let expiry = sub.expiry + AUTO_TTL;
        let credit_required = AUTO_TTL as u64 * &size;
        ensure_credit(
            &subscriber,
            current_epoch,
            &account.credit_free,
            &credit_required,
            &delegation,
        )?;
        // Update expiry index
        if expiry != sub.expiry {
            update_expiry_index(
                &mut self.expiries,
                subscriber,
                hash,
                Some((expiry, sub.auto_renew)),
                Some(sub.expiry),
            );
        }
        sub.expiry = expiry;
        debug!("renewed subscription to {} for {}", hash, subscriber);
        // Move free credit to committed credit
        self.credit_committed += &credit_required;
        account.credit_committed += &credit_required;
        account.credit_free -= &credit_required;
        // Update credit approval
        if let Some(delegation) = delegation {
            delegation.approval.committed += &credit_required;
        }
        debug!("committed {} credits from {}", credit_required, subscriber);
        Ok(account.clone())
    }

    pub fn get_blob(&self, hash: Hash) -> Option<Blob> {
        self.blobs.get(&hash).cloned()
    }

    pub fn get_blob_status(&self, hash: Hash, subscriber: Address) -> Option<BlobStatus> {
        let blob = self.blobs.get(&hash)?;
        if blob.subs.contains_key(&subscriber) {
            Some(blob.status.clone())
        } else {
            None
        }
    }

    pub fn get_added_blobs(&self, size: u32) -> Vec<(Hash, HashSet<(Address, PublicKey)>)> {
        self.added
            .iter()
            .take(size as usize)
            .map(|element| (*element.0, element.1.clone()))
            .collect::<Vec<_>>()
    }

    pub fn get_pending_blobs(&self, size: u32) -> Vec<(Hash, HashSet<(Address, PublicKey)>)> {
        self.pending
            .iter()
            .take(size as usize)
            .map(|element| (*element.0, element.1.clone()))
            .collect::<Vec<_>>()
    }

    pub fn set_pending(&mut self, subscriber: Address, hash: Hash, source: PublicKey) {
        // set blob status to pending
        let blob = self.blobs.get_mut(&hash).unwrap();
        blob.status = BlobStatus::Pending;
        // Add to pending map
        self.pending
            .entry(hash)
            .and_modify(|sources| {
                sources.insert((subscriber, source));
            })
            .or_insert(HashSet::from([(subscriber, source)]));
        // Remove from added map
        self.added.remove(&hash);
    }

    pub fn finalize_blob(
        &mut self,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        status: BlobStatus,
    ) -> anyhow::Result<(), ActorError> {
        if matches!(status, BlobStatus::Pending) {
            return Err(ActorError::illegal_state(format!(
                "cannot finalize pending blob {}",
                hash
            )));
        }
        let account = self
            .accounts
            .entry(subscriber)
            .or_insert(Account::new(BigInt::zero(), current_epoch));
        let blob = if let Some(blob) = self.blobs.get_mut(&hash) {
            blob
        } else {
            // The blob may have been deleted before it was finalized
            return Ok(());
        };
        if matches!(blob.status, BlobStatus::Resolved) {
            // Blob is already finalized as resolved.
            // We can ignore later finalizations, even if they are failed.
            return Ok(());
        }
        let sub = blob
            .subs
            .get(&subscriber)
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, hash
            )))?;
        // Do not error if the approval was removed while this blob was pending
        let delegation = if let Some((origin, caller)) = sub.delegate {
            // First look for an approval for origin keyed by origin, which denotes it's valid for
            // any caller.
            // Second look for an approval for the supplied caller.
            let approval = if let Some(approvals) = account.approvals.get_mut(&origin) {
                if let Some(approval) = approvals.get_mut(&origin) {
                    Some(approval)
                } else {
                    approvals.get_mut(&caller)
                }
            } else {
                None
            };
            approval.map(|approval| CreditDelegation::new(origin, caller, approval))
        } else {
            None
        };
        // Update blob status
        blob.status = status;
        debug!("finalized blob {} to status {}", hash, blob.status);
        if matches!(blob.status, BlobStatus::Failed) {
            let size = BigInt::from(blob.size);
            // We're not going to make a debit, but we need to refund
            // any spent credits that may have been used on this
            // blob in the event the last debit is later than the
            // added epoch.
            if account.last_debit_epoch > sub.added {
                let refund_blocks = account.last_debit_epoch - sub.added;
                let refund = refund_blocks as u64 * &size;
                account.credit_free += &refund; // re-mint spent credit
                self.credit_debited -= &refund;
                debug!("refunded {} credits to {}", refund, subscriber);
            }
            // Account for reclaimed size and move committed credit to
            // free credit
            self.capacity_used -= &size;
            debug!("released {} bytes to subnet", size);
            account.capacity_used -= &size;
            debug!("released {} bytes to {}", size, subscriber);
            if sub.expiry > account.last_debit_epoch {
                let reclaim = (sub.expiry - account.last_debit_epoch) * &size;
                self.credit_committed -= &reclaim;
                account.credit_committed -= &reclaim;
                account.credit_free += &reclaim;
                // Update credit approval
                if let Some(delegation) = delegation {
                    delegation.approval.committed -= &reclaim;
                }
                debug!("released {} credits to {}", reclaim, subscriber);
            }
        }
        // Remove entry from pending
        if let Some(entry) = self.pending.get_mut(&hash) {
            entry.remove(&(subscriber, sub.source));
            if entry.is_empty() {
                self.pending.remove(&hash);
            }
        }
        Ok(())
    }

    pub fn delete_blob(
        &mut self,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
    ) -> anyhow::Result<bool, ActorError> {
        let account = self
            .accounts
            .entry(subscriber)
            .or_insert(Account::new(BigInt::zero(), current_epoch));
        let blob = if let Some(blob) = self.blobs.get_mut(&hash) {
            blob
        } else {
            // We could error here, but since this method is called from other actors,
            // they would need to be able to identify this specific case.
            // For example, the bucket actor may need to delete a blob while overwriting
            // an existing key.
            // However, the system may have already deleted the blob due to expiration or
            // insufficient funds.
            // We could use a custom error code, but this is easier.
            return Ok(false);
        };
        let sub = blob
            .subs
            .get(&subscriber)
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, hash
            )))?;
        let delegation = if let Some((origin, caller)) = sub.delegate {
            // First look for an approval for origin keyed by origin, which denotes it's valid for
            // any caller.
            // Second look for an approval for the supplied caller.
            let approval = if let Some(approvals) = account.approvals.get_mut(&origin) {
                if let Some(approval) = approvals.get_mut(&origin) {
                    Some(approval)
                } else {
                    approvals.get_mut(&caller)
                }
            } else {
                None
            };
            if let Some(approval) = approval {
                Some(CreditDelegation::new(origin, caller, approval))
            } else {
                // Approval may have been removed, or this is a call from the system actor,
                // in which case the origin will be supplied as the subscriber
                if origin != subscriber {
                    return Err(ActorError::forbidden(format!(
                        "approval from {} to {} via caller {} not found",
                        subscriber, origin, caller
                    )));
                }
                None
            }
        } else {
            None
        };
        // If the subscription does not have a delegate, the caller must be the subscriber.
        // If the subscription has a delegate, it must be the caller or the
        // caller must be the subscriber.
        match &delegation {
            None => {
                if origin != subscriber {
                    return Err(ActorError::forbidden(format!(
                        "origin {} is not subscriber {} for blob {}",
                        caller, subscriber, hash
                    )));
                }
            }
            Some(delegation) => {
                if !(origin == delegation.origin && caller == delegation.caller)
                    && origin != subscriber
                {
                    return Err(ActorError::forbidden(format!(
                        "origin {} is not delegate origin {} or caller {} is not delegate caller {} or subscriber {} for blob {}",
                        origin, delegation.origin, caller, delegation.caller, subscriber, hash
                    )));
                }
                if let Some(expiry) = delegation.approval.expiry {
                    if expiry <= current_epoch {
                        return Err(ActorError::forbidden(format!(
                            "approval from {} to {} via caller {} expired",
                            subscriber, delegation.origin, delegation.caller
                        )));
                    }
                }
            }
        }
        // Since the charge will be for all the account's blobs, we can only
        // account for capacity up to this blob's expiry if it is less than
        // the current epoch.
        let debit_epoch = sub.expiry.min(current_epoch);
        // Account capacity is changing, debit for existing usage.
        // It could be possible that debit epoch is less than the last debit,
        // in which case we need to refund for that duration.
        if debit_epoch > account.last_debit_epoch {
            let debit_blocks = debit_epoch - account.last_debit_epoch;
            let debit = debit_blocks as u64 * &account.capacity_used;
            self.credit_debited += &debit;
            self.credit_committed -= &debit;
            account.credit_committed -= &debit;
            account.last_debit_epoch = debit_epoch;
            debug!("debited {} credits from {}", debit, subscriber);
        } else {
            // The account was debited after this blob's expiry
            let refund_blocks = account.last_debit_epoch - sub.expiry;
            let refund = refund_blocks as u64 * &BigInt::from(blob.size);
            account.credit_free += &refund; // re-mint spent credit
            self.credit_debited -= &refund;
            debug!("refunded {} credits to {}", refund, subscriber);
        }
        // Account for reclaimed size and move committed credit to free credit
        // If blob failed, capacity and committed credits have already been returned
        if !matches!(blob.status, BlobStatus::Failed) {
            let size = BigInt::from(blob.size);
            account.capacity_used -= &size;
            if blob.subs.is_empty() {
                self.capacity_used -= &size;
                debug!("released {} bytes to subnet", size);
            }
            debug!("released {} bytes to {}", size, subscriber);
            // We can release credits if expiry is in the future
            if debit_epoch == current_epoch {
                let reclaim = (sub.expiry - debit_epoch) * &size;
                self.credit_committed -= &reclaim;
                account.credit_committed -= &reclaim;
                account.credit_free += &reclaim;
                // Update credit approval
                if let Some(delegation) = delegation {
                    delegation.approval.committed -= &reclaim;
                }
                debug!("released {} credits to {}", reclaim, subscriber);
            }
        }
        // Update expiry index
        update_expiry_index(&mut self.expiries, subscriber, hash, None, Some(sub.expiry));
        // Remove entry from the "added" map if it exists
        if let Some(entry) = self.added.get_mut(&hash) {
            entry.remove(&(subscriber, sub.source));
            if entry.is_empty() {
                self.added.remove(&hash);
            }
        }

        // Remove entry from pending
        if let Some(entry) = self.pending.get_mut(&hash) {
            entry.remove(&(subscriber, sub.source));
            if entry.is_empty() {
                self.pending.remove(&hash);
            }
        }
        // Delete subscription
        blob.subs.remove(&subscriber);
        debug!("deleted subscription to {} for {}", hash, subscriber);
        // Delete or update blob
        let delete_blob = blob.subs.is_empty();
        if delete_blob {
            self.blobs.remove(&hash);
            debug!("deleted blob {}", hash);
        }
        Ok(delete_blob)
    }

    /// Return available capacity as a difference between `capacity_total` and `capacity_used`.
    fn capacity_available(&self) -> BigInt {
        &self.capacity_total - &self.capacity_used
    }
}

/// Check if `subscriber` has enough credits, including delegated credits.
fn ensure_credit(
    subscriber: &Address,
    current_epoch: ChainEpoch,
    credit_free: &BigInt,
    required_credit: &BigInt,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    ensure_enough_credits(subscriber, credit_free, required_credit)?;
    ensure_delegated_credit(subscriber, current_epoch, required_credit, delegation)
}

/// Check if `subscriber` owns enough free credits.
fn ensure_enough_credits(
    subscriber: &Address,
    credit_free: &BigInt,
    required_credit: &BigInt,
) -> anyhow::Result<(), ActorError> {
    if credit_free >= required_credit {
        Ok(())
    } else {
        Err(ActorError::insufficient_funds(format!(
            "account {} has insufficient credit (available: {}; required: {})",
            subscriber, credit_free, required_credit
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn ensure_credit_or_buy(
    account_credit_free: &mut BigInt,
    state_credit_sold: &mut BigInt,
    credit_required: &BigInt,
    tokens_received: &TokenAmount,
    debit_credit_rate: u64,
    subscriber: &Address,
    current_epoch: ChainEpoch,
    delegate: &Option<CreditDelegation>,
) -> anyhow::Result<TokenAmount, ActorError> {
    let tokens_received_non_zero = !tokens_received.is_zero();
    let has_delegation = delegate.is_some();
    match (tokens_received_non_zero, has_delegation) {
        (true, true) => Err(ActorError::illegal_argument(format!(
            "can not buy credits inline for {}",
            subscriber,
        ))),
        (true, false) => {
            // Try buying credits for self
            let not_enough_credits = *account_credit_free < *credit_required;
            if not_enough_credits {
                let credits_needed = credit_required - &*account_credit_free;
                let tokens_needed_atto = &credits_needed / debit_credit_rate;
                let tokens_needed = TokenAmount::from_atto(tokens_needed_atto);
                if tokens_needed <= *tokens_received {
                    let tokens_to_rebate = tokens_received - tokens_needed;
                    *state_credit_sold += &credits_needed;
                    *account_credit_free += &credits_needed;
                    Ok(tokens_to_rebate)
                } else {
                    Err(ActorError::insufficient_funds(format!(
                        "account {} sent insufficient tokens (received: {}; required: {})",
                        subscriber, tokens_received, tokens_needed
                    )))
                }
            } else {
                Ok(TokenAmount::zero())
            }
        }
        (false, true) => {
            ensure_credit(
                subscriber,
                current_epoch,
                account_credit_free,
                credit_required,
                delegate,
            )?;
            Ok(TokenAmount::zero())
        }
        (false, false) => {
            ensure_credit(
                subscriber,
                current_epoch,
                account_credit_free,
                credit_required,
                delegate,
            )?;
            Ok(TokenAmount::zero())
        }
    }
}

fn ensure_delegated_credit(
    subscriber: &Address,
    current_epoch: ChainEpoch,
    required_credit: &BigInt,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    if let Some(delegation) = delegation {
        if let Some(limit) = &delegation.approval.limit {
            let uncommitted = &(limit - &delegation.approval.committed);
            if uncommitted < required_credit {
                return Err(ActorError::insufficient_funds(format!(
                    "approval from {} to {} via caller {} has insufficient credit (available: {}; required: {})",
                    subscriber, delegation.origin, delegation.caller, uncommitted, required_credit
                )));
            }
        }
        if let Some(expiry) = delegation.approval.expiry {
            if expiry <= current_epoch {
                return Err(ActorError::forbidden(format!(
                    "approval from {} to {} via caller {} expired",
                    subscriber, delegation.origin, delegation.caller
                )));
            }
        }
    }
    Ok(())
}

fn update_expiry_index(
    expiries: &mut BTreeMap<ChainEpoch, HashMap<Address, HashMap<Hash, bool>>>,
    subscriber: Address,
    hash: Hash,
    add: Option<(ChainEpoch, bool)>,
    remove: Option<ChainEpoch>,
) {
    if let Some((add, auto_renew)) = add {
        expiries
            .entry(add)
            .and_modify(|entry| {
                entry
                    .entry(subscriber)
                    .and_modify(|subs| {
                        subs.insert(hash, auto_renew);
                    })
                    .or_insert(HashMap::from([(hash, auto_renew)]));
            })
            .or_insert(HashMap::from([(
                subscriber,
                HashMap::from([(hash, auto_renew)]),
            )]));
    }
    if let Some(remove) = remove {
        if let Some(entry) = expiries.get_mut(&remove) {
            if let Some(subs) = entry.get_mut(&subscriber) {
                subs.remove(&hash);
                if subs.is_empty() {
                    entry.remove(&subscriber);
                }
            }
            if entry.is_empty() {
                expiries.remove(&remove);
            }
        }
    }
}

fn accept_ttl(ttl: Option<ChainEpoch>) -> anyhow::Result<(ChainEpoch, bool), ActorError> {
    let (ttl, auto_renew) = ttl.map(|ttl| (ttl, false)).unwrap_or((AUTO_TTL, true));
    if ttl < MIN_TTL {
        Err(ActorError::illegal_argument(format!(
            "minimum blob TTL is {}",
            MIN_TTL
        )))
    } else {
        Ok((ttl, auto_renew))
    }
}
