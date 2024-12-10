// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Bound::{Included, Unbounded};

use cid::Cid;
use fendermint_actor_blobs_shared::params::GetStatsReturn;
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, CreditAllowance, CreditApproval, Hash, PublicKey, Subscription,
    SubscriptionGroup, SubscriptionId, TtlStatus,
};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::{BigInt, BigUint};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use log::{debug, warn};
use num_traits::{Signed, ToPrimitive, Zero};

use crate::accounts::Accounts;
use crate::hamt_blobs::HamtBlobsRoot;

/// The minimum epoch duration a blob can be stored.
const MIN_TTL: ChainEpoch = 3600; // one hour
/// The rolling epoch duration used for non-expiring blobs.
const AUTO_TTL: ChainEpoch = 3600; // one hour

// pub type BlobMap<'a, BS> = Map<BS, Hash, Blob>;

/// The state represents all accounts and stored blobs.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total storage capacity of the subnet.
    /// TODO: Remove this in favor of the value in the hoku config actor
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
    /// TODO: Remove this in favor of the value in the hoku config actor
    pub blob_credits_per_byte_block: u64,
    /// Map containing all accounts by robust (non-ID) actor address.

    /// Map containing all blobs.
    // pub blobs: HashMap<Hash, Blob>,
    /// Map of expiries to blob hashes.
    pub expiries: BTreeMap<ChainEpoch, HashMap<Address, HashMap<ExpiryKey, bool>>>,
    /// Map of currently added blob hashes to account and source Iroh node IDs.
    pub added: BTreeMap<Hash, HashSet<(Address, SubscriptionId, PublicKey)>>,
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: BTreeMap<Hash, HashSet<(Address, SubscriptionId, PublicKey)>>,
    pub accounts_root: Cid,
    pub blobs_root: HamtBlobsRoot
}

/// Key used to namespace subscriptions in the expiry index.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct ExpiryKey {
    /// Key hash.
    pub hash: Hash,
    /// Key subscription ID.
    pub id: SubscriptionId,
}

impl ExpiryKey {
    /// Create a new expiry key.
    pub fn new(hash: Hash, id: &SubscriptionId) -> Self {
        Self {
            hash,
            id: id.clone(),
        }
    }
}

/// Helper for handling credit approvals.
struct CreditDelegation<'a> {
    /// The address that is submitting the transaction to add this blob.
    pub origin: Address,
    /// The address that is calling into the blob actor.
    /// If the blob actor is accessed directly, this will be the same as "origin".
    /// However, most of the time this will be the address of the actor instance that is
    /// calling into the blobs actor, i.e., a specific Bucket or Timehub instance.
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

    /// Tuple of (Origin, Caller) addresses.
    pub fn addresses(&self) -> (Address, Address) {
        (self.origin, self.caller)
    }
}

impl State {
    pub fn new<BS: Blockstore>(
        store: &BS,
        blob_capacity: u64,
        blob_credits_per_byte_block: u64,
    ) -> anyhow::Result<Self, ActorError> {
        Ok(Self {
            capacity_total: BigInt::from(blob_capacity),
            capacity_used: BigInt::zero(),
            credit_sold: BigInt::zero(),
            credit_committed: BigInt::zero(),
            credit_debited: BigInt::zero(),
            blob_credits_per_byte_block,
            // blobs: HashMap::new(),
            expiries: BTreeMap::new(),
            added: BTreeMap::new(),
            pending: BTreeMap::new(),
            accounts_root: Accounts::flush_empty(store)?,
            blobs_root: HamtBlobsRoot::flush_empty(store)?,
        })
    }

