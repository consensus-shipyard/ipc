// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Bound::{Included, Unbounded};

use fendermint_actor_blobs_shared::params::GetStatsReturn;
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, CreditAllowance, CreditApproval, Hash, PublicKey, Subscription,
    SubscriptionGroup, SubscriptionId, TtlStatus,
};
use fendermint_actor_hoku_config_shared::HokuConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::{BigInt, BigUint};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use log::{debug, warn};
use num_traits::{Signed, ToPrimitive, Zero};

use crate::state_fields::{AccountsState, BlobsProgressCollection, BlobsState};

/// The minimum epoch duration a blob can be stored.
const MIN_TTL: ChainEpoch = 3600; // one hour
/// The rolling epoch duration used for non-expiring blobs.
const AUTO_TTL: ChainEpoch = 3600; // one hour

/// The state represents all accounts and stored blobs.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total used storage capacity of the subnet.
    pub capacity_used: u64,
    /// The total number of credits sold in the subnet.
    pub credit_sold: BigInt,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: BigInt,
    /// The total number of credits debited in the subnet.
    pub credit_debited: BigInt,
    /// Map of expiries to blob hashes.
    pub expiries: BTreeMap<ChainEpoch, HashMap<Address, HashMap<ExpiryKey, bool>>>,
    /// Map of currently added blob hashes to account and source Iroh node IDs.
    pub added: BlobsProgressCollection,
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: BlobsProgressCollection,
    /// HAMT containing all accounts keyed by robust (non-ID) actor address.
    pub accounts: AccountsState,
    /// HAMT containing all blobs keyed by blob hash.
    pub blobs: BlobsState,
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
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self, ActorError> {
        Ok(Self {
            capacity_used: 0,
            credit_sold: BigInt::zero(),
            credit_committed: BigInt::zero(),
            credit_debited: BigInt::zero(),
            expiries: BTreeMap::new(),
            added: BlobsProgressCollection::new(),
            pending: BlobsProgressCollection::new(),
            accounts: AccountsState::new(store)?,
            blobs: BlobsState::new(store)?,
        })
    }

    pub fn get_stats(&self, balance: TokenAmount, hoku_config: &HokuConfig) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(hoku_config.blob_capacity),
            capacity_used: self.capacity_used.clone(),
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            blob_credits_per_byte_block: hoku_config.blob_credits_per_byte_block,
            num_accounts: self.accounts.len(),
            num_blobs: self.blobs.len(),
            num_resolving: self.pending.len(),
            bytes_resolving: self.pending.bytes_size(),
            num_added: self.added.len(),
            bytes_added: self.added.bytes_size(),
        }
    }

    pub fn buy_credit<BS: Blockstore>(
        &mut self,
        hoku_config: &HokuConfig,
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
        if self.capacity_available(hoku_config.blob_capacity).is_zero() {
            return Err(ActorError::forbidden(
                "credits not available (subnet has reached storage capacity)".into(),
            ));
        }
        self.credit_sold += &credits;
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&to, || Account::new(current_epoch))?;
        account.credit_free += &credits;
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&to, account.clone())?);

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
        let mut accounts = self.accounts.hamt(store)?;
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
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&addr, account)?);

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
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&from, || Account::new(current_epoch))?;
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
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&from, account)?);

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
        let mut accounts = self.accounts.hamt(store)?;
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
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&from, account)?);

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
        let accounts = self.accounts.hamt(store)?;
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
        let accounts = self.accounts.hamt(store)?;
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
        let accounts = self.accounts.hamt(store)?;
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
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&from, || Account::new(current_epoch))?;
        account.credit_sponsor = sponsor;
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&from, account)?);

        debug!("set credit sponsor for {} to {:?}", from, sponsor);
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn debit_accounts<BS: Blockstore>(
        &mut self,
        hoku_config: &HokuConfig,
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
                            hoku_config,
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
                        hoku_config,
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
        let reader = self.accounts.hamt(store)?;
        let mut writer = self.accounts.hamt(store)?;
        reader.for_each(|address, account| {
            let mut account = account.clone();
            let debit_blocks = current_epoch - account.last_debit_epoch;
            let debit_credits = self.get_storage_cost(
                hoku_config.blob_credits_per_byte_block,
                debit_blocks,
                &account.capacity_used,
            );
            self.credit_debited += &debit_credits;
            self.credit_committed -= &debit_credits;
            account.credit_committed -= &debit_credits;
            account.last_debit_epoch = current_epoch;
            debug!("debited {} credits from {}", debit_credits, address);
            writer.set(&address, account)?;
            Ok(())
        })?;
        self.accounts.root = writer.flush()?;
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
        hoku_config: &HokuConfig,
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
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&subscriber, || Account::new(current_epoch))?;
        // Validate the TTL
        let (ttl, auto_renew) = accept_ttl(ttl, &account)?;
        // Get the credit delegation if needed
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
        let expiry = current_epoch + ttl;
        let mut new_capacity: u64 = 0;
        let mut new_account_capacity: u64 = 0;
        let credit_required: BigInt;
        // Like cashback but for sending unspent tokens back
        let tokens_unspent: TokenAmount;
        // Get or create a new blob
        let mut blobs = self.blobs.hamt(store)?;
        let (sub, blob) = if let Some(mut blob) = blobs.get(&hash)? {
            let sub = if let Some(group) = blob.subscribers.get_mut(&subscriber.to_string()) {
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
                        let refund_credits = self.get_storage_cost(
                            hoku_config.blob_credits_per_byte_block,
                            refund_blocks,
                            &size,
                        );
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
                let group_expiry = group_expiry.map_or(current_epoch, |e| e.max(current_epoch));
                credit_required = self.get_storage_cost(
                    hoku_config.blob_credits_per_byte_block,
                    new_group_expiry - group_expiry,
                    &size,
                );
                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &tokens_received,
                    &subscriber,
                    current_epoch,
                    &delegation,
                )?;
                if let Some(sub) = group.subscriptions.get_mut(&id.to_string()) {
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
                    group
                        .subscriptions
                        .insert(id.clone().to_string(), sub.clone());
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
                new_account_capacity = size;
                // One or more accounts have already committed credit.
                // However, we still need to reserve the full required credit from the new
                // subscriber, as the existing account(s) may decide to change the expiry or cancel.
                credit_required =
                    self.get_storage_cost(hoku_config.blob_credits_per_byte_block, ttl, &size);
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
                    subscriber.to_string(),
                    SubscriptionGroup {
                        subscriptions: HashMap::from([(id.clone().to_string(), sub.clone())]),
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
                self.added.upsert(hash, subscriber, id, source, blob.size);
            }
            (sub, blob)
        } else {
            new_account_capacity = size;
            // New blob increases network capacity as well.
            // Ensure there is enough capacity available.
            let available_capacity = self.capacity_available(hoku_config.blob_capacity);
            if size > available_capacity {
                return Err(ActorError::forbidden(format!(
                    "subnet has insufficient storage capacity (available: {}; required: {})",
                    available_capacity, size
                )));
            }
            new_capacity = size.clone();
            credit_required =
                self.get_storage_cost(hoku_config.blob_credits_per_byte_block, ttl, &size);
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
                    subscriber.to_string(),
                    SubscriptionGroup {
                        subscriptions: HashMap::from([(id.clone().to_string(), sub.clone())]),
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
                .insert(hash, HashSet::from([(subscriber, id, source)]), blob.size);
            (sub, blob)
        };
        // Account capacity is changing, debit for existing usage
        let debit = self.get_storage_cost(
            hoku_config.blob_credits_per_byte_block,
            current_epoch - account.last_debit_epoch,
            &account.capacity_used,
        );
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
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);

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

    fn get_storage_cost(&self, blob_credits_per_byte_block: u64, ttl: i64, size: &u64) -> BigInt {
        let byte_blocks_required = ttl * BigInt::from(size.clone());
        byte_blocks_required * blob_credits_per_byte_block
    }

    fn renew_blob<BS: Blockstore>(
        &mut self,
        hoku_config: &HokuConfig,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        id: SubscriptionId,
    ) -> anyhow::Result<Account, ActorError> {
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&subscriber, || Account::new(current_epoch))?;
        // Get the blob
        let mut blobs = self.blobs.hamt(store)?;
        let mut blob = blobs.get_or_err(&hash)?;
        if matches!(blob.status, BlobStatus::Failed) {
            // Do not renew failed blobs.
            return Err(ActorError::illegal_state(format!(
                "cannot renew failed blob {}",
                hash
            )));
        }
        let group =
            blob.subscribers
                .get_mut(&subscriber.to_string())
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
            .get_mut(&id.to_string())
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
        let size = blob.size;
        if account.last_debit_epoch > group_expiry {
            // The refund extends up to the last debit epoch
            let refund_credits = self.get_storage_cost(
                hoku_config.blob_credits_per_byte_block,
                account.last_debit_epoch - group_expiry,
                &size,
            );
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
        let credit_required = self.get_storage_cost(
            hoku_config.blob_credits_per_byte_block,
            new_group_expiry - group_expiry.max(account.last_debit_epoch),
            &size,
        );
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
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&subscriber, account.clone())?);
        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);

        debug!("committed {} credits from {}", credit_required, subscriber);
        Ok(account)
    }

    pub fn get_blob<BS: Blockstore>(
        &self,
        store: &BS,
        hash: Hash,
    ) -> anyhow::Result<Option<Blob>, ActorError> {
        let blobs = self.blobs.hamt(store)?;
        blobs.get(&hash)
    }

    pub fn get_blob_status<BS: Blockstore>(
        &self,
        store: &BS,
        subscriber: Address,
        hash: Hash,
        id: SubscriptionId,
    ) -> Option<BlobStatus> {
        let blob = self
            .blobs
            .hamt(store)
            .ok()
            .and_then(|blobs| blobs.get(&hash).ok())
            .flatten()?;
        if blob.subscribers.contains_key(&subscriber.to_string()) {
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
                        .get(&subscriber.to_string())
                        .unwrap() // safe here
                        .subscriptions
                        .get(&id.to_string())
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
        self.added.take_page(size)
    }

    #[allow(clippy::type_complexity)]
    pub fn get_pending_blobs(
        &self,
        size: u32,
    ) -> Vec<(Hash, HashSet<(Address, SubscriptionId, PublicKey)>)> {
        self.pending.take_page(size)
    }

    pub fn set_blob_pending<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        hash: Hash,
        id: SubscriptionId,
        source: PublicKey,
    ) -> anyhow::Result<(), ActorError> {
        let mut blobs = self.blobs.hamt(store)?;
        let mut blob = if let Some(blob) = blobs.get(&hash)? {
            blob
        } else {
            // The blob may have been deleted before it was set to pending
            return Ok(());
        };
        blob.status = BlobStatus::Pending;
        // Add to pending
        self.pending
            .insert(hash, HashSet::from([(subscriber, id, source)]), blob.size);
        // Remove from added
        self.added.remove(&hash, blob.size);
        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
        Ok(())
    }

    pub fn finalize_blob<BS: Blockstore>(
        &mut self,
        hoku_config: &HokuConfig,
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
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&subscriber, || Account::new(current_epoch))?;
        // Get the blob
        let mut blobs = self.blobs.hamt(store)?;
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
        let group =
            blob.subscribers
                .get_mut(&subscriber.to_string())
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
            .get_mut(&id.to_string())
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
            let size = blob.size;
            // We're not going to make a debit, but we need to refund any spent credits that may
            // have been used on this group in the event the last debit is later than the
            // added epoch.
            if account.last_debit_epoch > sub.added && sub_is_min_added {
                // The refund extends up to either the next minimum added epoch that is less
                // than the last debit epoch, or the last debit epoch.
                let refund_cutoff = next_min_added
                    .unwrap_or(account.last_debit_epoch)
                    .min(account.last_debit_epoch);
                let refund_credits = self.get_storage_cost(
                    hoku_config.blob_credits_per_byte_block,
                    refund_cutoff - sub.added,
                    &size,
                );
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
                account.capacity_used -= size;
                debug!("released {} bytes to {}", size, subscriber);
            }
            // Release credits considering other subscriptions may still be pending.
            // When failing, the existing group expiry will always contain a value.
            let group_expiry = group_expiry.unwrap();
            if account.last_debit_epoch < group_expiry {
                let reclaim_credits = self.get_storage_cost(
                    hoku_config.blob_credits_per_byte_block,
                    group_expiry
                        - new_group_expiry.map_or(account.last_debit_epoch, |e| {
                            e.max(account.last_debit_epoch)
                        }),
                    &size,
                );
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
        self.pending
            .remove_entry(hash, subscriber, id, sub.source, blob.size);
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn delete_blob<BS: Blockstore>(
        &mut self,
        hoku_config: &HokuConfig,
        store: &BS,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        id: SubscriptionId,
    ) -> anyhow::Result<bool, ActorError> {
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&subscriber, || Account::new(current_epoch))?;
        // Get the blob
        let mut blobs = self.blobs.hamt(store)?;
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
        let group =
            blob.subscribers
                .get_mut(&subscriber.to_string())
                .ok_or(ActorError::forbidden(format!(
                    "subscriber {} is not subscribed to blob {}",
                    subscriber, hash
                )))?;
        let (group_expiry, new_group_expiry) = group.max_expiries(&id, Some(0));
        let sub = group
            .subscriptions
            .get(&id.to_string())
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
            let debit = self.get_storage_cost(
                hoku_config.blob_credits_per_byte_block,
                debit_epoch - account.last_debit_epoch,
                &account.capacity_used,
            );
            self.credit_debited += &debit;
            self.credit_committed -= &debit;
            account.credit_committed -= &debit;
            account.last_debit_epoch = debit_epoch;
            debug!("debited {} credits from {}", debit, subscriber);
        } else if account.last_debit_epoch != debit_epoch {
            // The account was debited after this blob's expiry
            let refund_credits = self.get_storage_cost(
                hoku_config.blob_credits_per_byte_block,
                account.last_debit_epoch - group_expiry,
                &blob.size,
            );
            // Re-mint spent credit
            self.credit_debited -= &refund_credits;
            self.credit_committed += &refund_credits;
            account.credit_committed += &refund_credits;
            debug!("refunded {} credits to {}", refund_credits, subscriber);
        }
        // Account for reclaimed size and move committed credit to free credit
        // If blob failed, capacity and committed credits have already been returned
        if !matches!(blob.status, BlobStatus::Failed) {
            let size = blob.size;
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
                let reclaim_credits = self.get_storage_cost(
                    hoku_config.blob_credits_per_byte_block,
                    group_expiry
                        - new_group_expiry.map_or(account.last_debit_epoch, |e| {
                            e.max(account.last_debit_epoch)
                        }),
                    &blob.size,
                );
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
        self.added
            .remove_entry(hash, subscriber, id.clone(), sub.source, blob.size);
        // Remove entry from pending
        self.pending
            .remove_entry(hash, subscriber, id.clone(), sub.source, blob.size);
        // Delete subscription
        group.subscriptions.remove(&id.to_string());
        debug!(
            "deleted subscription to blob {} for {} (key: {})",
            hash, subscriber, id
        );
        // Delete the group if empty
        let delete_blob = if group.subscriptions.is_empty() {
            blob.subscribers.remove(&subscriber.to_string());
            debug!("deleted subscriber {} to blob {}", subscriber, hash);
            // Delete or update blob
            let delete_blob = blob.subscribers.is_empty();
            if delete_blob {
                self.blobs
                    .save_tracked(blobs.delete_and_flush_tracked(&hash)?);
                debug!("deleted blob {}", hash);
            }
            delete_blob
        } else {
            self.blobs
                .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
            false
        };
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
        Ok(delete_blob)
    }

    pub fn set_ttl_status<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        status: TtlStatus,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        match status {
            // We don't want to create an account for default TTL
            TtlStatus::Default => {
                if let Some(mut account) = accounts.get(&subscriber)? {
                    account.max_ttl = status.into();
                    self.accounts
                        .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
                }
            }
            _ => {
                // Get or create a new account
                let mut account =
                    accounts.get_or_create(&subscriber, || Account::new(current_epoch))?;
                account.max_ttl = status.into();
                self.accounts
                    .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
            }
        }
        Ok(())
    }

    /// Return available capacity as a difference between `blob_capacity_total` and `capacity_used`.
    fn capacity_available(&self, blob_capacity_total: u64) -> u64 {
        blob_capacity_total - &self.capacity_used
    }

    /// Adjusts all subscriptions for `account` according to its max TTL.
    /// Returns the number of subscriptions processed and the next key to continue iteration.
    /// If `starting_hash` is `None`, iteration starts from the beginning.
    /// If `limit` is `None`, all subscriptions are processed.
    /// If `limit` is not `None`, iteration stops after examining `limit` blobs.
    pub fn adjust_blob_ttls_for_account<BS: Blockstore>(
        &mut self,
        hoku_config: &HokuConfig,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        starting_hash: Option<Hash>,
        limit: Option<usize>,
    ) -> anyhow::Result<(u32, Option<Hash>, Vec<Hash>), ActorError> {
        use hoku_ipld::hamt::BytesKey;

        let new_ttl = self.get_account_max_ttl(store, subscriber)?;

        let mut deleted_blobs = Vec::new();

        let mut processed = 0;
        let blobs = self.blobs.hamt(store)?;
        let starting_key = starting_hash.map(|h| BytesKey::from(h.0.as_slice()));
        let (_, next_key) = blobs.for_each_ranged(
            starting_key.as_ref(),
            limit,
            |hash, blob| -> Result<(), ActorError> {
                if let Some(group) = blob.subscribers.get(&subscriber.to_string()) {
                    for (id, sub) in &group.subscriptions {
                        if sub.expiry - sub.added > new_ttl {
                            if new_ttl == 0 {
                                // Delete subscription
                                if self.delete_blob(
                                    hoku_config,
                                    store,
                                    subscriber,
                                    subscriber,
                                    subscriber,
                                    current_epoch,
                                    hash,
                                    SubscriptionId::new(&id.clone())?,
                                )? {
                                    deleted_blobs.push(hash);
                                };
                            } else {
                                self.add_blob(
                                    hoku_config,
                                    store,
                                    subscriber,
                                    subscriber,
                                    subscriber,
                                    current_epoch,
                                    hash,
                                    blob.metadata_hash.clone(),
                                    SubscriptionId::new(&id.clone())?,
                                    blob.size,
                                    Some(new_ttl),
                                    sub.source,
                                    TokenAmount::zero(),
                                )?;
                            }

                            processed += 1;
                        } else if sub.expiry - sub.added < TtlStatus::DEFAULT_MAX_TTL
                            && sub.auto_renew
                            && new_ttl != ChainEpoch::MAX
                        {
                            // if extended user added a blob with no TTL (i.e. with auto renew) and
                            // then switched to default account, we need to set the TTL to the default
                            // max TTL with no auto renew
                            self.add_blob(
                                hoku_config,
                                store,
                                subscriber,
                                subscriber,
                                subscriber,
                                current_epoch,
                                hash,
                                blob.metadata_hash.clone(),
                                SubscriptionId::new(&id.clone())?,
                                blob.size,
                                Some(TtlStatus::DEFAULT_MAX_TTL),
                                sub.source,
                                TokenAmount::zero(),
                            )?;
                            processed += 1;
                        }
                    }
                }
                Ok(())
            },
        )?;

        Ok((processed, next_key, deleted_blobs))
    }

    pub fn get_account_max_ttl<BS: Blockstore>(
        &self,
        store: &BS,
        account: Address,
    ) -> Result<ChainEpoch, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        Ok(accounts
            .get(&account)?
            .map_or(TtlStatus::DEFAULT_MAX_TTL, |account| account.max_ttl))
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

    if ChainEpoch::from(account.max_ttl) < ttl {
        return Err(ActorError::forbidden(format!(
            "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
            ttl, account.max_ttl,
        )));
    }
    if account.max_ttl == TtlStatus::DEFAULT_MAX_TTL {
        Ok((
            if auto_renew {
                TtlStatus::DEFAULT_MAX_TTL
            } else {
                ttl
            },
            false,
        ))
    } else {
        Ok((ttl, auto_renew))
    }
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let to = new_address();
        let amount = TokenAmount::from_whole(1);

        let res = state.buy_credit(&hoku_config, &store, to, amount.clone(), 1);
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(-1);

        let res = state.buy_credit(&hoku_config, &store, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().msg(), "token amount must be positive");
    }

    #[test]
    fn test_buy_credit_at_capacity() {
        setup_logs();
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(1);

        state.capacity_used = hoku_config.blob_capacity;
        let res = state.buy_credit(&hoku_config, &store, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            "credits not available (subnet has reached storage capacity)"
        );
    }

    #[test]
    fn test_approve_credit_success() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
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
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&hoku_config, &store, from, amount.clone(), current_epoch)
            .unwrap();
        let res = state.approve_credit(&store, from, to, None, current_epoch, None, None);
        assert!(res.is_ok());

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &hoku_config,
            &store,
            to,
            to,
            from,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
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
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
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
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();

        let res = state.revoke_credit(&store, from, to, None);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("{} not found in accounts", from)
        );
    }

    #[test]
    fn test_debit_accounts_delete_from_disc() {
        setup_logs();
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                origin,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        debit_accounts_delete_from_disc(
            &hoku_config,
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        debit_accounts_delete_from_disc(
            &hoku_config,
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
        hoku_config: &HokuConfig,
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
        let id1 = SubscriptionId::default();
        let ttl1 = ChainEpoch::from(MIN_TTL);
        let source = new_pk();
        let res = state.add_blob(
            hoku_config,
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

        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_accounts, 1);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size);

        // Set to status pending
        let res = state.set_blob_pending(&store, subscriber, hash, id1.clone(), source);
        assert!(res.is_ok());
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 1);
        assert_eq!(stats.bytes_resolving, size);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            id1.clone(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(account.credit_committed, BigInt::from(ttl1 as u64 * size));
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Add the same blob but this time uses a different subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let ttl2 = ChainEpoch::from(MIN_TTL);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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

        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        assert_eq!(
            account.credit_committed, // stays the same becuase we're starting over
            BigInt::from(ttl2 as u64 * size),
        );
        credit_amount -= BigInt::from((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Check the subscription group
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 2);

        // Debit all accounts at an epoch between the two expiries (3601-3621)
        let debit_epoch = ChainEpoch::from(MIN_TTL + 11);
        let deletes_from_disc = state
            .debit_accounts(&hoku_config, &store, debit_epoch)
            .unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            BigInt::from((ttl2 - (debit_epoch - add2_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check the subscription group
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 1); // the first subscription was deleted

        // Debit all accounts at an epoch greater than group expiry (3621)
        let debit_epoch = ChainEpoch::from(MIN_TTL + 31);
        let deletes_from_disc = state
            .debit_accounts(&hoku_config, &store, debit_epoch)
            .unwrap();
        assert!(!deletes_from_disc.is_empty()); // blob is marked for deletion

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // the second debit reduces this to zero
            BigInt::from(0),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, 0);

        // Check state
        assert_eq!(state.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(
            state.credit_debited,
            token_amount.atto() - &account.credit_free
        );
        assert_eq!(state.capacity_used, 0); // capacity was released

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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                origin,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        add_blob_refund(
            &hoku_config,
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        add_blob_refund(
            &hoku_config,
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
        hoku_config: &HokuConfig,
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
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            hoku_config,
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

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size1);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(MIN_TTL as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1);

        assert!(state
            .set_ttl_status(&store, subscriber, TtlStatus::Extended, current_epoch)
            .is_ok());

        // Add another blob past the first blob's expiry
        let (hash2, size2) = new_hash(2048);
        let add2_epoch = ChainEpoch::from(MIN_TTL + 11);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 2);
        assert_eq!(stats.bytes_added, size1 + size2);

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
        assert_eq!(account.capacity_used, size1 + size2);

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
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 2);
        assert_eq!(stats.bytes_added, size1 + size2);

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
        assert_eq!(account.capacity_used, size1 + size2);

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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                origin,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        add_blob_same_hash_same_account(
            &hoku_config,
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        add_blob_same_hash_same_account(
            &hoku_config,
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let caller = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
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
            &hoku_config,
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
        hoku_config: &HokuConfig,
        store: &BS,
        mut state: State,
        origin: Address,
        caller: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
    ) {
        let mut credit_amount = token_amount.atto().clone();

        assert!(state
            .set_ttl_status(&store, subscriber, TtlStatus::Extended, current_epoch)
            .is_ok());

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size);

        // Check the blob status
        assert_eq!(
            state.get_blob_status(&store, subscriber, hash, id1.clone()),
            Some(BlobStatus::Added)
        );

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Added);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 1);
        let got_sub = group.subscriptions.get(&id1.clone().to_string()).unwrap();
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
        assert_eq!(account.capacity_used, size);

        // Set to status pending
        let res = state.set_blob_pending(&store, subscriber, hash, id1.clone(), source);
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 1);
        assert_eq!(stats.bytes_resolving, size);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            id1.clone(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());
        assert_eq!(
            state.get_blob_status(&store, subscriber, hash, id1.clone()),
            Some(BlobStatus::Resolved)
        );

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Add the same blob again with a default subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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
            state.get_blob_status(&store, subscriber, hash, id1.clone()),
            Some(BlobStatus::Resolved)
        );

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 1); // Still only one subscription
        let got_sub = group.subscriptions.get(&id1.clone().to_string()).unwrap();
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
        assert_eq!(account.capacity_used, size); // not changed

        // Add the same blob again but use a different subscription ID
        let add3_epoch = ChainEpoch::from(31);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the blob status
        // Should already be resolved
        assert_eq!(
            state.get_blob_status(&store, subscriber, hash, id2.clone()),
            Some(BlobStatus::Resolved)
        );

        // Check the blob
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        assert_eq!(blob.subscribers.len(), 1); // still only one subscriber
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 2);
        let got_sub = group.subscriptions.get(&id2.clone().to_string()).unwrap();
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
        assert_eq!(account.capacity_used, size); // not changed

        // Debit all accounts
        let debit_epoch = ChainEpoch::from(41);
        let deletes_from_disc = state
            .debit_accounts(&hoku_config, &store, debit_epoch)
            .unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            BigInt::from((AUTO_TTL - (debit_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check indexes
        assert_eq!(state.expiries.len(), 2);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Delete the default subscription ID
        let delete_epoch = ChainEpoch::from(51);
        let res = state.delete_blob(
            &hoku_config,
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
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        assert_eq!(blob.subscribers.len(), 1); // still one subscriber
        assert_eq!(blob.status, BlobStatus::Resolved);
        assert_eq!(blob.size, size);

        // Check the subscription group
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 1);
        let sub = group.subscriptions.get(&id2.clone().to_string()).unwrap();
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
        assert_eq!(account.capacity_used, size); // not changed

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            token_amount.atto() - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, size);

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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                amount.clone(),
                current_epoch,
            )
            .unwrap();
        let mut credit_amount = amount.atto().clone();

        assert!(state
            .set_ttl_status(&store, subscriber, TtlStatus::Extended, current_epoch)
            .is_ok());

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add_epoch = current_epoch;
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
            &store,
            subscriber,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
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
        assert_eq!(account.capacity_used, size);

        // Set to status pending
        let res =
            state.set_blob_pending(&store, subscriber, hash, SubscriptionId::default(), source);
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::default(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());

        // Renew blob
        let renew_epoch = ChainEpoch::from(21);
        let res = state.renew_blob(
            &hoku_config,
            &store,
            subscriber,
            renew_epoch,
            hash,
            SubscriptionId::default(),
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
        assert_eq!(account.capacity_used, size);

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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                amount.clone(),
                current_epoch,
            )
            .unwrap();
        let mut credit_amount = amount.atto().clone();

        assert!(state
            .set_ttl_status(&store, subscriber, TtlStatus::Extended, current_epoch)
            .is_ok());

        // Add blob with default a subscription ID
        let (hash1, size1) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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
        assert_eq!(account.capacity_used, size1);

        // Add another blob past the first blob's expiry
        let (hash2, size2) = new_hash(2048);
        let add2_epoch = ChainEpoch::from(AUTO_TTL + 11);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
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
        assert_eq!(account.capacity_used, size1 + size2);

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
        let res = state.renew_blob(
            &hoku_config,
            &store,
            subscriber,
            renew_epoch,
            hash1,
            id1.clone(),
        );
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
        assert_eq!(account.capacity_used, size1 + size2);

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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                amount.clone(),
                current_epoch,
            )
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &hoku_config,
            &store,
            subscriber,
            subscriber,
            subscriber,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
            size,
            None,
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Finalize as pending
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::default(),
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                amount.clone(),
                current_epoch,
            )
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
            &store,
            subscriber,
            subscriber,
            subscriber,
            current_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Set to status pending
        let res =
            state.set_blob_pending(&store, subscriber, hash, SubscriptionId::default(), source);
        assert!(res.is_ok());

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::default(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(&store, subscriber, hash, SubscriptionId::default())
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                amount.clone(),
                current_epoch,
            )
            .unwrap();
        let credit_amount = amount.atto().clone();

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
            &store,
            subscriber,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
            size,
            None,
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Set to status pending
        let res =
            state.set_blob_pending(&store, subscriber, hash, SubscriptionId::default(), source);
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::default(),
            BlobStatus::Failed,
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(&store, subscriber, hash, SubscriptionId::default())
            .unwrap();
        assert!(matches!(status, BlobStatus::Failed));

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(account.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, 0); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(state.credit_debited, BigInt::from(0));
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed_refund() {
        setup_logs();
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                amount.clone(),
                current_epoch,
            )
            .unwrap();
        let mut credit_amount = amount.atto().clone();

        assert!(state
            .set_ttl_status(&store, subscriber, TtlStatus::Extended, current_epoch)
            .is_ok());

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &hoku_config,
            &store,
            subscriber,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
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
        assert_eq!(account.capacity_used, size);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(state.credit_debited, BigInt::from(0));
        assert_eq!(state.capacity_used, account.capacity_used); // capacity was released

        // Debit accounts to trigger a refund when we fail below
        let debit_epoch = ChainEpoch::from(11);
        let deletes_from_disc = state
            .debit_accounts(&hoku_config, &store, debit_epoch)
            .unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from((AUTO_TTL - (debit_epoch - add_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            BigInt::from((debit_epoch - add_epoch) as u64 * size)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Set to status pending
        let res =
            state.set_blob_pending(&store, subscriber, hash, SubscriptionId::default(), source);
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(21);
        let res = state.finalize_blob(
            &hoku_config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            SubscriptionId::default(),
            BlobStatus::Failed,
        );
        assert!(res.is_ok());

        // Check status
        let status = state
            .get_blob_status(&store, subscriber, hash, SubscriptionId::default())
            .unwrap();
        assert!(matches!(status, BlobStatus::Failed));

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(account.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(account.credit_free, amount.atto().clone()); // credit was refunded
        assert_eq!(account.capacity_used, 0); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, BigInt::from(0)); // credit was released
        assert_eq!(state.credit_debited, BigInt::from(0)); // credit was refunded and released
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_delete_blob_refund() {
        setup_logs();
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                origin,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        delete_blob_refund(
            &hoku_config,
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
        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &hoku_config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(&store, subscriber, origin, None, current_epoch, None, None)
            .unwrap();
        delete_blob_refund(
            &hoku_config,
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

        const YEAR: ChainEpoch = 365 * 24 * 60 * 60;

        // Test cases structure
        struct TestCase {
            name: &'static str,
            account_ttl_status: TtlStatus,
            blob_ttl: Option<ChainEpoch>,
            should_succeed: bool,
            should_auto_renew: bool,
            expected_account_ttl: ChainEpoch,
            expected_blob_ttl: ChainEpoch,
        }

        // Define test cases
        let test_cases = vec![
            TestCase {
                name: "Reduced status rejects even minimum TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: Some(MIN_TTL),
                should_succeed: false,
                should_auto_renew: false,
                expected_account_ttl: 0,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Reduced status rejects no TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: Some(MIN_TTL),
                should_succeed: false,
                should_auto_renew: false,
                expected_account_ttl: 0,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Default status allows default TTL",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(TtlStatus::DEFAULT_MAX_TTL),
                should_succeed: true,
                should_auto_renew: false,
                expected_account_ttl: TtlStatus::DEFAULT_MAX_TTL,
                expected_blob_ttl: TtlStatus::DEFAULT_MAX_TTL,
            },
            TestCase {
                name: "Default status sets no TTL to default without auto renew",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: None,
                should_succeed: true,
                should_auto_renew: false,
                expected_account_ttl: TtlStatus::DEFAULT_MAX_TTL,
                expected_blob_ttl: TtlStatus::DEFAULT_MAX_TTL,
            },
            TestCase {
                name: "Default status preserves given TTL if it's less than default",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(TtlStatus::DEFAULT_MAX_TTL - 1),
                should_succeed: true,
                should_auto_renew: false,
                expected_account_ttl: TtlStatus::DEFAULT_MAX_TTL,
                expected_blob_ttl: TtlStatus::DEFAULT_MAX_TTL - 1,
            },
            TestCase {
                name: "Default status rejects TTLs higher than default",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(TtlStatus::DEFAULT_MAX_TTL + 1),
                should_succeed: false,
                should_auto_renew: false,
                expected_account_ttl: TtlStatus::DEFAULT_MAX_TTL,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Extended status allows any TTL",
                account_ttl_status: TtlStatus::Extended,
                blob_ttl: Some(YEAR),
                should_succeed: true,
                should_auto_renew: false,
                expected_account_ttl: ChainEpoch::MAX,
                expected_blob_ttl: YEAR,
            },
            TestCase {
                name: "Extended status allows auto renew",
                account_ttl_status: TtlStatus::Extended,
                blob_ttl: None,
                should_succeed: true,
                should_auto_renew: true,
                expected_account_ttl: ChainEpoch::MAX,
                expected_blob_ttl: AUTO_TTL,
            },
        ];

        // Run all test cases
        for tc in test_cases {
            let hoku_config = HokuConfig::default();
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let subscriber = new_address();
            let current_epoch = ChainEpoch::from(1);
            let amount = TokenAmount::from_whole(10);

            state
                .buy_credit(
                    &hoku_config,
                    &store,
                    subscriber,
                    amount.clone(),
                    current_epoch,
                )
                .unwrap();
            state
                .set_ttl_status(&store, subscriber, tc.account_ttl_status, current_epoch)
                .unwrap();

            let (hash, size) = new_hash(1024);
            let res = state.add_blob(
                &hoku_config,
                &store,
                subscriber,
                subscriber,
                subscriber,
                current_epoch,
                hash,
                new_metadata_hash(),
                SubscriptionId::default(),
                size,
                tc.blob_ttl,
                new_pk(),
                TokenAmount::zero(),
            );

            let account_ttl = state.get_account_max_ttl(&store, subscriber).unwrap();
            assert_eq!(
                account_ttl, tc.expected_account_ttl,
                "Test case '{}' has unexpected account TTL",
                tc.name
            );

            if tc.should_succeed {
                assert!(
                    res.is_ok(),
                    "Test case '{}' should succeed but failed: {:?}",
                    tc.name,
                    res.err()
                );

                let res = state.get_blob(&store, hash);
                assert!(res.is_ok(), "Failed to get blob: {:?}", res.err());
                let blob = res.unwrap().unwrap();
                for (_, group) in blob.subscribers {
                    for (_, sub) in group.subscriptions {
                        assert_eq!(
                            sub.expiry,
                            current_epoch + tc.expected_blob_ttl,
                            "Test case '{}' has unexpected blob expiry",
                            tc.name
                        );
                        assert_eq!(
                            sub.auto_renew, tc.should_auto_renew,
                            "Test case '{}' has unexpected auto renew value",
                            tc.name
                        );
                    }
                }
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
                        tc.blob_ttl.map_or_else(|| "none".to_string(), |ttl| ttl.to_string()), i64::from(tc.account_ttl_status),
                    ),
                    "Test case '{}' failed with unexpected error message",
                    tc.name
                );
            }
        }
    }

    fn delete_blob_refund<BS: Blockstore>(
        hoku_config: &HokuConfig,
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
            hoku_config,
            &store,
            origin,
            caller,
            subscriber,
            add1_epoch,
            hash1,
            new_metadata_hash(),
            SubscriptionId::default(),
            size1,
            Some(MIN_TTL),
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size1);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            BigInt::from(MIN_TTL as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1);

        // Add another blob past the first blob expiry
        // This will trigger a debit on the account
        let add2_epoch = ChainEpoch::from(MIN_TTL + 10);
        let (hash2, size2) = new_hash(2048);
        let res = state.add_blob(
            &hoku_config,
            &store,
            origin,
            caller,
            subscriber,
            add2_epoch,
            hash2,
            new_metadata_hash(),
            SubscriptionId::default(),
            size2,
            Some(MIN_TTL),
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 2);
        assert_eq!(stats.bytes_added, size1 + size2);

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
        assert_eq!(account.capacity_used, size1 + size2);

        // Delete the first blob
        let delete_epoch = ChainEpoch::from(MIN_TTL + 20);
        let delete_from_disc = state
            .delete_blob(
                &hoku_config,
                &store,
                origin,
                caller,
                subscriber,
                delete_epoch,
                hash1,
                SubscriptionId::default(),
            )
            .unwrap();
        assert!(delete_from_disc);

        // Check stats
        let stats = state.get_stats(TokenAmount::zero(), &hoku_config);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch); // not changed, blob is expired
        assert_eq!(
            account.credit_committed, // should not include overcharge due to refund
            BigInt::from(MIN_TTL as u64 * size2),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed); // credit was released
        assert_eq!(state.credit_debited, BigInt::from(MIN_TTL as u64 * size1));
        assert_eq!(state.capacity_used, size2); // capacity was released

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

    #[test]
    fn test_set_ttl_status() {
        setup_logs();

        struct TestCase {
            name: &'static str,
            initial_ttl_status: Option<TtlStatus>, // None means don't set initial status
            new_ttl_status: TtlStatus,
            expected_ttl: ChainEpoch,
        }

        let test_cases = vec![
            TestCase {
                name: "Setting Reduced on new account",
                initial_ttl_status: None,
                new_ttl_status: TtlStatus::Reduced,
                expected_ttl: 0,
            },
            TestCase {
                name: "Setting Default on new account",
                initial_ttl_status: None,
                new_ttl_status: TtlStatus::Default,
                expected_ttl: TtlStatus::DEFAULT_MAX_TTL,
            },
            TestCase {
                name: "Changing from Default to Reduced",
                initial_ttl_status: Some(TtlStatus::Default),
                new_ttl_status: TtlStatus::Reduced,
                expected_ttl: 0,
            },
            TestCase {
                name: "Changing from Extended to Reduced",
                initial_ttl_status: Some(TtlStatus::Extended),
                new_ttl_status: TtlStatus::Reduced,
                expected_ttl: 0,
            },
            TestCase {
                name: "Changing from Reduced to Extended",
                initial_ttl_status: Some(TtlStatus::Reduced),
                new_ttl_status: TtlStatus::Extended,
                expected_ttl: ChainEpoch::MAX,
            },
        ];

        for tc in test_cases {
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let account = new_address();
            let current_epoch = ChainEpoch::from(1);

            // Initialize account if needed
            if tc.initial_ttl_status.is_some() {
                state
                    .set_ttl_status(
                        &store,
                        account,
                        tc.initial_ttl_status.unwrap(),
                        current_epoch,
                    )
                    .unwrap();
            }

            // Change TTL status
            let res = state.set_ttl_status(&store, account, tc.new_ttl_status, current_epoch);
            assert!(
                res.is_ok(),
                "Test case '{}' failed to set TTL status",
                tc.name
            );

            // Verify max TTL
            let max_ttl = state.get_account_max_ttl(&store, account).unwrap();
            assert_eq!(
                max_ttl, tc.expected_ttl,
                "Test case '{}' failed: expected max TTL {}, got {}",
                tc.name, tc.expected_ttl, max_ttl
            );
        }
    }

    #[test]
    fn test_adjust_blob_ttls_for_account() {
        setup_logs();
        let hoku_config = HokuConfig::default();

        const HOUR: ChainEpoch = 3600;
        const TWO_HOURS: ChainEpoch = HOUR * 2;
        const DAY: ChainEpoch = HOUR * 24;
        const YEAR: ChainEpoch = DAY * 365;

        let blobs_ttls: Vec<Option<ChainEpoch>> =
            vec![None, Some(HOUR), Some(TWO_HOURS), Some(DAY), Some(YEAR)];

        struct TestCase {
            name: &'static str,
            account_ttl: TtlStatus,
            expected_ttls: Vec<ChainEpoch>,
            expected_auto_renewals: Vec<bool>,
            limit: Option<usize>, // None means process all at once
        }

        let test_cases = vec![
            TestCase {
                name: "Set to zero with Reduced status",
                account_ttl: TtlStatus::Reduced,
                expected_ttls: vec![0, 0, 0, 0, 0],
                expected_auto_renewals: vec![false, false, false, false, false],
                limit: None,
            },
            TestCase {
                name: "Set to default with Default status",
                account_ttl: TtlStatus::Default,
                expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, DAY],
                expected_auto_renewals: vec![false, false, false, false, false],
                limit: None,
            },
            TestCase {
                name: "Set to extended with Extended status",
                account_ttl: TtlStatus::Extended,
                expected_ttls: vec![HOUR, HOUR, TWO_HOURS, DAY, YEAR],
                expected_auto_renewals: vec![true, false, false, false, false],
                limit: None,
            },
        ];

        for tc in test_cases {
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let addr = new_address();
            let current_epoch = ChainEpoch::from(1);

            // Setup account with credits and TTL status
            let token = TokenAmount::from_whole(1000);

            state
                .buy_credit(&hoku_config, &store, addr, token, current_epoch)
                .unwrap();
            // Set extended TTL status to allow adding all blobs
            state
                .set_ttl_status(&store, addr, TtlStatus::Extended, current_epoch)
                .unwrap();

            // Add blobs
            let mut blob_hashes = Vec::new();
            let mut total_cost = BigInt::zero();
            let mut expected_credits = BigInt::zero();
            for (i, ttl) in blobs_ttls.iter().enumerate() {
                let size = (i + 1) * 1024;
                let (hash, _) = new_hash(size);
                let size = size as u64;
                blob_hashes.push(hash);

                state
                    .add_blob(
                        &hoku_config,
                        &store,
                        addr,
                        addr,
                        addr,
                        current_epoch,
                        hash,
                        new_metadata_hash(),
                        SubscriptionId::try_from(format!("blob-{}", i)).unwrap(),
                        size as u64,
                        *ttl,
                        new_pk(),
                        TokenAmount::zero(),
                    )
                    .unwrap();

                total_cost += state.get_storage_cost(
                    hoku_config.blob_credits_per_byte_block,
                    ttl.unwrap_or(AUTO_TTL),
                    &size,
                );
                expected_credits += state.get_storage_cost(
                    hoku_config.blob_credits_per_byte_block,
                    tc.expected_ttls[i],
                    &size,
                );
            }

            let account = state.get_account(&store, addr).unwrap().unwrap();
            assert_eq!(
                account.credit_committed, total_cost,
                "Test case '{}' failed: committed credits don't match",
                tc.name
            );

            state
                .set_ttl_status(&store, addr, tc.account_ttl, current_epoch)
                .unwrap();

            let res = state.adjust_blob_ttls_for_account(
                &hoku_config,
                &store,
                addr,
                current_epoch,
                None,
                tc.limit,
            );
            assert!(
                res.is_ok(),
                "Test case '{}' failed to adjust TTLs: {}",
                tc.name,
                res.err().unwrap()
            );

            // Verify TTLs were adjusted correctly
            for (i, hash) in blob_hashes.iter().enumerate() {
                // If the TTL is zero, the blob should be deleted
                if tc.expected_ttls[i] == 0 {
                    assert!(
                        state.get_blob(&store, *hash).unwrap().is_none(),
                        "Test case '{}' failed: blob {} not deleted",
                        tc.name,
                        i
                    );
                } else {
                    let blob = state.get_blob(&store, *hash).unwrap().unwrap();
                    let group = blob.subscribers.get(&addr.to_string()).unwrap();
                    let sub = group.subscriptions.get(&format!("blob-{}", i)).unwrap();

                    assert_eq!(
                        sub.expiry - sub.added,
                        tc.expected_ttls[i],
                        "Test case '{}' failed: blob {} TTL not adjusted correctly. Expected {}, got {}",
                        tc.name,
                        i,
                        tc.expected_ttls[i],
                        sub.expiry - sub.added,
                    );
                    assert_eq!(
                        sub.auto_renew,
                        tc.expected_auto_renewals[i],
                        "Test case '{}' failed: blob {} auto-renewal not adjusted correctly. Expected {}, got {}",
                        tc.name,
                        i,
                        tc.expected_auto_renewals[i],
                        sub.auto_renew
                    );
                }
            }

            let account = state.get_account(&store, addr).unwrap().unwrap();
            assert_eq!(
                account.credit_committed, expected_credits,
                "Test case '{}' failed: account's committed credits after blob adjustment don't match",
                tc.name
            );

            assert_eq!(
                state.credit_committed, expected_credits,
                "Test case '{}' failed: state's committed credits after blob adjustment don't match",
                tc.name
            );
        }
    }

    #[test]
    fn test_adjust_blob_ttls_pagination() {
        setup_logs();
        let hoku_config = HokuConfig::default();

        // Test cases for pagination
        struct PaginationTest {
            name: &'static str,
            limit: Option<usize>,
            start: Option<usize>,
            expected_next_key: Option<usize>,
            expected_processed: usize,
        }

        let test_cases = vec![
            PaginationTest {
                name: "Process all at once",
                limit: None,
                start: None,
                expected_next_key: None,
                expected_processed: 5,
            },
            PaginationTest {
                name: "Process two at a time from beginning",
                limit: Some(2),
                start: None,
                expected_next_key: Some(2),
                expected_processed: 2,
            },
            PaginationTest {
                name: "Process one at a time with offset",
                limit: Some(1),
                start: Some(1),
                expected_next_key: Some(2),
                expected_processed: 1,
            },
            PaginationTest {
                name: "Out of bounds limit",
                limit: Some(10),
                start: Some(1),
                expected_next_key: None,
                expected_processed: 4,
            },
            PaginationTest {
                name: "With offset ending at last item",
                limit: Some(2),
                start: Some(3),
                expected_next_key: None,
                expected_processed: 2,
            },
        ];

        for tc in test_cases {
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let addr = new_address();
            let current_epoch = ChainEpoch::from(1);

            // Setup account with credits and Extended TTL status to allow adding all blobs
            state
                .buy_credit(
                    &hoku_config,
                    &store,
                    addr,
                    TokenAmount::from_whole(1000),
                    current_epoch,
                )
                .unwrap();
            state
                .set_ttl_status(&store, addr, TtlStatus::Extended, current_epoch)
                .unwrap();

            // Add 5 blobs with different sizes to ensure different hashes
            for i in 0..5 {
                let (hash, size) = new_hash((i + 1) * 1024);
                state
                    .add_blob(
                        &hoku_config,
                        &store,
                        addr,
                        addr,
                        addr,
                        current_epoch,
                        hash,
                        new_metadata_hash(),
                        SubscriptionId::try_from(format!("blob-{}", i)).unwrap(),
                        size,
                        Some(7200), // 2 hours
                        new_pk(),
                        TokenAmount::zero(),
                    )
                    .unwrap();
            }

            // range over all blobs and store their hashes
            let mut blob_hashes = Vec::with_capacity(5);
            for _ in 0..5 {
                let res = state.blobs.hamt(&store).unwrap().for_each(
                    |hash, _| -> Result<(), ActorError> {
                        blob_hashes.push(hash);
                        Ok(())
                    },
                );
                assert!(
                    res.is_ok(),
                    "Failed to iterate over blobs: {}",
                    res.err().unwrap()
                );
            }

            // Change to Reduced status and process blobs with pagination
            state
                .set_ttl_status(&store, addr, TtlStatus::Reduced, current_epoch)
                .unwrap();

            let res = state.adjust_blob_ttls_for_account(
                &hoku_config,
                &store,
                addr,
                current_epoch,
                tc.start.map(|ind| blob_hashes[ind]),
                tc.limit,
            );
            assert!(
                res.is_ok(),
                "Test case '{}' failed to adjust TTLs: {}",
                tc.name,
                res.err().unwrap()
            );

            let (processed, next, deleted_blobs) = res.unwrap();

            assert_eq!(
                processed as usize, tc.expected_processed,
                "Test case '{}' had unexpected number of items processed",
                tc.name
            );

            assert_eq!(
                deleted_blobs.len(),
                tc.expected_processed,
                "Test case '{}' had unexpected number of deleted blobs",
                tc.name
            );

            if let Some(expected_next_key) = tc.expected_next_key {
                assert!(next.is_some(), "Test case '{}' expected next key", tc.name);
                assert_eq!(
                    next.unwrap(),
                    blob_hashes[expected_next_key],
                    "Test case '{}' had unexpected next key",
                    tc.name
                );
            } else {
                assert!(next.is_none(), "Test case '{}' had no next key", tc.name);
            }
        }
    }

    #[test]
    fn test_adjust_blob_ttls_for_multiple_accounts() {
        setup_logs();

        let hoku_config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let account1 = new_address();
        let account2 = new_address();
        let current_epoch = ChainEpoch::from(1);

        // Setup accounts with credits and Extended TTL status to allow adding all blobs
        state
            .buy_credit(
                &hoku_config,
                &store,
                account1,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .buy_credit(
                &hoku_config,
                &store,
                account2,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .set_ttl_status(&store, account1, TtlStatus::Extended, current_epoch)
            .unwrap();
        state
            .set_ttl_status(&store, account2, TtlStatus::Extended, current_epoch)
            .unwrap();

        // Add blobs for both accounts
        let mut blob_hashes_account1 = Vec::new();
        let mut blob_hashes_account2 = Vec::new();
        for i in 0..3 {
            let (hash, size) = new_hash((i + 1) * 1024);
            blob_hashes_account1.push(hash);
            state
                .add_blob(
                    &hoku_config,
                    &store,
                    account1,
                    account1,
                    account1,
                    current_epoch,
                    hash,
                    new_metadata_hash(),
                    SubscriptionId::try_from(format!("blob-1-{}", i)).unwrap(),
                    size,
                    Some(7200), // 2 hours
                    new_pk(),
                    TokenAmount::zero(),
                )
                .unwrap();
        }
        for i in 0..3 {
            let (hash, size) = new_hash((i + 1) * 1024);
            blob_hashes_account2.push(hash);
            state
                .add_blob(
                    &hoku_config,
                    &store,
                    account2,
                    account2,
                    account2,
                    current_epoch,
                    hash,
                    new_metadata_hash(),
                    SubscriptionId::try_from(format!("blob-2-{}", i)).unwrap(),
                    size,
                    Some(7200), // 2 hours
                    new_pk(),
                    TokenAmount::zero(),
                )
                .unwrap();
        }

        // Change TTL status for account1 and adjust blobs
        state
            .set_ttl_status(&store, account1, TtlStatus::Reduced, current_epoch)
            .unwrap();
        let res = state.adjust_blob_ttls_for_account(
            &hoku_config,
            &store,
            account1,
            current_epoch,
            None,
            None,
        );
        assert!(
            res.is_ok(),
            "Failed to adjust TTLs for account1: {}",
            res.err().unwrap()
        );

        // Verify account1's blobs were adjusted
        for hash in &blob_hashes_account1 {
            assert!(
                state.get_blob(&store, *hash).unwrap().is_none(),
                "Blob {} for account1 was not deleted",
                hash,
            );
        }

        // Verify account2's blobs were not adjusted
        for hash in &blob_hashes_account2 {
            assert!(
                state.get_blob(&store, *hash).unwrap().is_some(),
                "Blob {} for account2 was incorrectly deleted",
                hash,
            );
        }
    }
}