    // TODO: Don't calculate stats on the fly, use running counters: num_accounts, num_blobs, bytes_resolving, bytes_added
    pub fn get_stats(&self, balance: TokenAmount) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(),
            capacity_used: self.capacity_used.clone(),
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            blob_credits_per_byte_block: self.blob_credits_per_byte_block,
            // num_accounts: self.accounts.len() as u64,
            // num_blobs: self.blobs.len() as u64, FIXME SU
            num_resolving: self.pending.len() as u64,
            // bytes_resolving: self.pending.keys().map(|hash| self.blobs[hash].size).sum(), FIXME SU Use running counter
            num_added: self.added.len() as u64,
            // bytes_added: self.added.keys().map(|hash| self.blobs[hash].size).sum(), FIXME SU Use running counter
        }
    }

    pub fn buy_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        to: Address,
        amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<Account, ActorError> {
        if amount.is_negative() {
            return Err(ActorError::illegal_argument(
                "token amount must be positive".into(),
            ));
        }
        let credits = amount.atto().clone();
        // Don't sell credits if we're at storage capacity
        if self.capacity_available().is_zero() {
            return Err(ActorError::forbidden(
                "credits not available (subnet has reached storage capacity)".into(),
            ));
        }
        self.credit_sold += &credits;
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&to, current_epoch)?;
        account.credit_free += &credits;
        // Save account
        self.accounts_root = accounts.set_and_flush(&to, account.clone())?;

        debug!("sold {} credits to {}", credits, to);
        Ok(account)
    }

    pub fn update_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        add_amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let addr = sponsor.unwrap_or(from);
        // Get the account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_err(&addr)?;
        let delegation = if let Some(sponsor) = sponsor {
            let approval =
                account
                    .approvals
                    .get_mut(&from.to_string())
                    .ok_or(ActorError::forbidden(format!(
                        "approval from {} to {} not found",
                        sponsor, from
                    )))?;
            Some(CreditDelegation::new(from, from, approval))
        } else {
            None
        };
        // Check credit balance and debit
        let add_credit = add_amount.atto().clone();
        if add_credit.is_negative() {
            let credit_required = -add_credit.clone();
            ensure_credit(
                &addr,
                current_epoch,
                &account.credit_free,
                &credit_required,
                &delegation,
            )?;
        }
        self.credit_debited -= &add_credit;
        account.credit_free += &add_credit;
        // Update credit approval
        if let Some(delegation) = delegation {
            delegation.approval.used -= &add_credit;
        }
        // Save account
        self.accounts_root = accounts.set_and_flush(&addr, account)?;

        if add_credit.is_positive() {
            debug!("refunded {} credits to {}", add_credit, addr);
        } else {
            debug!("debited {} credits from {}", add_credit.magnitude(), addr);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn approve_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        to: Address,
        caller_allowlist: Option<HashSet<Address>>,
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
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&from, current_epoch)?;
        // Get or add a new approval
        let approval = account
            .approvals
            .entry(to.to_string())
            .or_insert(CreditApproval {
                limit: limit.clone(),
                expiry,
                used: BigInt::zero(),
                caller_allowlist: caller_allowlist.clone(),
            });
        // Validate approval changes
        if let Some(limit) = limit.clone() {
            if approval.used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used credits ({})",
                    approval.used
                )));
            }
        }
        approval.limit = limit;
        approval.expiry = expiry;
        approval.caller_allowlist = caller_allowlist.clone();
        let approval = approval.clone();
        // Save account
        self.accounts_root = accounts.set_and_flush(&from, account)?;

        debug!(
            "approved credits from {} to {} (limit: {:?}; expiry: {:?}, caller_allowlist: {:?})",
            from, to, approval.limit, approval.expiry, caller_allowlist
        );
        Ok(approval)
    }

    /// Revokes credit from one account to another.
    /// If a caller is specified, this will remove it from the caller allowlist.
    /// It is not possible to use this method to transform an approval with a caller allowlist
    /// into one without a caller allowlist.
    pub fn revoke_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        to: Address,
        for_caller: Option<Address>,
    ) -> anyhow::Result<(), ActorError> {
        // Get the account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_err(&from)?;
        if let Some(caller) = for_caller {
            let approval =
                account
                    .approvals
                    .get_mut(&to.to_string())
                    .ok_or(ActorError::not_found(format!(
                        "approval from {} to {} not found",
                        from, to,
                    )))?;
            if !approval.remove_caller(&caller) {
                return Err(ActorError::not_found(format!(
                    "approval from {} to {} via caller {} not found",
                    from, to, caller
                )));
            } else if !approval.has_allowlist() {
                // Remove the entire approval.
                // Otherwise, removing this caller would make the approval valid for any caller,
                // which is likely not the intention.
                account.approvals.remove(&to.to_string());
            }
        } else if account.approvals.remove(&to.to_string()).is_none() {
            return Err(ActorError::not_found(format!(
                "approval from {} to {} not found",
                from, to
            )));
        }
        // Save account
        self.accounts_root = accounts.set_and_flush(&from, account)?;

        debug!(
            "revoked credits from {} to {} (required_caller: {:?})",
            from, to, for_caller
        );
        Ok(())
    }

    pub fn get_account<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
    ) -> anyhow::Result<Option<Account>, ActorError> {
        let accounts = Accounts::load(store, &self.accounts_root)?;
        accounts.get(&from)
    }

    /// Returns a [`CreditApproval`] from the given address to the given address
    /// or [`None`] if no approval exists.
    pub fn get_credit_approval<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
        to: Address,
    ) -> anyhow::Result<Option<CreditApproval>, ActorError> {
        let accounts = Accounts::load(store, &self.accounts_root)?;
        Ok(accounts
            .get(&from)?
            .map(|a| a.approvals.get(&to.to_string()).cloned())
            .and_then(|a| a))
    }

    /// Returns the free credit for the given address, including an amount from a default sponsor.
    /// Note: An error returned from this method would be fatal, as it's called from the FVM executor.
    pub fn get_credit_allowance<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<CreditAllowance, ActorError> {
        // Get the account or return default allowance
        let accounts = Accounts::load(store, &self.accounts_root)?;
        let account = match accounts.get(&from)? {
            None => return Ok(CreditAllowance::default()),
            Some(account) => account,
        };
        let mut allowance = CreditAllowance {
            amount: TokenAmount::from_atto(account.credit_free.clone()),
            ..Default::default()
        };
        if let Some(credit_sponsor) = account.credit_sponsor {
            let sponsor = match accounts.get(&credit_sponsor)? {
                None => return Ok(allowance),
                Some(account) => account,
            };
            let sponsored = sponsor
                .approvals
                .get(&from.to_string())
                .and_then(|approval| {
                    let expiry_valid = approval
                        .expiry
                        .map_or(true, |expiry| expiry > current_epoch);
                    if !expiry_valid {
                        return None;
                    }
                    let credit_free = sponsor.credit_free.clone();
                    let used = approval.used.clone();
                    let amount = approval
                        .limit
                        .clone()
                        .map_or(credit_free.clone(), |limit| (limit - used).min(credit_free));
                    Some(TokenAmount::from_atto(amount))
                })
                .unwrap_or(TokenAmount::zero());
            allowance.sponsor = Some(credit_sponsor);
            allowance.sponsored_amount = sponsored;
        } else {
            return Ok(allowance);
        }
        Ok(allowance)
    }

    pub fn set_credit_sponsor<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&from, current_epoch)?;
        account.credit_sponsor = sponsor;
        // Save account
        self.accounts_root = accounts.set_and_flush(&from, account)?;

        debug!("set credit sponsor for {} to {:?}", from, sponsor);
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn debit_accounts<BS: Blockstore>(
        &mut self,
        store: &BS,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<HashSet<Hash>, ActorError> {
        // Delete expired subscriptions
        let mut delete_from_disc = HashSet::new();
        let expiries: Vec<(ChainEpoch, HashMap<Address, HashMap<ExpiryKey, bool>>)> = self
            .expiries
            .range((Unbounded, Included(current_epoch)))
            .map(|(expiry, entry)| (*expiry, entry.clone()))
            .collect();
        let mut num_renewed = 0;
        let mut num_deleted = 0;
        for (_, entry) in expiries {
            for (subscriber, subs) in entry {
                for (key, auto_renew) in subs {
                    if auto_renew {
                        if let Err(e) = self.renew_blob(
                            store,
                            subscriber,
                            current_epoch,
                            key.hash,
                            key.id.clone(),
                        ) {
                            // Warn and skip down to delete
                            warn!(
                                "failed to renew blob {} for {} (id: {}): {}",
                                key.hash, subscriber, key.id, e
                            );
                        } else {
                            num_renewed += 1;
                            continue;
                        }
                    }
                    match self.delete_blob(
                        store,
                        subscriber,
                        subscriber,
                        subscriber,
                        current_epoch,
                        key.hash,
                        key.id.clone(),
                    ) {
                        Ok(from_disc) => {
                            num_deleted += 1;
                            if from_disc {
                                delete_from_disc.insert(key.hash);
                            }
                        }
                        Err(e) => {
                            warn!(
                                "failed to delete blob {} for {} (id: {}): {}",
                                key.hash, subscriber, key.id, e
                            )
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
        let reader = Accounts::load(store, &self.accounts_root)?;
        let mut writer = Accounts::load(store, &self.accounts_root)?;
        reader.map.for_each(|address, account| {
            let mut account = account.clone();
            let debit_blocks = current_epoch - account.last_debit_epoch;
            let debit_byte_block = debit_blocks as u64 * &account.capacity_used;
            let debit_credits = self.blob_credits_per_byte_block * debit_byte_block;
            self.credit_debited += &debit_credits;
            self.credit_committed -= &debit_credits;
            account.credit_committed -= &debit_credits;
            account.last_debit_epoch = current_epoch;
            debug!("debited {} credits from {}", debit_credits, address);
            writer.map.set(&address, account)?;
            Ok(())
        })?;
        self.accounts_root = writer.map.flush()?;
        Ok(delete_from_disc)
    }

    /// Add a blob.
    ///
    /// @param origin - The address that is submitting the transaction to add this blob.
    /// @param caller - The address that is calling into this function.
    //    If the blob actor is accessed directly, this will be the same as "origin".
    //    However, most of the time this will be the address of the actor instance that is
    //    calling into the blobs actor, i.e., a specific Bucket or Timehub instance.
    /// @param subscriber - The address responsible for the subscription to keep this blob around.
    ///   This is whose credits will be spent by this transaction, and going forward to continue to
    ///   pay for the blob over time. Generally, this is the owner of the wrapping Actor
    ///   (e.g., Buckets, Timehub).
    #[allow(clippy::too_many_arguments)]
    pub fn add_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        metadata_hash: Hash,
        id: SubscriptionId,
        size: u64,
        ttl: Option<ChainEpoch>,
        source: PublicKey,
        tokens_received: TokenAmount,
    ) -> anyhow::Result<(Subscription, TokenAmount), ActorError> {
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&subscriber, current_epoch)?;
        // Validate the TTL
        let (ttl, auto_renew) = accept_ttl(ttl, &account)?;
        // Get the credit delgation if needed
        let delegation = if origin != subscriber {
            // Look for an approval for origin from subscriber and validate the caller is allowed.
            let approval = account
                .approvals
                .get_mut(&origin.to_string())
                .and_then(|approval| approval.is_caller_allowed(&caller).then_some(approval))
                .ok_or(ActorError::forbidden(format!(
                    "approval from {} to {} via caller {} not found",
                    subscriber, origin, caller
                )))?;
            Some(CreditDelegation::new(origin, caller, approval))
        } else {
            None
        };
        // Capacity updates and required credit depend on whether the subscriber is already
        // subscribing to this blob
        let size = BigInt::from(size);
        let expiry = current_epoch + ttl;
        let mut new_capacity = BigInt::zero();
        let mut new_account_capacity = BigInt::zero();
        let credit_required: BigInt;
        // Like cashback but for sending unspent tokens back
        let tokens_unspent: TokenAmount;


        let blobs = &mut self.blobs_root.load(store)?;

        let (sub, blob) = if let Some(mut blob) = blobs.get(&hash)? {
            let sub = if let Some(group) = blob.subscribers.get_mut(&subscriber) {
                let (group_expiry, new_group_expiry) = group.max_expiries(&id, Some(expiry));
                // If the subscriber has been debited after the group's max expiry, we need to
                // clean up the accounting with a refund.
                // If the ensure-credit check below fails, the refund won't be saved in the
                // subscriber's state.
                // However, they will get rerefunded during the next auto debit tick.
                if let Some(group_expiry) = group_expiry {
                    if account.last_debit_epoch > group_expiry {
                        // The refund extends up to the current epoch because we need to
                        // account for the charge that will happen below at the current epoch.
                        let refund_blocks = current_epoch - group_expiry;
                        let refund_byte_blocks = refund_blocks as u64 * &size;
                        let refund_credits = self.blob_credits_per_byte_block * refund_byte_blocks;
                        // Re-mint spent credit
                        self.credit_debited -= &refund_credits;
                        self.credit_committed += &refund_credits;
                        account.credit_committed += &refund_credits;
                        debug!("refunded {} credits to {}", refund_credits, subscriber);
                    }
                }
                // Ensure subscriber has enough credits, considering the subscription group may
                // have expiries that cover a portion of the addition.
                // Required credit can be negative if subscriber is reducing expiry.
                // When adding, the new group expiry will always contain a value.
                let new_group_expiry = new_group_expiry.unwrap();
                let byte_blocks_required = if let Some(group_expiry) = group_expiry {
                    (new_group_expiry - group_expiry.max(current_epoch)) as u64 * &size
                } else {
                    (new_group_expiry - current_epoch) as u64 * &size
                };
                credit_required = byte_blocks_required * self.blob_credits_per_byte_block;
                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &tokens_received,
                    &subscriber,
                    current_epoch,
                    &delegation,
                )?;
                if let Some(sub) = group.subscriptions.get_mut(&id) {
                    // Update expiry index
                    if expiry != sub.expiry {
                        update_expiry_index(
                            &mut self.expiries,
                            subscriber,
                            hash,
                            &id,
                            Some((expiry, auto_renew)),
                            Some(sub.expiry),
                        );
                    }
                    sub.expiry = expiry;
                    sub.auto_renew = auto_renew;
                    // Overwrite source allows subscriber to retry resolving
                    sub.source = source;
                    sub.delegate = delegation.as_ref().map(|d| d.addresses());
                    sub.failed = false;
                    debug!(
                        "updated subscription to blob {} for {} (key: {})",
                        hash, subscriber, id
                    );
                    sub.clone()
                } else {
                    // Add new subscription
                    let sub = Subscription {
                        added: current_epoch,
                        expiry,
                        auto_renew,
                        source,
                        delegate: delegation.as_ref().map(|d| d.addresses()),
                        failed: false,
                    };
                    group.subscriptions.insert(id.clone(), sub.clone());
                    debug!(
                        "created new subscription to blob {} for {} (key: {})",
                        hash, subscriber, id
                    );
                    // Update expiry index
                    update_expiry_index(
                        &mut self.expiries,
                        subscriber,
                        hash,
                        &id,
                        Some((expiry, auto_renew)),
                        None,
                    );
                    sub
                }
            } else {
                new_account_capacity = size.clone();
                // One or more accounts have already committed credit.
                // However, we still need to reserve the full required credit from the new
                // subscriber, as the existing account(s) may decide to change the expiry or cancel.
                let byte_blocks_required = ttl as u64 * &size;
                credit_required = byte_blocks_required * self.blob_credits_per_byte_block;
                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &tokens_received,
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
                    failed: false,
                };
                blob.subscribers.insert(
                    subscriber,
                    SubscriptionGroup {
                        subscriptions: HashMap::from([(id.clone(), sub.clone())]),
                    },
                );
                debug!(
                    "created new subscription to blob {} for {} (key: {})",
                    hash, subscriber, id
                );
                // Update expiry index
                update_expiry_index(
                    &mut self.expiries,
                    subscriber,
                    hash,
                    &id,
                    Some((expiry, auto_renew)),
                    None,
                );
                sub
            };
            if !matches!(blob.status, BlobStatus::Resolved) {
                // It's pending or failed, reset to added status
                blob.status = BlobStatus::Added;
                // Add/update added with hash and its source
                self.added
                    .entry(hash)
                    .and_modify(|sources| {
                        sources.insert((subscriber, id.clone(), source));
                    })
                    .or_insert(HashSet::from([(subscriber, id, source)]));
            }
            (sub, blob)
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
            let byte_blocks_required = ttl as u64 * &size;
            credit_required = byte_blocks_required * self.blob_credits_per_byte_block;
            tokens_unspent = ensure_credit_or_buy(
                &mut account.credit_free,
                &mut self.credit_sold,
                &credit_required,
                &tokens_received,
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
                failed: false,
            };
            let blob = Blob {
                size: size.to_u64().unwrap(),
                metadata_hash,
                subscribers: HashMap::from([(
                    subscriber,
                    SubscriptionGroup {
                        subscriptions: HashMap::from([(id.clone(), sub.clone())]),
                    },
                )]),
                status: BlobStatus::Added,
            };
            debug!("created new blob {}", hash);
            debug!(
                "created new subscription to blob {} for {} (key: {})",
                hash, subscriber, id
            );
            // Update expiry index
            update_expiry_index(
                &mut self.expiries,
                subscriber,
                hash,
                &id,
                Some((expiry, auto_renew)),
                None,
            );
            // Add to added
            self.added
                .insert(hash, HashSet::from([(subscriber, id, source)]));
            (sub, blob)
        };
        // Account capacity is changing, debit for existing usage
        let debit_blocks = current_epoch - account.last_debit_epoch;
        let debit_byte_blocks = debit_blocks as u64 * &account.capacity_used;
        let debit = debit_byte_blocks * self.blob_credits_per_byte_block;
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
            delegation.approval.used += &credit_required;
        }
        // Save account
        self.accounts_root = accounts.set_and_flush(&subscriber, account)?;
        // Save blob
        self.blobs_root = blobs.set_and_flush(&hash, blob)?;

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

    fn renew_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        id: SubscriptionId,
    ) -> anyhow::Result<Account, ActorError> {
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&subscriber, current_epoch)?;
        // Get the blob
        let blobs = &mut self.blobs_root.load(store)?;
        let mut blob = blobs
            .get_or_err(&hash)?;
        if matches!(blob.status, BlobStatus::Failed) {
            // Do not renew failed blobs.
            return Err(ActorError::illegal_state(format!(
                "cannot renew failed blob {}",
                hash
            )));
        }
        let group = blob
            .subscribers
            .get_mut(&subscriber)
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, hash
            )))?;
        // Renewal must begin at the current epoch to avoid potential issues with auto-renewal.
        // Simply adding TTL to the current max expiry could result in an expiry date that's not
        // truly in the future, depending on how frequently auto-renewal occurs.
        // We'll ensure below that past unpaid for blocks are accounted for.
        let expiry = current_epoch + AUTO_TTL;
        let (group_expiry, new_group_expiry) = group.max_expiries(&id, Some(expiry));
        let sub = group
            .subscriptions
            .get_mut(&id)
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                id.clone()
            )))?;
        let delegation = if let Some((origin, caller)) = sub.delegate {
            // Look for an approval for origin from subscriber and validate the caller is allowed.
            let approval = account
                .approvals
                .get_mut(&origin.to_string())
                .and_then(|approval| approval.is_caller_allowed(&caller).then_some(approval))
                .ok_or(ActorError::forbidden(format!(
                    "approval from {} to {} via caller {} not found",
                    subscriber, origin, caller
                )))?;
            Some(CreditDelegation::new(origin, caller, approval))
        } else {
            None
        };
        // If the subscriber has been debited after the group's max expiry, we need to
        // clean up the accounting with a refund.
        // We could just account for the refund amount when ensuring credit below, but if that
        // fails, the overcharge would still exist.
        // When renewing, the existing group expiry will always contain a value.
        let group_expiry = group_expiry.unwrap();
        let size = BigInt::from(blob.size);
        if account.last_debit_epoch > group_expiry {
            // The refund extends up to the last debit epoch
            let refund_blocks = account.last_debit_epoch - group_expiry;
            let refund_byte_blocks = refund_blocks as u64 * &size;
            let refund_credits = refund_byte_blocks * self.blob_credits_per_byte_block;
            // Re-mint spent credit
            self.credit_debited -= &refund_credits;
            self.credit_committed += &refund_credits;
            account.credit_committed += &refund_credits;
            debug!("refunded {} credits to {}", refund_credits, subscriber);
        }
        // Ensure subscriber has enough credits, considering the subscription group may
        // have expiries that cover a portion of the renewal.
        // Required credit can be negative if subscriber is reducing expiry.
        // When renewing, the new group expiry will always contain a value.
        // There may be a gap between the existing expiry and the last debit that will make
        // the renewal discontinuous.
        let new_group_expiry = new_group_expiry.unwrap();
        let byte_blocks_required =
            (new_group_expiry - group_expiry.max(account.last_debit_epoch)) as u64 * &size;
        let credit_required = byte_blocks_required * self.blob_credits_per_byte_block;
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
                &id,
                Some((expiry, sub.auto_renew)),
                Some(sub.expiry),
            );
        }
        sub.expiry = expiry;
        debug!(
            "renewed subscription to blob {} for {} (key: {})",
            hash, subscriber, id
        );
        // Move free credit to committed credit
        self.credit_committed += &credit_required;
        account.credit_committed += &credit_required;
        account.credit_free -= &credit_required;
        // Update credit approval
        if let Some(delegation) = delegation {
            delegation.approval.used += &credit_required;
        }
        // Save account
        self.accounts_root = accounts.set_and_flush(&subscriber, account.clone())?;
        // Seve blobs
        self.blobs_root = blobs.set_and_flush(&hash, blob)?;

        debug!("committed {} credits from {}", credit_required, subscriber);
        Ok(account)
    }

    pub fn get_blob<BS: Blockstore>(&self,  store: &BS, hash: Hash) -> anyhow::Result<Option<Blob>, ActorError> {
        let blobs = self.blobs_root.load(store)?;
        blobs.get(&hash)
    }

    pub fn get_blob_status<BS: Blockstore>(
        &self,
        store: &BS,
        subscriber: Address,
        hash: Hash,
        id: SubscriptionId,
    ) -> Option<BlobStatus> {
        let blob = self.blobs_root.load(store).ok().and_then(|blobs| blobs.get(&hash).ok()).flatten();
        if blob.is_none() {
            return None;
        }
        let blob = blob.unwrap();
        if blob.subscribers.contains_key(&subscriber) {
            match blob.status {
                BlobStatus::Added => Some(BlobStatus::Added),
                BlobStatus::Pending => Some(BlobStatus::Pending),
                BlobStatus::Resolved => Some(BlobStatus::Resolved),
                BlobStatus::Failed => {
                    // The blob state's status may have been finalized as failed by another
                    // subscription.
                    // We need to see if this specific subscription failed.
                    if let Some(sub) = blob
                        .subscribers
                        .get(&subscriber)
                        .unwrap() // safe here
                        .subscriptions
                        .get(&id)
                    {
                        if sub.failed {
                            Some(BlobStatus::Failed)
                        } else {
                            Some(BlobStatus::Pending)
                        }
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn get_added_blobs(
        &self,
        size: u32,
    ) -> Vec<(Hash, HashSet<(Address, SubscriptionId, PublicKey)>)> {
        self.added
            .iter()
            .take(size as usize)
            .map(|element| (*element.0, element.1.clone()))
            .collect::<Vec<_>>()
    }

    #[allow(clippy::type_complexity)]
    pub fn get_pending_blobs(
        &self,
        size: u32,
    ) -> Vec<(Hash, HashSet<(Address, SubscriptionId, PublicKey)>)> {
        self.pending
            .iter()
            .take(size as usize)
            .map(|element| (*element.0, element.1.clone()))
            .collect::<Vec<_>>()
    }

    pub fn set_blob_pending<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        hash: Hash,
        id: SubscriptionId,
        source: PublicKey,
    ) -> anyhow::Result<(), ActorError> {
        let mut blobs = self.blobs_root.load(store)?;
        let mut blob = if let Some(blob) = blobs.get(&hash)? {
            blob
        } else {
            // The blob may have been deleted before it was set to pending
            return Ok(());
        };
        blob.status = BlobStatus::Pending;
        self.blobs_root = blobs.set_and_flush(&hash, blob)?;
        // Add to pending
        self.pending
            .insert(hash, HashSet::from([(subscriber, id, source)]));
        // Remove from added
        self.added.remove(&hash);
        Ok(())
    }

    pub fn finalize_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        id: SubscriptionId,
        status: BlobStatus,
    ) -> anyhow::Result<(), ActorError> {
        // Validate incoming status
        if matches!(status, BlobStatus::Added | BlobStatus::Pending) {
            return Err(ActorError::illegal_state(format!(
                "cannot finalize blob {} as added or pending",
                hash
            )));
        }
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&subscriber, current_epoch)?;
        // Get the blob
        let mut blobs = self.blobs_root.load(store)?;
        let mut blob = if let Some(blob) = blobs.get(&hash)? {
            blob
        } else {
            // The blob may have been deleted before it was finalized
            return Ok(());
        };
        if matches!(blob.status, BlobStatus::Added) {
            return Err(ActorError::illegal_state(format!(
                "blob {} cannot be finalized from status added",
                hash
            )));
        } else if matches!(blob.status, BlobStatus::Resolved) {
            // Blob is already finalized as resolved.
            // We can ignore later finalizations, even if they are failed.
            return Ok(());
        }
        let group = blob
            .subscribers
            .get_mut(&subscriber)
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, hash
            )))?;
        // Get max expiries with the current subscription removed in case we need them below.
        // We have to do this here to avoid breaking borrow rules.
        let (group_expiry, new_group_expiry) = group.max_expiries(&id, Some(0));
        let (sub_is_min_added, next_min_added) = group.is_min_added(&id)?;
        let sub = group
            .subscriptions
            .get_mut(&id)
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                id.clone()
            )))?;
        // Do not error if the approval was removed while this blob was pending
        let delegation = if let Some((origin, caller)) = sub.delegate {
            // Look for an approval for origin from subscriber and validate the caller is allowed.
            account
                .approvals
                .get_mut(&origin.to_string())
                .and_then(|approval| approval.is_caller_allowed(&caller).then_some(approval))
                .map(|approval| CreditDelegation::new(origin, caller, approval))
        } else {
            None
        };
        // Update blob status
        blob.status = status;
        debug!("finalized blob {} to status {}", hash, blob.status);
        if matches!(blob.status, BlobStatus::Failed) {
            let size = BigInt::from(blob.size);
            // We're not going to make a debit, but we need to refund any spent credits that may
            // have been used on this group in the event the last debit is later than the
            // added epoch.
            if account.last_debit_epoch > sub.added && sub_is_min_added {
                // The refund extends up to either the next minimum added epoch that is less
                // than the last debit epoch, or the last debit epoch.
                let refund_cutoff = next_min_added
                    .unwrap_or(account.last_debit_epoch)
                    .min(account.last_debit_epoch);
                let refund_blocks = refund_cutoff - sub.added;
                let refund_byte_blocks = refund_blocks as u64 * &size;
                let refund_credits = refund_byte_blocks * self.blob_credits_per_byte_block;
                // Re-mint spent credit
                self.credit_debited -= &refund_credits;
                account.credit_free += &refund_credits; // move directly to free
                debug!("refunded {} credits to {}", refund_credits, subscriber);
            }
            // If there's no new group expiry, all subscriptions have failed.
            if new_group_expiry.is_none() {
                // Account for reclaimed size and move committed credit to free credit
                self.capacity_used -= &size;
                debug!("released {} bytes to subnet", size);
                account.capacity_used -= &size;
                debug!("released {} bytes to {}", size, subscriber);
            }
            // Release credits considering other subscriptions may still be pending.
            // When failing, the existing group expiry will always contain a value.
            let group_expiry = group_expiry.unwrap();
            if account.last_debit_epoch < group_expiry {
                let reclaim_byte_blocks = if let Some(new_group_expiry) = new_group_expiry {
                    (group_expiry - new_group_expiry.max(account.last_debit_epoch)) * &size
                } else {
                    (group_expiry - account.last_debit_epoch) * &size
                };
                let reclaim_credits = reclaim_byte_blocks * self.blob_credits_per_byte_block;
                self.credit_committed -= &reclaim_credits;
                account.credit_committed -= &reclaim_credits;
                account.credit_free += &reclaim_credits;
                // Update credit approval
                if let Some(delegation) = delegation {
                    delegation.approval.used -= &reclaim_credits;
                }
                debug!("released {} credits to {}", reclaim_credits, subscriber);
            }
            sub.failed = true;
        }
        // Remove entry from pending
        if let Some(entry) = self.pending.get_mut(&hash) {
            entry.remove(&(subscriber, id, sub.source));
            if entry.is_empty() {
                self.pending.remove(&hash);
            }
        }
        // Save account
        self.accounts_root = accounts.set_and_flush(&subscriber, account)?;
        self.blobs_root = blobs.set_and_flush(&hash, blob)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn delete_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        id: SubscriptionId,
    ) -> anyhow::Result<bool, ActorError> {
        // Get or create a new account
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        let mut account = accounts.get_or_create(&subscriber, current_epoch)?;
        // Get the blob
        let mut blobs = self.blobs_root.load(store)?;

        let mut blob = if let Some(blob) = blobs.get(&hash)? {
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
        let num_subscribers = blob.subscribers.len();
        let group = blob
            .subscribers
            .get_mut(&subscriber)
            .ok_or(ActorError::forbidden(format!(
                "subscriber {} is not subscribed to blob {}",
                subscriber, hash
            )))?;
        let (group_expiry, new_group_expiry) = group.max_expiries(&id, Some(0));
        let sub = group
            .subscriptions
            .get(&id)
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                id.clone()
            )))?;
        let delegation = if let Some((origin, caller)) = sub.delegate {
            // Look for an approval for origin from subscriber and validate the caller is allowed.
            let approval = account
                .approvals
                .get_mut(&origin.to_string())
                .and_then(|approval| approval.is_caller_allowed(&caller).then_some(approval));
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
        // When deleting, the existing group expiry will always contain a value.
        let group_expiry = group_expiry.unwrap();
        let debit_epoch = group_expiry.min(current_epoch);
        // Account capacity is changing, debit for existing usage.
        // It could be possible that debit epoch is less than the last debit,
        // in which case we need to refund for that duration.
        if account.last_debit_epoch < debit_epoch {
            let debit_blocks = debit_epoch - account.last_debit_epoch;
            let debit_byte_blocks = debit_blocks as u64 * &account.capacity_used;
            let debit = debit_byte_blocks * self.blob_credits_per_byte_block;
            self.credit_debited += &debit;
            self.credit_committed -= &debit;
            account.credit_committed -= &debit;
            account.last_debit_epoch = debit_epoch;
            debug!("debited {} credits from {}", debit, subscriber);
        } else {
            // The account was debited after this blob's expiry
            let refund_blocks = account.last_debit_epoch - group_expiry;
            let refund_byte_blocks = refund_blocks as u64 * &BigInt::from(blob.size);
            let refund_credits = refund_byte_blocks * self.blob_credits_per_byte_block;
            // Re-mint spent credit
            self.credit_debited -= &refund_credits;
            self.credit_committed += &refund_credits;
            account.credit_committed += &refund_credits;
            debug!("refunded {} credits to {}", refund_credits, subscriber);
        }
        // Account for reclaimed size and move committed credit to free credit
        // If blob failed, capacity and committed credits have already been returned
        if !matches!(blob.status, BlobStatus::Failed) {
            let size = BigInt::from(blob.size);
            // If there's no new group expiry, we can reclaim capacity.
            if new_group_expiry.is_none() {
                account.capacity_used -= &size;
                if num_subscribers == 1 {
                    self.capacity_used -= &size;
                    debug!("released {} bytes to subnet", size);
                }
                debug!("released {} bytes to {}", size, subscriber);
            }
            // We can release credits if the new group expiry is in the future,
            // considering other subscriptions may still be active.
            if account.last_debit_epoch < group_expiry {
                let reclaim_byte_blocks = if let Some(new_group_expiry) = new_group_expiry {
                    (group_expiry - new_group_expiry.max(account.last_debit_epoch)) * &size
                } else {
                    (group_expiry - account.last_debit_epoch) * &size
                };
                let reclaim_credits = reclaim_byte_blocks * self.blob_credits_per_byte_block;
                self.credit_committed -= &reclaim_credits;
                account.credit_committed -= &reclaim_credits;
                account.credit_free += &reclaim_credits;
                // Update credit approval
                if let Some(delegation) = delegation {
                    delegation.approval.used -= &reclaim_credits;
                }
                debug!("released {} credits to {}", reclaim_credits, subscriber);
            }
        }
        // Update expiry index
        update_expiry_index(
            &mut self.expiries,
            subscriber,
            hash,
            &id,
            None,
            Some(sub.expiry),
        );
        // Remove entry from added
        if let Some(entry) = self.added.get_mut(&hash) {
            entry.remove(&(subscriber, id.clone(), sub.source));
            if entry.is_empty() {
                self.added.remove(&hash);
            }
        }
        // Remove entry from pending
        if let Some(entry) = self.pending.get_mut(&hash) {
            entry.remove(&(subscriber, id.clone(), sub.source));
            if entry.is_empty() {
                self.pending.remove(&hash);
            }
        }
        // Delete subscription
        group.subscriptions.remove(&id);
        debug!(
            "deleted subscription to blob {} for {} (key: {})",
            hash, subscriber, id
        );
        // Delete the group if empty
        let delete_blob = if group.subscriptions.is_empty() {
            blob.subscribers.remove(&subscriber);
            debug!("deleted subscriber {} to blob {}", subscriber, hash);
            // Delete or update blob
            let delete_blob = blob.subscribers.is_empty();
            if delete_blob {
                self.blobs_root = blobs.delete_and_flush(&hash)?;
                debug!("deleted blob {}", hash);
            }
            delete_blob
        } else {
            self.blobs_root = blobs.set_and_flush(&hash, blob)?;
            false
        };
        // Save account
        self.accounts_root = accounts.set_and_flush(&subscriber, account)?;
        Ok(delete_blob)
    }

    pub fn set_ttl_status<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        status: TtlStatus,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let mut accounts = Accounts::load(store, &self.accounts_root)?;
        match status {
            // We don't want to create an account for default TTL
            TtlStatus::Default => {
                if let Some(mut account) = accounts.get(&subscriber)? {
                    account.max_ttl_epochs = status.into();
                    self.accounts_root = accounts.set_and_flush(&subscriber, account)?;
                }
            }
            _ => {
                // Get or create a new account
                let mut account = accounts.get_or_create(&subscriber, current_epoch)?;
                account.max_ttl_epochs = status.into();
                self.accounts_root = accounts.set_and_flush(&subscriber, account)?;
            }
        }
        Ok(())
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
    credit_required: &BigInt,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    ensure_enough_credits(subscriber, credit_free, credit_required)?;
    ensure_delegated_credit(subscriber, current_epoch, credit_required, delegation)
}

/// Check if `subscriber` owns enough free credits.
fn ensure_enough_credits(
    subscriber: &Address,
    credit_free: &BigInt,
    credit_required: &BigInt,
) -> anyhow::Result<(), ActorError> {
    if credit_free >= credit_required {
        Ok(())
    } else {
        Err(ActorError::insufficient_funds(format!(
            "account {} has insufficient credit (available: {}; required: {})",
            subscriber, credit_free, credit_required
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn ensure_credit_or_buy(
    account_credit_free: &mut BigInt,
    state_credit_sold: &mut BigInt,
    credit_required: &BigInt,
    tokens_received: &TokenAmount,
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
                let tokens_needed = TokenAmount::from_atto(credits_needed.clone());
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
    credit_required: &BigInt,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    if let Some(delegation) = delegation {
        if let Some(limit) = &delegation.approval.limit {
            let unused = &(limit - &delegation.approval.used);
            if unused < credit_required {
                return Err(ActorError::insufficient_funds(format!(
                    "approval from {} to {} via caller {} has insufficient credit (available: {}; required: {})",
                    subscriber, delegation.origin, delegation.caller, unused, credit_required
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
    expiries: &mut BTreeMap<ChainEpoch, HashMap<Address, HashMap<ExpiryKey, bool>>>,
    subscriber: Address,
    hash: Hash,
    id: &SubscriptionId,
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
                        subs.insert(ExpiryKey::new(hash, id), auto_renew);
                    })
                    .or_insert(HashMap::from([(ExpiryKey::new(hash, id), auto_renew)]));
            })
            .or_insert(HashMap::from([(
                subscriber,
                HashMap::from([(ExpiryKey::new(hash, id), auto_renew)]),
            )]));
    }
    if let Some(remove) = remove {
        if let Some(entry) = expiries.get_mut(&remove) {
            if let Some(subs) = entry.get_mut(&subscriber) {
                subs.remove(&ExpiryKey::new(hash, id));
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

fn accept_ttl(
    ttl: Option<ChainEpoch>,
    account: &Account,
) -> anyhow::Result<(ChainEpoch, bool), ActorError> {
    let (ttl, auto_renew) = ttl.map(|ttl| (ttl, false)).unwrap_or((AUTO_TTL, true));
    if ttl < MIN_TTL {
        return Err(ActorError::illegal_argument(format!(
            "minimum blob TTL is {}",
            MIN_TTL
        )));
    }
    if ChainEpoch::from(account.max_ttl_epochs) < ttl {
        return Err(ActorError::forbidden(format!(
            "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
            ttl, account.max_ttl_epochs,
        )));
    }
    Ok((ttl, auto_renew))
}

#[cfg(test)]
mod tests {
    use super::*;

    use fvm_ipld_blockstore::MemoryBlockstore;

    use rand::RngCore;

    fn setup_logs() {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(tracing_subscriber::fmt::format().with_line_number(true))
                    .with_writer(std::io::stdout),
            )
            .with(EnvFilter::from_default_env())
            .try_init()
            .ok();
    }

    fn new_hash(size: usize) -> (Hash, u64) {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; size];
        rng.fill_bytes(&mut data);
        (
            Hash(*iroh_base::hash::Hash::new(&data).as_bytes()),
            size as u64,
        )
    }

    fn new_metadata_hash() -> Hash {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; 8];
        rng.fill_bytes(&mut data);
        Hash(*iroh_base::hash::Hash::new(&data).as_bytes())
    }

    pub fn new_pk() -> PublicKey {
        let mut rng = rand::thread_rng();
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);
        PublicKey(data)
    }

    // The state does not care about the address type.
    // We use an actor style (t/f2) addresses because they are straightforward to create.
    fn new_address() -> Address {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; 32];
        rng.fill_bytes(&mut data);
        Address::new_actor(&data)
    }

    fn check_approval(account: Account, origin: Address, caller: Address, expect_used: BigInt) {
        if !account.approvals.is_empty() {
            let approval = account.approvals.get(&origin.to_string()).unwrap();
            if origin != caller {
                assert!(approval.has_allowlist() && approval.is_caller_allowed(&caller));
            }
            assert_eq!(approval.used, expect_used);
        }
    }

    #[test]
    fn test_buy_credit_success() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let to = new_address();
        let amount = TokenAmount::from_whole(1);

        let res = state.buy_credit(&store, to, amount.clone(), 1);
        assert!(res.is_ok());
        let account = res.unwrap();
        let credit_sold = amount.atto().clone();
        assert_eq!(account.credit_free, credit_sold);
        assert_eq!(state.credit_sold, credit_sold);
        let account_back = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(account, account_back);
    }

    #[test]
    fn test_buy_credit_negative_amount() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(-1);

        let res = state.buy_credit(&store, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().msg(), "token amount must be positive");
    }

    #[test]
    fn test_buy_credit_at_capacity() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(1);

        state.capacity_used = BigInt::from(capacity);
        let res = state.buy_credit(&store, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            "credits not available (subnet has reached storage capacity)"
        );
    }

    #[test]
    fn test_approve_credit_success() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        // No limit or expiry
        let res = state.approve_credit(&store, from, to, None, current_epoch, None, None);
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.limit, None);
        assert_eq!(approval.expiry, None);

        // Add limit
        let limit = 1_000_000_000_000_000_000u64;
        let res = state.approve_credit(
            &store,
            from,
            to,
            None,
            current_epoch,
            Some(BigUint::from(limit)),
            None,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.limit, Some(BigInt::from(limit)));
        assert_eq!(approval.expiry, None);

        // Add ttl
        let ttl = ChainEpoch::from(MIN_TTL);
        let res = state.approve_credit(
            &store,
            from,
            to,
            None,
            current_epoch,
            Some(BigUint::from(limit)),
            Some(ttl),
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.limit, Some(BigInt::from(limit)));
        assert_eq!(approval.expiry, Some(ttl + current_epoch));

        // Require caller
        let require_caller = new_address();
        let res = state.approve_credit(
            &store,
            from,
            to,
            Some(HashSet::from([require_caller])),
            current_epoch,
            None,
            None,
        );
        assert!(res.is_ok());

        // Check the account approvals
        let account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(account.approvals.len(), 1);
        let approval = account.approvals.get(&to.to_string()).unwrap();
        assert!(approval.has_allowlist() && approval.is_caller_allowed(&require_caller));
    }

    #[test]
    fn test_approve_credit_invalid_ttl() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let ttl = ChainEpoch::from(MIN_TTL - 1);
        let res = state.approve_credit(&store, from, to, None, current_epoch, None, Some(ttl));
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("minimum approval TTL is {}", MIN_TTL)
        );
    }

    #[test]
    fn test_approve_credit_insufficient_credit() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, from, amount.clone(), current_epoch)
            .unwrap();
        let res = state.approve_credit(&store, from, to, None, current_epoch, None, None);
        assert!(res.is_ok());

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &store,
            to,
            to,
            from,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::Default,
            size,
            None,
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check approval
        let account = state.get_account(&store, from).unwrap().unwrap();
        let approval = account.approvals.get(&to.to_string()).unwrap();
        assert_eq!(account.credit_committed, approval.used);

        // Try to update approval with a limit below what's already been committed
        let limit = 1_000u64;
        let res = state.approve_credit(
            &store,
            from,
            to,
            None,
            current_epoch,
            Some(BigUint::from(limit)),
            None,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!(
                "limit cannot be less than amount of already used credits ({})",
                approval.used
            )
        );
    }

    #[test]
    fn test_revoke_credit_success() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let res = state.approve_credit(&store, from, to, None, current_epoch, None, None);
        assert!(res.is_ok());

        // Check the account approval
        let account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(account.approvals.len(), 1);
        let approval = account.approvals.get(&to.to_string()).unwrap();
        assert!(!approval.has_allowlist());

        // Update the approval with a required caller
        let require_caller = new_address();
        let res = state.approve_credit(
            &store,
            from,
            to,
            Some(HashSet::from([require_caller])),
            current_epoch,
            None,
            None,
        );
        assert!(res.is_ok());

        // Check the account approval
        let account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(account.approvals.len(), 1);
        let approval = account.approvals.get(&to.to_string()).unwrap();
        assert!(approval.has_allowlist() && approval.is_caller_allowed(&require_caller));

        // Remove the approval
        let res = state.revoke_credit(&store, from, to, None);
        assert!(res.is_ok());
        let account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(account.approvals.len(), 0);
    }

    #[test]
    fn test_revoke_credit_account_not_found() {
        setup_logs();
        let capacity = 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let from = new_address();
        let to = new_address();

        let res = state.revoke_credit(&store, from, to, None);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("account {} not found", from)
        );
    }

    #[test]
    fn test_debit_accounts_delete_from_disc() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        debit_accounts_delete_from_disc(
            &store,
            state,
            origin,
            origin,
            origin,
            current_epoch,
            token_amount,
        );
    }

    #[test]
    fn test_debit_accounts_delete_from_disc_with_approval() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, token_amount.clone(), current_epoch)
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        debit_accounts_delete_from_disc(
            &store,
            state,
            origin,
            origin,
            subscriber,
            current_epoch,
            token_amount,
        );
    }

    fn debit_accounts_delete_from_disc<BS: Blockstore>(
        store: &BS,
        mut state: State,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
    ) {
        let mut credit_amount = token_amount.atto().clone();

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::Default;
        let ttl1 = ChainEpoch::from(MIN_TTL);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add1_epoch,
            hash,
            new_metadata_hash(),
            id1.clone(),
            size,
            Some(ttl1),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Set to status pending
        let res = state.set_blob_pending(subscriber, hash, id1.clone(), source);
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            id1.clone(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(account.credit_committed, BigInt::from(ttl1 as u64 * size));
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size));

        // Add the same blob but this time uses a different subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let ttl2 = ChainEpoch::from(MIN_TTL);
        let id2 = SubscriptionId::Key(b"foo".to_vec());
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add2_epoch,
            hash,
            new_metadata_hash(),
            id2.clone(),
            size,
            Some(ttl2),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            BigInt::from(ttl2 as u64 * size),
        );
        credit_amount -= BigInt::from((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size)); // not changed

        // Check the subscription group
        let blob = state.get_blob(hash).unwrap();
        let group = blob.subscribers.get(&subscriber).unwrap();
        assert_eq!(group.subscriptions.len(), 2);

        // Debit all accounts at an epoch between the two expiries (3601-3621)
        let debit_epoch = ChainEpoch::from(MIN_TTL + 11);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            BigInt::from((ttl2 - (debit_epoch - add2_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, BigInt::from(size)); // not changed

        // Check the subscription group
        let blob = state.get_blob(hash).unwrap();
        let group = blob.subscribers.get(&subscriber).unwrap();
        assert_eq!(group.subscriptions.len(), 1); // the first subscription was deleted

        // Debit all accounts at an epoch greater than group expiry (3621)
        let debit_epoch = ChainEpoch::from(MIN_TTL + 31);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(!deletes_from_disc.is_empty()); // blob is marked for deletion

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // the second debit reduces this to zero
            BigInt::from(0),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, BigInt::from(0));

        // Check state
        assert_eq!(state.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(
            state.credit_debited,
            token_amount.atto() - &account.credit_free
        );
        assert_eq!(state.capacity_used, BigInt::from(0)); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(), 0);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        let account_committed = account.credit_committed.clone();
        check_approval(
            account,
            origin,
            caller,
            state.credit_debited + account_committed,
        );
    }

    #[test]
    fn test_add_blob_refund() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        add_blob_refund(
            &store,
            state,
            origin,
            origin,
            origin,
            current_epoch,
            token_amount,
        );
    }

    #[test]
    fn test_add_blob_refund_with_approval() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, token_amount.clone(), current_epoch)
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        add_blob_refund(
            &store,
            state,
            origin,
            origin,
            subscriber,
            current_epoch,
            token_amount,
        );
    }

    fn add_blob_refund<BS: Blockstore>(
        store: &BS,
        mut state: State,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
    ) {
        let mut credit_amount = token_amount.atto().clone();

        // Add blob with default a subscription ID
        let (hash1, size1) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::Default;
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add1_epoch,
            hash1,
            new_metadata_hash(),
            id1.clone(),
            size1,
            Some(MIN_TTL),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(MIN_TTL as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1));

        // Add another blob past the first blob's expiry
        let (hash2, size2) = new_hash(2048);
        let add2_epoch = ChainEpoch::from(MIN_TTL + 11);
        let id2 = SubscriptionId::Key(b"foo".to_vec());
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add2_epoch,
            hash2,
            new_metadata_hash(),
            id2.clone(),
            size2,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(MIN_TTL + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be refunded
            AUTO_TTL as u64 * size2 - overcharge,
        );
        credit_amount -= BigInt::from(AUTO_TTL as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1 + size2));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            token_amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Add the first (now expired) blob again
        let add3_epoch = ChainEpoch::from(MIN_TTL + 21);
        let id1 = SubscriptionId::Default;
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add3_epoch,
            hash1,
            new_metadata_hash(),
            id1.clone(),
            size1,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add3_epoch);
        assert_eq!(
            account.credit_committed, // should not include overcharge due to refund
            BigInt::from(
                (AUTO_TTL - (add3_epoch - add2_epoch)) as u64 * size2 + AUTO_TTL as u64 * size1
            ),
        );
        credit_amount -= BigInt::from(AUTO_TTL as u64 * size1);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1 + size2));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            token_amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        let account_committed = account.credit_committed.clone();
        check_approval(
            account,
            origin,
            caller,
            state.credit_debited + account_committed,
        );
    }

    #[test]
    fn test_add_blob_same_hash_same_account() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        add_blob_same_hash_same_account(
            &store,
            state,
            origin,
            origin,
            origin,
            current_epoch,
            token_amount,
        );
    }

    #[test]
    fn test_add_blob_same_hash_same_account_with_approval() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, token_amount.clone(), current_epoch)
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        add_blob_same_hash_same_account(
            &store,
            state,
            origin,
            origin,
            subscriber,
            current_epoch,
            token_amount,
        );
    }

    #[test]
    fn test_add_blob_same_hash_same_account_with_scoped_approval() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let caller = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, token_amount.clone(), current_epoch)
            .unwrap();
        state
            .approve_credit(
                &store,
                subscriber,
                origin,
                Some(HashSet::from([caller])),
                current_epoch,
                None,
                None,
            )
            .unwrap();
        add_blob_same_hash_same_account(
            &store,
            state,
            origin,
            caller,
            subscriber,
            current_epoch,
            token_amount,
        );
    }

    fn add_blob_same_hash_same_account<BS: Blockstore>(
        store: &BS,
        mut state: State,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
    ) {
        let mut credit_amount = token_amount.atto().clone();

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::Default;
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add1_epoch,
            hash,
            new_metadata_hash(),
            id1.clone(),
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add1_epoch);
        assert_eq!(sub.expiry, add1_epoch + AUTO_TTL);
        assert!(sub.auto_renew);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != origin {
            assert_eq!(sub.delegate, Some((origin, caller)));
        }

        // Check the blob status
        assert_eq!(
            state.get_blob_status(subscriber, hash, id1.clone()),
            Some(BlobStatus::Added)
        );

        // Check the blob
        let blob = state.get_blob(hash).unwrap();
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Added);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber).unwrap();
        assert_eq!(group.subscriptions.len(), 1);
        let got_sub = group.subscriptions.get(&id1.clone()).unwrap();
        assert_eq!(*got_sub, sub);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(AUTO_TTL as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size));

        // Set to status pending
        let res = state.set_blob_pending(subscriber, hash, id1.clone(), source);
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            id1.clone(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());
        assert_eq!(
            state.get_blob_status(subscriber, hash, id1.clone()),
            Some(BlobStatus::Resolved)
        );

        // Add the same blob again with a default subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add2_epoch,
            hash,
            new_metadata_hash(),
            id1.clone(),
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add1_epoch); // added should not change
        assert_eq!(sub.expiry, add2_epoch + AUTO_TTL);
        assert!(sub.auto_renew);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != origin {
            assert_eq!(sub.delegate, Some((origin, caller)));
        }

        // Check the blob status
        // Should already be resolved
        assert_eq!(
            state.get_blob_status(subscriber, hash, id1.clone()),
            Some(BlobStatus::Resolved)
        );

        // Check the blob
        let blob = state.get_blob(hash).unwrap();
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber).unwrap();
        assert_eq!(group.subscriptions.len(), 1); // Still only one subscription
        let got_sub = group.subscriptions.get(&id1.clone()).unwrap();
        assert_eq!(*got_sub, sub);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            BigInt::from(AUTO_TTL as u64 * size),
        );
        credit_amount -= BigInt::from((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size)); // not changed

        // Add the same blob again but use a different subscription ID
        let add3_epoch = ChainEpoch::from(31);
        let id2 = SubscriptionId::Key(b"foo".to_vec());
        let source = new_pk();
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add3_epoch,
            hash,
            new_metadata_hash(),
            id2.clone(),
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add3_epoch);
        assert_eq!(sub.expiry, add3_epoch + AUTO_TTL);
        assert!(sub.auto_renew);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != origin {
            assert_eq!(sub.delegate, Some((origin, caller)));
        }

        // Check the blob status
        // Should already be resolved
        assert_eq!(
            state.get_blob_status(subscriber, hash, id2.clone()),
            Some(BlobStatus::Resolved)
        );

        // Check the blob
        let blob = state.get_blob(hash).unwrap();
        assert_eq!(blob.subscribers.len(), 1); // still only one subscriber
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber).unwrap();
        assert_eq!(group.subscriptions.len(), 2);
        let got_sub = group.subscriptions.get(&id2.clone()).unwrap();
        assert_eq!(*got_sub, sub);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add3_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            BigInt::from(AUTO_TTL as u64 * size),
        );
        credit_amount -= BigInt::from((add3_epoch - add2_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size)); // not changed

        // Debit all accounts
        let debit_epoch = ChainEpoch::from(41);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            BigInt::from((AUTO_TTL - (debit_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, BigInt::from(size)); // not changed

        // Check indexes
        assert_eq!(state.expiries.len(), 2);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Delete the default subscription ID
        let delete_epoch = ChainEpoch::from(51);
        let res = state.delete_blob(
            &store,
            origin,
            caller,
            subscriber,
            delete_epoch,
            hash,
            id1.clone(),
        );
        assert!(res.is_ok());
        let delete_from_disk = res.unwrap();
        assert!(!delete_from_disk);

        // Check the blob
        let blob = state.get_blob(hash).unwrap();
        assert_eq!(blob.subscribers.len(), 1); // still one subscriber
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber).unwrap();
        assert_eq!(group.subscriptions.len(), 1);
        let sub = group.subscriptions.get(&id2.clone()).unwrap();
        assert_eq!(sub.added, add3_epoch);
        assert_eq!(sub.expiry, add3_epoch + AUTO_TTL);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, delete_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            BigInt::from((AUTO_TTL - (delete_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, BigInt::from(size)); // not changed

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            token_amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, BigInt::from(size));

        // Check indexes
        assert_eq!(state.expiries.len(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        let account_committed = account.credit_committed.clone();
        check_approval(
            account,
            origin,
            caller,
            state.credit_debited + account_committed,
        );
    }

    #[test]
    fn test_renew_blob_success() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, amount.clone(), current_epoch)
            .unwrap();
        let mut credit_amount = amount.atto().clone();

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add_epoch = current_epoch;
        let source = new_pk();
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::Default,
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(AUTO_TTL as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size));

        // Set to status pending
        let res = state.set_blob_pending(subscriber, hash, SubscriptionId::Default, source);
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::Default,
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());

        // Renew blob
        let renew_epoch = ChainEpoch::from(21);
        let res = state.renew_blob(
            &store,
            subscriber,
            renew_epoch,
            hash,
            SubscriptionId::Default,
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from((AUTO_TTL + (renew_epoch - add_epoch)) as u64 * size),
        );
        credit_amount -= (renew_epoch - add_epoch) as u64 * size;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_renew_blob_refund() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, amount.clone(), current_epoch)
            .unwrap();
        let mut credit_amount = amount.atto().clone();

        // Add blob with default a subscription ID
        let (hash1, size1) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::Default;
        let source = new_pk();
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            add1_epoch,
            hash1,
            new_metadata_hash(),
            id1.clone(),
            size1,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(AUTO_TTL as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1));

        // Add another blob past the first blob's expiry
        let (hash2, size2) = new_hash(2048);
        let add2_epoch = ChainEpoch::from(AUTO_TTL + 11);
        let id2 = SubscriptionId::Key(b"foo".to_vec());
        let source = new_pk();
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            add2_epoch,
            hash2,
            new_metadata_hash(),
            id2.clone(),
            size2,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(AUTO_TTL + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be accounted for
            AUTO_TTL as u64 * size2 - overcharge,
        );
        credit_amount -= BigInt::from(AUTO_TTL as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1 + size2));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Renew the first blob
        let renew_epoch = ChainEpoch::from(AUTO_TTL + 31);
        let res = state.renew_blob(&store, subscriber, renew_epoch, hash1, id1.clone());
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry2 = ChainEpoch::from(AUTO_TTL + renew_epoch);
        let blob2_expiry = ChainEpoch::from(AUTO_TTL + add2_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(
                (blob2_expiry - add2_epoch) as u64 * size2
                    + (blob1_expiry2 - add2_epoch) as u64 * size1
            ),
        );
        credit_amount -= BigInt::from((blob1_expiry2 - add2_epoch) as u64 * size1);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1 + size2));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_from_bad_state() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, amount.clone(), current_epoch)
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::Default,
            size,
            None,
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Finalize as pending
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::Default,
            BlobStatus::Pending,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("cannot finalize blob {} as added or pending", hash)
        );
    }

    #[test]
    fn test_finalize_blob_resolved() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, amount.clone(), current_epoch)
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::Default,
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Set to status pending
        let res = state.set_blob_pending(subscriber, hash, SubscriptionId::Default, source);
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::Default,
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(subscriber, hash, SubscriptionId::Default)
            .unwrap();
        assert!(matches!(status, BlobStatus::Resolved));

        // Check indexes
        assert_eq!(state.expiries.len(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, amount.clone(), current_epoch)
            .unwrap();
        let credit_amount = amount.atto().clone();

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::Default,
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Set to status pending
        let res = state.set_blob_pending(subscriber, hash, SubscriptionId::Default, source);
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::Default,
            BlobStatus::Failed,
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(subscriber, hash, SubscriptionId::Default)
            .unwrap();
        assert!(matches!(status, BlobStatus::Failed));

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(account.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(0)); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(state.credit_debited, BigInt::from(0));
        assert_eq!(state.capacity_used, BigInt::from(0)); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed_refund() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, amount.clone(), current_epoch)
            .unwrap();
        let mut credit_amount = amount.atto().clone();

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &store,
            subscriber,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::Default,
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(AUTO_TTL as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(state.credit_debited, BigInt::from(0));
        assert_eq!(state.capacity_used, account.capacity_used); // capacity was released

        // Debit accounts to trigger a refund when we fail below
        let debit_epoch = ChainEpoch::from(11);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from((AUTO_TTL - (debit_epoch - add_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, BigInt::from(size));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            BigInt::from((debit_epoch - add_epoch) as u64 * size)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Set to status pending
        let res = state.set_blob_pending(subscriber, hash, SubscriptionId::Default, source);
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(21);
        let res = state.finalize_blob(
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::Default,
            BlobStatus::Failed,
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(subscriber, hash, SubscriptionId::Default)
            .unwrap();
        assert!(matches!(status, BlobStatus::Failed));

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(account.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(account.credit_free, amount.atto().clone()); // credit was refunded
        assert_eq!(account.capacity_used, BigInt::from(0)); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(state.credit_debited, BigInt::from(0)); // credit was refunded and released
        assert_eq!(state.capacity_used, BigInt::from(0)); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_delete_blob_refund() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        delete_blob_refund(
            &store,
            state,
            origin,
            origin,
            origin,
            current_epoch,
            token_amount,
        );
    }

    #[test]
    fn test_delete_blob_refund_with_approval() {
        setup_logs();
        let capacity = 1024 * 1024;
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, capacity, 1).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&store, subscriber, token_amount.clone(), current_epoch)
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        delete_blob_refund(
            &store,
            state,
            origin,
            origin,
            subscriber,
            current_epoch,
            token_amount,
        );
    }

    #[test]
    fn test_if_blobs_ttl_exceeds_accounts_ttl_should_error() {
        setup_logs();

        // Test cases structure
        struct TestCase {
            name: &'static str,
            account_ttl_status: TtlStatus,
            blob_ttl: ChainEpoch,
            should_succeed: bool,
        }

        // Define test cases
        let test_cases = vec![
            TestCase {
                name: "Reduced status rejects even minimum TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: MIN_TTL,
                should_succeed: false,
            },
            TestCase {
                name: "Default status allows default TTL",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: TtlStatus::DEFAULT_MAX_TTL,
                should_succeed: true,
            },
            TestCase {
                name: "Default status rejects higher TTL",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: TtlStatus::DEFAULT_MAX_TTL + 1,
                should_succeed: false,
            },
            TestCase {
                name: "Custom status allows matching TTL",
                account_ttl_status: TtlStatus::Custom(7200),
                blob_ttl: 7200,
                should_succeed: true,
            },
            TestCase {
                name: "Custom status rejects higher TTL",
                account_ttl_status: TtlStatus::Custom(7200),
                blob_ttl: 7201,
                should_succeed: false,
            },
            TestCase {
                name: "Extended status allows any TTL",
                account_ttl_status: TtlStatus::Extended,
                blob_ttl: 365 * 24 * 60 * 60, // 1 year
                should_succeed: true,
            },
        ];

        // Run all test cases
        for tc in test_cases {
            let capacity = 1024 * 1024;
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store, capacity, 1).unwrap();
            let subscriber = new_address();
            let current_epoch = ChainEpoch::from(1);
            let amount = TokenAmount::from_whole(10);

            state
                .buy_credit(&store, subscriber, amount.clone(), current_epoch)
                .unwrap();
            state
                .set_ttl_status(&store, subscriber, tc.account_ttl_status, current_epoch)
                .unwrap();

            let (hash, size) = new_hash(1024);
            let res = state.add_blob(
                &store,
                subscriber,
                subscriber,
                subscriber,
                current_epoch,
                hash,
                new_metadata_hash(),
                SubscriptionId::Default,
                size,
                Some(tc.blob_ttl),
                new_pk(),
                TokenAmount::zero(),
            );

            if tc.should_succeed {
                assert!(
                    res.is_ok(),
                    "Test case '{}' should succeed but failed: {:?}",
                    tc.name,
                    res.err()
                );
            } else {
                assert!(
                    res.is_err(),
                    "Test case '{}' should fail but succeeded",
                    tc.name
                );
                assert_eq!(
                    res.err().unwrap().msg(),
                    format!(
                        "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
                        tc.blob_ttl, i64::from(tc.account_ttl_status),
                    ),
                    "Test case '{}' failed with unexpected error message",
                    tc.name
                );
            }
        }
    }

    fn delete_blob_refund<BS: Blockstore>(
        store: &BS,
        mut state: State,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
    ) {
        let mut credit_amount = token_amount.atto().clone();

        // Add a blob
        let add1_epoch = current_epoch;
        let (hash1, size1) = new_hash(1024);
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add1_epoch,
            hash1,
            new_metadata_hash(),
            SubscriptionId::Default,
            size1,
            Some(MIN_TTL),
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(MIN_TTL as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1));

        // Add another blob past the first blob expiry
        // This will trigger a debit on the account
        let add2_epoch = ChainEpoch::from(MIN_TTL + 10);
        let (hash2, size2) = new_hash(2048);
        let res = state.add_blob(
            &store,
            origin,
            caller,
            subscriber,
            add2_epoch,
            hash2,
            new_metadata_hash(),
            SubscriptionId::Default,
            size2,
            Some(MIN_TTL),
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(MIN_TTL + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be refunded
            MIN_TTL as u64 * size2 - overcharge,
        );
        credit_amount -= BigInt::from(MIN_TTL as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, BigInt::from(size1 + size2));

        // Delete the first blob
        let delete_epoch = ChainEpoch::from(MIN_TTL + 20);
        let delete_from_disc = state
            .delete_blob(
                &store,
                origin,
                caller,
                subscriber,
                delete_epoch,
                hash1,
                SubscriptionId::Default,
            )
            .unwrap();
        assert!(delete_from_disc);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch); // not changed, blob is expired
        assert_eq!(
            account.credit_committed, // should not include overcharge due to refund
            BigInt::from(MIN_TTL as u64 * size2),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, BigInt::from(size2));

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed); // credit was released
        assert_eq!(state.credit_debited, BigInt::from(MIN_TTL as u64 * size1));
        assert_eq!(state.capacity_used, BigInt::from(size2)); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(), 1);
        assert_eq!(state.added.len(), 1);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        let account_committed = account.credit_committed.clone();
        check_approval(
            account,
            origin,
            caller,
            state.credit_debited + account_committed,
        );
    }
}
