// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use fendermint_actor_blobs_shared::params::GetStatsReturn;
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, Credit, CreditApproval, GasAllowance, Hash, PublicKey, Subscription,
    SubscriptionGroup, SubscriptionId, TokenCreditRate, TtlStatus,
};
use fendermint_actor_hoku_config_shared::HokuConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use hoku_ipld::hamt::{BytesKey, MapKey};
use log::{debug, warn};
use num_traits::{ToPrimitive, Zero};

mod accounts;
mod blobs;
mod expiries;

use accounts::AccountsState;
use blobs::{BlobsProgressCollection, BlobsState};
use expiries::{ExpiriesState, ExpiryUpdate};

/// The state represents all accounts and stored blobs.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total used storage capacity of the subnet.
    pub capacity_used: u64,
    /// The total number of credits sold in the subnet.
    pub credit_sold: Credit,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: Credit,
    /// The total number of credits debited in the subnet.
    pub credit_debited: Credit,
    /// Map of expiries to blob hashes.
    pub expiries: ExpiriesState,
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

impl Display for ExpiryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExpiryKey(hash: {}, id: {})", self.hash, self.id)
    }
}

impl MapKey for ExpiryKey {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        let raw_bytes = RawBytes::from(b.to_vec());
        fil_actors_runtime::cbor::deserialize(&raw_bytes, "ExpiryKey")
            .map_err(|e| format!("Failed to deserialize ExpiryKey {}", e))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let raw_bytes = fil_actors_runtime::cbor::serialize(self, "ExpiryKey")
            .map_err(|e| format!("Failed to serialize ExpiryKey {}", e))?;
        Ok(raw_bytes.to_vec())
    }
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
    /// Information about the approval that allows "origin" to use credits.
    /// Note that the Address that has issued this approval (the subscriber/sponsor), and whose
    /// credits are being allowed to be used, are not stored internal to this struct.
    pub approval: &'a mut CreditApproval,
}

impl<'a> CreditDelegation<'a> {
    pub fn new(origin: Address, approval: &'a mut CreditApproval) -> Self {
        Self { origin, approval }
    }
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self, ActorError> {
        Ok(Self {
            capacity_used: 0,
            credit_sold: Credit::zero(),
            credit_committed: Credit::zero(),
            credit_debited: Credit::zero(),
            expiries: ExpiriesState::new(store)?,
            added: BlobsProgressCollection::new(store, "added blobs queue")?,
            pending: BlobsProgressCollection::new(store, "pending blobs queue")?,
            accounts: AccountsState::new(store)?,
            blobs: BlobsState::new(store)?,
        })
    }

    pub fn get_stats(&self, config: &HokuConfig, balance: TokenAmount) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(config.blob_capacity),
            capacity_used: self.capacity_used,
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            token_credit_rate: config.token_credit_rate.clone(),
            num_accounts: self.accounts.len(),
            num_blobs: self.blobs.len(),
            num_added: self.added.len(),
            bytes_added: self.added.bytes_size(),
            num_resolving: self.pending.len(),
            bytes_resolving: self.pending.bytes_size(),
        }
    }

    pub fn buy_credit<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
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

        let credits: Credit = amount.clone() * &config.token_credit_rate;
        // Don't sell credits if we're at storage capacity
        if self.capacity_available(config.blob_capacity).is_zero() {
            return Err(ActorError::forbidden(
                "credits not available (subnet has reached storage capacity)".into(),
            ));
        }
        self.credit_sold += &credits;
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account =
            accounts.get_or_create(&to, || Account::new(current_epoch, config.blob_default_ttl))?;
        account.credit_free += &credits;
        account.gas_allowance += amount;
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&to, account.clone())?);

        debug!("sold {} credits to {}", credits, to);
        Ok(account)
    }

    pub fn update_gas_allowance<BS: Blockstore>(
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
        let delegation =
            if let Some(sponsor) = sponsor {
                let approval = account.approvals_to.get_mut(&from.to_string()).ok_or(
                    ActorError::forbidden(format!(
                        "approval from {} to {} not found",
                        sponsor, from
                    )),
                )?;
                Some(CreditDelegation::new(from, approval))
            } else {
                None
            };
        // Check gas balance and debit
        if add_amount.is_negative() {
            let gas_required = -add_amount.clone();
            ensure_gas_limit(&addr, current_epoch, &gas_required, &delegation)?;
        }

        account.gas_allowance += &add_amount.clone();
        // Update credit approval
        if let Some(delegation) = delegation {
            let origin = delegation.origin;
            let mut origin_account = accounts.get_or_err(&origin)?;
            let origin_approval = origin_account
                .approvals_from
                .get_mut(&addr.to_string())
                .ok_or(ActorError::illegal_state(format!(
                    "approval from {} to {} not found in 'to' account",
                    addr, origin
                )))?;

            delegation.approval.gas_fee_used -= add_amount.clone();
            origin_approval.gas_fee_used -= add_amount.clone();
            // Save delegation origin account
            accounts.set(&origin, origin_account)?;
        }
        // Save accounts
        accounts.set(&addr, account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        if add_amount.is_positive() {
            debug!("refunded {} atto to {}", add_amount.atto(), addr);
        } else {
            debug!(
                "debited {} atto from {}",
                add_amount.atto().magnitude(),
                addr
            );
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn approve_credit<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
        store: &BS,
        from: Address,
        to: Address,
        current_epoch: ChainEpoch,
        credit_limit: Option<Credit>,
        gas_fee_limit: Option<TokenAmount>,
        ttl: Option<ChainEpoch>,
    ) -> anyhow::Result<CreditApproval, ActorError> {
        let credit_limit = credit_limit.map(Credit::from);
        let gas_fee_limit = gas_fee_limit.map(TokenAmount::from);
        if let Some(ttl) = ttl {
            if ttl < config.blob_min_ttl {
                return Err(ActorError::illegal_argument(format!(
                    "minimum approval TTL is {}",
                    config.blob_min_ttl
                )));
            }
        }
        let expiry = ttl.map(|t| t + current_epoch);
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut from_account = accounts.get_or_create(&from, || {
            Account::new(current_epoch, config.blob_default_ttl)
        })?;
        let mut to_account =
            accounts.get_or_create(&to, || Account::new(current_epoch, config.blob_default_ttl))?;
        // Get or add a new approval
        let approval = CreditApproval {
            credit_limit: credit_limit.clone(),
            gas_fee_limit: gas_fee_limit.clone(),
            expiry,
            credit_used: Credit::zero(),
            gas_fee_used: TokenAmount::zero(),
        };
        let from_approval = from_account
            .approvals_to
            .entry(to.to_string())
            .or_insert(approval.clone());
        let to_approval = to_account
            .approvals_from
            .entry(from.to_string())
            .or_insert(approval);
        if from_approval != to_approval {
            return Err(ActorError::illegal_state(format!(
                "approval in 'from' account ({}) doesn't match approval in 'to' account ({})",
                from, to,
            )));
        }

        // Validate approval changes
        if let Some(limit) = credit_limit.clone() {
            if from_approval.credit_used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used credits ({})",
                    from_approval.credit_used
                )));
            }
        }

        if let Some(limit) = gas_fee_limit.clone() {
            if from_approval.gas_fee_used > limit {
                return Err(ActorError::illegal_argument(format!(
                    "limit cannot be less than amount of already used gas fees ({})",
                    from_approval.gas_fee_used
                )));
            }
        }
        from_approval.credit_limit = credit_limit.clone();
        from_approval.gas_fee_limit = gas_fee_limit.clone();
        from_approval.expiry = expiry;
        to_approval.credit_limit = credit_limit;
        to_approval.gas_fee_limit = gas_fee_limit;
        to_approval.expiry = expiry;
        // Save accounts
        let from_approval = from_approval.clone();
        accounts.set(&from, from_account)?;
        accounts.set(&to, to_account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        debug!(
            "approved credits from {} to {} (credit limit: {:?}; gas fee limit: {:?}, expiry: {:?}",
            from, to, from_approval.credit_limit, from_approval.gas_fee_limit, from_approval.expiry
        );
        Ok(from_approval)
    }

    /// Revokes credit from one account to another.
    pub fn revoke_credit<BS: Blockstore>(
        &mut self,
        store: &BS,
        from: Address,
        to: Address,
    ) -> anyhow::Result<(), ActorError> {
        // Get the account
        let mut accounts = self.accounts.hamt(store)?;
        let mut from_account = accounts.get_or_err(&from)?;
        if from_account.approvals_to.remove(&to.to_string()).is_none() {
            return Err(ActorError::not_found(format!(
                "approval from {} to {} not found",
                from, to
            )));
        }
        let mut to_account = accounts.get_or_err(&to)?;
        if to_account
            .approvals_from
            .remove(&from.to_string())
            .is_none()
        {
            return Err(ActorError::not_found(format!(
                "approval from {} to {} not found in 'to' account",
                from, to
            )));
        }
        // Save accounts
        accounts.set(&from, from_account)?;
        accounts.set(&to, to_account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        debug!("revoked credits from {} to {}", from, to);
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
            .map(|a| a.approvals_to.get(&to.to_string()).cloned())
            .and_then(|a| a))
    }

    /// Returns the gas allowance for the given address, including an amount from a default sponsor.
    /// An error returned from this method would be fatal, as it's called from the FVM executor.
    pub fn get_gas_allowance<BS: Blockstore>(
        &self,
        store: &BS,
        from: Address,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<GasAllowance, ActorError> {
        // Get the account or return default allowance
        let accounts = self.accounts.hamt(store)?;
        let account = match accounts.get(&from)? {
            None => return Ok(GasAllowance::default()),
            Some(account) => account,
        };
        let mut allowance = GasAllowance {
            amount: account.gas_allowance.clone(),
            ..Default::default()
        };
        if let Some(credit_sponsor) = account.credit_sponsor {
            let sponsor = match accounts.get(&credit_sponsor)? {
                None => return Ok(allowance),
                Some(account) => account,
            };
            let sponsored = sponsor
                .approvals_to
                .get(&from.to_string())
                .and_then(|approval| {
                    let expiry_valid = approval
                        .expiry
                        .map_or(true, |expiry| expiry > current_epoch);
                    if !expiry_valid {
                        return None;
                    }
                    let gas_allowance = sponsor.gas_allowance.clone();
                    let used = approval.gas_fee_used.clone();
                    let amount = approval
                        .gas_fee_limit
                        .clone()
                        .map_or(gas_allowance.clone(), |limit| {
                            (limit - used).min(gas_allowance)
                        });
                    Some(amount)
                })
                .unwrap_or(TokenAmount::zero());
            allowance.sponsor = Some(credit_sponsor);
            allowance.sponsored_amount = sponsored;
        } else {
            return Ok(allowance);
        }
        Ok(allowance)
    }

    pub fn set_account_sponsor<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
        store: &BS,
        from: Address,
        sponsor: Option<Address>,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<(), ActorError> {
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_create(&from, || {
            Account::new(current_epoch, config.blob_default_ttl)
        })?;
        account.credit_sponsor = sponsor;
        // Save account
        self.accounts
            .save_tracked(accounts.set_and_flush_tracked(&from, account)?);

        debug!("set credit sponsor for {} to {:?}", from, sponsor);
        Ok(())
    }

    pub fn set_account_status<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
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
                    account.max_ttl = status.get_max_ttl(config.blob_default_ttl);
                    self.accounts
                        .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
                }
            }
            _ => {
                // Get or create a new account
                let max_ttl = status.get_max_ttl(config.blob_default_ttl);
                let mut account =
                    accounts.get_or_create(&subscriber, || Account::new(current_epoch, max_ttl))?;
                account.max_ttl = max_ttl;
                self.accounts
                    .save_tracked(accounts.set_and_flush_tracked(&subscriber, account)?);
            }
        }
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
        let mut num_deleted = 0;
        let expiries = self.expiries.clone();
        expiries.foreach_up_to_epoch(store, current_epoch, |_, subscriber, key| {
            match self.delete_blob(
                store,
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
            Ok(())
        })?;
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
            let debit_credits =
                Credit::from_whole(self.get_storage_cost(debit_blocks, &account.capacity_used));
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
    /// @param subscriber - The address responsible for the subscription to keep this blob around.
    ///   This is whose credits will be spent by this transaction, and going forward to continue to
    ///   pay for the blob over time. Generally, this is the owner of the wrapping Actor
    ///   (e.g., Buckets, Timehub).
    #[allow(clippy::too_many_arguments)]
    pub fn add_blob<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
        store: &BS,
        origin: Address,
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
        let mut account = accounts.get_or_create(&subscriber, || {
            Account::new(current_epoch, config.blob_default_ttl)
        })?;
        // Validate the TTL
        let ttl = self.validate_ttl(config, ttl, &account)?;
        // Get the credit delegation if needed
        let delegation =
            if origin != subscriber {
                // Look for an approval for origin from subscriber
                let approval = account.approvals_to.get_mut(&origin.to_string()).ok_or(
                    ActorError::forbidden(format!(
                        "approval from {} to {} not found",
                        subscriber, origin
                    )),
                )?;
                Some(CreditDelegation::new(origin, approval))
            } else {
                None
            };
        // Capacity updates and required credit depend on whether the subscriber is already
        // subscribing to this blob
        let expiry = current_epoch + ttl;
        let mut new_capacity: u64 = 0;
        let mut new_account_capacity: u64 = 0;
        let credit_required: Credit;
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
                        let return_credits = Credit::from_whole(
                            self.get_storage_cost(current_epoch - group_expiry, &size),
                        );
                        // Return over-debited credit
                        self.credit_debited -= &return_credits;
                        self.credit_committed += &return_credits;
                        account.credit_committed += &return_credits;
                        debug!("returned {} credits to {}", return_credits, subscriber);
                    }
                }
                // Ensure subscriber has enough credits, considering the subscription group may
                // have expiries that cover a portion of the addition.
                // Required credit can be negative if subscriber is reducing expiry.
                // When adding, the new group expiry will always contain a value.
                let new_group_expiry = new_group_expiry.unwrap();
                let group_expiry = group_expiry.map_or(current_epoch, |e| e.max(current_epoch));
                credit_required = Credit::from_whole(
                    self.get_storage_cost(new_group_expiry - group_expiry, &size),
                );
                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &config.token_credit_rate,
                    &tokens_received,
                    &subscriber,
                    current_epoch,
                    &delegation,
                )?;
                if let Some(sub) = group.subscriptions.get_mut(&id.to_string()) {
                    // Update expiry index
                    if expiry != sub.expiry {
                        self.expiries.update_index(
                            store,
                            subscriber,
                            hash,
                            &id,
                            vec![ExpiryUpdate::Add(expiry), ExpiryUpdate::Remove(sub.expiry)],
                        )?;
                    }
                    sub.expiry = expiry;
                    // Overwrite source allows subscriber to retry resolving
                    sub.source = source;
                    sub.delegate = delegation.as_ref().map(|d| d.origin);
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
                        source,
                        delegate: delegation.as_ref().map(|d| d.origin),
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
                    self.expiries.update_index(
                        store,
                        subscriber,
                        hash,
                        &id,
                        vec![ExpiryUpdate::Add(expiry)],
                    )?;
                    sub
                }
            } else {
                new_account_capacity = size;
                // One or more accounts have already committed credit.
                // However, we still need to reserve the full required credit from the new
                // subscriber, as the existing account(s) may decide to change the expiry or cancel.
                credit_required = Credit::from_whole(self.get_storage_cost(ttl, &size));
                tokens_unspent = ensure_credit_or_buy(
                    &mut account.credit_free,
                    &mut self.credit_sold,
                    &credit_required,
                    &config.token_credit_rate,
                    &tokens_received,
                    &subscriber,
                    current_epoch,
                    &delegation,
                )?;
                // Add new subscription
                let sub = Subscription {
                    added: current_epoch,
                    expiry,
                    source,
                    delegate: delegation.as_ref().map(|d| d.origin),
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
                self.expiries.update_index(
                    store,
                    subscriber,
                    hash,
                    &id,
                    vec![ExpiryUpdate::Add(expiry)],
                )?;
                sub
            };
            if !matches!(blob.status, BlobStatus::Resolved) {
                // It's pending or failed, reset to added status
                blob.status = BlobStatus::Added;
                // Add to or update the source in the added queue
                self.added
                    .upsert(store, hash, (subscriber, id, source), blob.size)?;
            }
            (sub, blob)
        } else {
            new_account_capacity = size;
            // New blob increases network capacity as well.
            // Ensure there is enough capacity available.
            let available_capacity = self.capacity_available(config.blob_capacity);
            if size > available_capacity {
                return Err(ActorError::forbidden(format!(
                    "subnet has insufficient storage capacity (available: {}; required: {})",
                    available_capacity, size
                )));
            }
            new_capacity = size;
            credit_required = Credit::from_whole(self.get_storage_cost(ttl, &size));
            tokens_unspent = ensure_credit_or_buy(
                &mut account.credit_free,
                &mut self.credit_sold,
                &credit_required,
                &config.token_credit_rate,
                &tokens_received,
                &subscriber,
                current_epoch,
                &delegation,
            )?;
            // Create new blob
            let sub = Subscription {
                added: current_epoch,
                expiry,
                source,
                delegate: delegation.as_ref().map(|d| d.origin),
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
            self.expiries.update_index(
                store,
                subscriber,
                hash,
                &id,
                vec![ExpiryUpdate::Add(expiry)],
            )?;
            // Add the source to the added queue
            self.added
                .upsert(store, hash, (subscriber, id, source), blob.size)?;
            (sub, blob)
        };
        // Account capacity is changing, debit for existing usage
        let debit = Credit::from_whole(self.get_storage_cost(
            current_epoch - account.last_debit_epoch,
            &account.capacity_used,
        ));
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
            let origin = delegation.origin;
            let mut origin_account = accounts.get_or_err(&origin)?;
            let origin_approval = origin_account
                .approvals_from
                .get_mut(&subscriber.to_string())
                .ok_or(ActorError::illegal_state(format!(
                    "approval from {} to {} not found in 'to' account",
                    subscriber, origin
                )))?;

            delegation.approval.credit_used += &credit_required;
            origin_approval.credit_used += &credit_required;
            // Save delegation origin account
            accounts.set(&origin, origin_account)?;
        }
        // Save accounts
        accounts.set(&subscriber, account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);

        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);

        if credit_required.is_positive() {
            debug!("committed {} credits from {}", credit_required, subscriber);
        } else {
            debug!(
                "released {} credits to {}",
                credit_required.atto().magnitude(),
                subscriber
            );
        }
        Ok((sub, tokens_unspent))
    }

    fn get_storage_cost(&self, ttl: i64, size: &u64) -> BigInt {
        ttl * BigInt::from(*size)
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
    pub fn get_added_blobs<BS: Blockstore>(
        &self,
        store: &BS,
        size: u32,
    ) -> anyhow::Result<Vec<(Hash, HashSet<(Address, SubscriptionId, PublicKey)>)>, ActorError>
    {
        self.added.take_page(store, size)
    }

    #[allow(clippy::type_complexity)]
    pub fn get_pending_blobs<BS: Blockstore>(
        &self,
        store: &BS,
        size: u32,
    ) -> anyhow::Result<Vec<(Hash, HashSet<(Address, SubscriptionId, PublicKey)>)>, ActorError>
    {
        self.pending.take_page(store, size)
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
        // Add the source to the pending queue
        self.pending
            .upsert(store, hash, (subscriber, id, source), blob.size)?;
        // Remove entire blob entry from the added queue
        self.added.remove_entry(store, &hash, blob.size)?;
        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finalize_blob<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
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
        let mut account = accounts.get_or_create(&subscriber, || {
            Account::new(current_epoch, config.blob_default_ttl)
        })?;
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
        let delegation = if let Some(origin) = sub.delegate {
            // Look for an approval for origin from subscriber
            account
                .approvals_to
                .get_mut(&origin.to_string())
                .map(|approval| CreditDelegation::new(origin, approval))
        } else {
            None
        };
        // Update blob status
        blob.status = status;
        debug!("finalized blob {} to status {}", hash, blob.status);
        if matches!(blob.status, BlobStatus::Failed) {
            // When failing, the existing group expiry will always contain a value.
            let group_expiry = group_expiry.unwrap();
            let size = blob.size;
            // We're not going to make a debit, but we need to refund any spent credits that may
            // have been used on this group in the event the last debit is later than the
            // added epoch.
            if account.last_debit_epoch > sub.added && sub_is_min_added {
                // The refund extends up to either the next minimum added epoch that is less
                // than the last debit epoch, or the last debit epoch.
                let cutoff = next_min_added
                    .unwrap_or(account.last_debit_epoch)
                    .min(account.last_debit_epoch);
                let refund_credits =
                    Credit::from_whole(self.get_storage_cost(cutoff - sub.added, &size));
                // Refund credit
                self.credit_debited -= &refund_credits;
                account.credit_free += &refund_credits; // move directly to free
                debug!("refunded {} credits to {}", refund_credits, subscriber);
                // Correct for over-refund
                if cutoff > group_expiry {
                    let correction_credits =
                        Credit::from_whole(self.get_storage_cost(cutoff - group_expiry, &size));
                    self.credit_committed += &correction_credits;
                    account.credit_committed += &correction_credits;
                    account.credit_free -= &correction_credits;
                    debug!(
                        "corrected refund with {} credits to {}",
                        correction_credits, subscriber
                    );
                }
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
            if account.last_debit_epoch < group_expiry {
                let reclaim_credits = Credit::from_whole(self.get_storage_cost(
                    group_expiry
                        - new_group_expiry.map_or(account.last_debit_epoch, |e| {
                            e.max(account.last_debit_epoch)
                        }),
                    &size,
                ));
                self.credit_committed -= &reclaim_credits;
                account.credit_committed -= &reclaim_credits;
                account.credit_free += &reclaim_credits;
                // Update credit approval
                if let Some(delegation) = delegation {
                    delegation.approval.credit_used -= &reclaim_credits;

                    let origin = delegation.origin;
                    let mut origin_account = accounts.get_or_err(&origin)?;
                    let origin_approval = origin_account
                        .approvals_from
                        .get_mut(&subscriber.to_string())
                        .ok_or(ActorError::illegal_state(format!(
                            "approval from {} to {} not found in 'to' account",
                            subscriber, origin
                        )))?;

                    delegation.approval.credit_used -= &reclaim_credits;
                    origin_approval.credit_used -= &reclaim_credits;
                    // Save delegation origin account
                    accounts.set(&origin, origin_account)?;
                }
                debug!("released {} credits to {}", reclaim_credits, subscriber);
            }
            sub.failed = true;
        }
        // Remove the source from the pending queue
        self.pending
            .remove_source(store, hash, (subscriber, id, sub.source), blob.size)?;
        // Save accounts
        accounts.set(&subscriber, account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);
        // Save blob
        self.blobs
            .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn delete_blob<BS: Blockstore>(
        &mut self,
        store: &BS,
        origin: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        id: SubscriptionId,
    ) -> anyhow::Result<bool, ActorError> {
        // Get or create a new account
        let mut accounts = self.accounts.hamt(store)?;
        let mut account = accounts.get_or_err(&subscriber)?;
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
        let delegation = if let Some(origin) = sub.delegate {
            // Look for an approval for origin from subscriber
            let approval = account.approvals_to.get_mut(&origin.to_string());
            if let Some(approval) = approval {
                Some(CreditDelegation::new(origin, approval))
            } else {
                // Approval may have been removed, or this is a call from the system actor,
                // in which case the origin will be supplied as the subscriber
                if origin != subscriber {
                    return Err(ActorError::forbidden(format!(
                        "approval from {} to {} not found",
                        subscriber, origin
                    )));
                }
                None
            }
        } else {
            None
        };
        // If the subscription does not have a delegate, the origin must be the subscriber.
        // If the subscription has a delegate, it must be the origin or the
        // origin must be the subscriber.
        match &delegation {
            None => {
                if origin != subscriber {
                    return Err(ActorError::forbidden(format!(
                        "origin {} is not subscriber {} for blob {}",
                        origin, subscriber, hash
                    )));
                }
            }
            Some(delegation) => {
                if origin != delegation.origin && origin != subscriber {
                    return Err(ActorError::forbidden(format!(
                        "origin {} is not delegate origin {} or subscriber {} for blob {}",
                        origin, delegation.origin, subscriber, hash
                    )));
                }
                if let Some(expiry) = delegation.approval.expiry {
                    if expiry <= current_epoch {
                        return Err(ActorError::forbidden(format!(
                            "approval from {} to {} expired",
                            subscriber, delegation.origin
                        )));
                    }
                }
            }
        }
        // Do not allow deletion if status is added or pending.
        // This would cause issues with deletion from disc.
        if matches!(blob.status, BlobStatus::Added) || matches!(blob.status, BlobStatus::Pending) {
            return Err(ActorError::forbidden(format!(
                "blob {} pending finalization; please wait",
                hash
            )));
        }
        // Since the charge will be for all the account's blobs, we can only
        // account for capacity up to this blob's expiry if it is less than
        // the current epoch.
        // If the subscription is failed, there may be no group expiry.
        if let Some(group_expiry) = group_expiry {
            let debit_epoch = group_expiry.min(current_epoch);
            // Account capacity is changing, debit for existing usage.
            // It could be possible that debit epoch is less than the last debit,
            // in which case we need to refund for that duration.
            if account.last_debit_epoch < debit_epoch {
                let debit = Credit::from_whole(self.get_storage_cost(
                    debit_epoch - account.last_debit_epoch,
                    &account.capacity_used,
                ));
                self.credit_debited += &debit;
                self.credit_committed -= &debit;
                account.credit_committed -= &debit;
                account.last_debit_epoch = debit_epoch;
                debug!("debited {} credits from {}", debit, subscriber);
            } else if account.last_debit_epoch != debit_epoch {
                // The account was debited after this blob's expiry
                let return_credits = Credit::from_whole(
                    self.get_storage_cost(account.last_debit_epoch - group_expiry, &blob.size),
                );
                // Return over-debited credit
                self.credit_debited -= &return_credits;
                self.credit_committed += &return_credits;
                account.credit_committed += &return_credits;
                debug!("returned {} credits to {}", return_credits, subscriber);
            }
        }
        // Account for reclaimed size and move committed credit to free credit
        // If blob failed, capacity and committed credits have already been returned
        if !matches!(blob.status, BlobStatus::Failed) && !sub.failed {
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
            if let Some(group_expiry) = group_expiry {
                if account.last_debit_epoch < group_expiry {
                    let reclaim_credits = Credit::from_whole(self.get_storage_cost(
                        group_expiry
                            - new_group_expiry.map_or(account.last_debit_epoch, |e| {
                                e.max(account.last_debit_epoch)
                            }),
                        &blob.size,
                    ));
                    self.credit_committed -= &reclaim_credits;
                    account.credit_committed -= &reclaim_credits;
                    account.credit_free += &reclaim_credits;
                    // Update credit approval
                    if let Some(delegation) = delegation {
                        delegation.approval.credit_used -= &reclaim_credits;

                        let origin = delegation.origin;
                        let mut origin_account = accounts.get_or_err(&origin)?;
                        let origin_approval = origin_account
                            .approvals_from
                            .get_mut(&subscriber.to_string())
                            .ok_or(ActorError::illegal_state(format!(
                                "approval from {} to {} not found in 'to' account",
                                subscriber, origin
                            )))?;

                        delegation.approval.credit_used -= &reclaim_credits;
                        origin_approval.credit_used -= &reclaim_credits;
                        // Save delegation origin account
                        accounts.set(&origin, origin_account)?;
                    }
                    debug!("released {} credits to {}", reclaim_credits, subscriber);
                }
            }
        }
        // Update expiry index
        self.expiries.update_index(
            store,
            subscriber,
            hash,
            &id,
            vec![ExpiryUpdate::Remove(sub.expiry)],
        )?;
        // Remove the source from the added queue
        self.added
            .remove_source(store, hash, (subscriber, id.clone(), sub.source), blob.size)?;
        // Remove the source from the pending queue
        self.pending
            .remove_source(store, hash, (subscriber, id.clone(), sub.source), blob.size)?;
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
                let (res, _) = blobs.delete_and_flush_tracked(&hash)?;
                self.blobs.save_tracked(res);
                debug!("deleted blob {}", hash);
            }
            delete_blob
        } else {
            self.blobs
                .save_tracked(blobs.set_and_flush_tracked(&hash, blob)?);
            false
        };
        // Save accounts
        accounts.set(&subscriber, account)?;
        self.accounts.save_tracked(accounts.flush_tracked()?);
        Ok(delete_blob)
    }

    /// Return available capacity as a difference between `blob_capacity_total` and `capacity_used`.
    fn capacity_available(&self, blob_capacity_total: u64) -> u64 {
        blob_capacity_total - self.capacity_used
    }

    /// Adjusts all subscriptions for `account` according to its max TTL.
    /// Returns the number of subscriptions processed and the next key to continue iteration.
    /// If `starting_hash` is `None`, iteration starts from the beginning.
    /// If `limit` is `None`, all subscriptions are processed.
    /// If `limit` is not `None`, iteration stops after examining `limit` blobs.
    pub fn trim_blob_expiries<BS: Blockstore>(
        &mut self,
        config: &HokuConfig,
        store: &BS,
        subscriber: Address,
        current_epoch: ChainEpoch,
        starting_hash: Option<Hash>,
        limit: Option<usize>,
    ) -> anyhow::Result<(u32, Option<Hash>, Vec<Hash>), ActorError> {
        let new_ttl = self.get_account_max_ttl(config, store, subscriber)?;
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
                                    store,
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
                                    config,
                                    store,
                                    subscriber,
                                    subscriber,
                                    current_epoch,
                                    hash,
                                    blob.metadata_hash,
                                    SubscriptionId::new(&id.clone())?,
                                    blob.size,
                                    Some(new_ttl),
                                    sub.source,
                                    TokenAmount::zero(),
                                )?;
                            }
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
        config: &HokuConfig,
        store: &BS,
        account: Address,
    ) -> Result<ChainEpoch, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        Ok(accounts
            .get(&account)?
            .map_or(config.blob_default_ttl, |account| account.max_ttl))
    }

    fn validate_ttl(
        &self,
        config: &HokuConfig,
        ttl: Option<ChainEpoch>,
        account: &Account,
    ) -> anyhow::Result<ChainEpoch, ActorError> {
        let ttl = ttl.unwrap_or(config.blob_default_ttl);
        if ttl < config.blob_min_ttl {
            return Err(ActorError::illegal_argument(format!(
                "minimum blob TTL is {}",
                config.blob_min_ttl
            )));
        } else if ttl > account.max_ttl {
            return Err(ActorError::forbidden(format!(
                "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
                ttl, account.max_ttl,
            )));
        }
        Ok(ttl)
    }
}

/// Check if `subscriber` has enough credits, including delegated credits.
fn ensure_credit(
    subscriber: &Address,
    current_epoch: ChainEpoch,
    credit_free: &Credit,
    credit_required: &Credit,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    ensure_enough_credits(subscriber, credit_free, credit_required)?;
    ensure_delegated_credit(subscriber, current_epoch, credit_required, delegation)
}

/// Check if `subscriber` owns enough free credits.
fn ensure_enough_credits(
    subscriber: &Address,
    credit_free: &Credit,
    credit_required: &Credit,
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
    account_credit_free: &mut Credit,
    state_credit_sold: &mut Credit,
    credit_required: &Credit,
    token_credit_rate: &TokenCreditRate,
    tokens_received: &TokenAmount,
    subscriber: &Address,
    current_epoch: ChainEpoch,
    delegate: &Option<CreditDelegation>,
) -> anyhow::Result<TokenAmount, ActorError> {
    let tokens_received_non_zero = !tokens_received.is_zero();
    let has_delegation = delegate.is_some();
    match (tokens_received_non_zero, has_delegation) {
        (true, true) => Err(ActorError::illegal_argument(format!(
            "cannot buy credits inline for {}",
            subscriber,
        ))),
        (true, false) => {
            // Try buying credits for self
            let not_enough_credits = *account_credit_free < *credit_required;
            if not_enough_credits {
                let credits_needed: Credit = credit_required - &*account_credit_free;
                let tokens_needed = &credits_needed / token_credit_rate;
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
    credit_required: &Credit,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    if let Some(delegation) = delegation {
        if let Some(limit) = &delegation.approval.credit_limit {
            let unused = &(limit - &delegation.approval.credit_used);
            if unused < credit_required {
                return Err(ActorError::insufficient_funds(format!(
                    "approval from {} to {} has insufficient credit (available: {}; required: {})",
                    subscriber, delegation.origin, unused, credit_required
                )));
            }
        }
        if let Some(expiry) = delegation.approval.expiry {
            if expiry <= current_epoch {
                return Err(ActorError::forbidden(format!(
                    "approval from {} to {} expired",
                    subscriber, delegation.origin
                )));
            }
        }
    }
    Ok(())
}

fn ensure_gas_limit(
    subscriber: &Address,
    current_epoch: ChainEpoch,
    gas_required: &TokenAmount,
    delegation: &Option<CreditDelegation>,
) -> anyhow::Result<(), ActorError> {
    if let Some(delegation) = delegation {
        if let Some(limit) = &delegation.approval.gas_fee_limit {
            let unused = &(limit - &delegation.approval.gas_fee_used);
            if unused < gas_required {
                return Err(ActorError::insufficient_funds(format!(
                    "approval from {} to {} has insufficient credit (available: {}; required: {})",
                    subscriber, delegation.origin, unused, gas_required
                )));
            }
        }
        if let Some(expiry) = delegation.approval.expiry {
            if expiry <= current_epoch {
                return Err(ActorError::forbidden(format!(
                    "approval from {} to {} expired",
                    subscriber, delegation.origin
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use fendermint_actor_blobs_testing::{
        new_address, new_hash, new_metadata_hash, new_pk, new_subscription_id, setup_logs,
    };
    use fvm_ipld_blockstore::MemoryBlockstore;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::collections::BTreeMap;
    use std::ops::{AddAssign, SubAssign};

    fn check_approval_used<BS: Blockstore>(
        state: &State,
        store: &BS,
        origin: Address,
        subscriber: Address,
    ) {
        let subscriber_account = state.get_account(&store, subscriber).unwrap().unwrap();
        let subscriber_approval = subscriber_account
            .approvals_to
            .get(&origin.to_string())
            .unwrap();
        assert_eq!(
            subscriber_approval.credit_used,
            state.credit_debited.clone() + subscriber_account.credit_committed.clone()
        );
        let origin_account = state.get_account(&store, origin).unwrap().unwrap();
        let origin_approval = origin_account
            .approvals_from
            .get(&subscriber.to_string())
            .unwrap();
        assert_eq!(
            subscriber_approval.credit_used,
            &state.credit_debited + &subscriber_account.credit_committed
        );
        assert_eq!(subscriber_approval.credit_used, origin_approval.credit_used);
    }

    fn check_approvals_match(
        state: &State,
        store: &MemoryBlockstore,
        from: Address,
        to: Address,
        expected: CreditApproval,
    ) {
        let from_account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(
            *from_account.approvals_to.get(&to.to_string()).unwrap(),
            expected
        );
        let to_account = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(
            *to_account.approvals_from.get(&from.to_string()).unwrap(),
            expected
        );
    }

    #[test]
    fn test_buy_credit_success() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let to = new_address();
        let amount = TokenAmount::from_whole(1);

        let res = state.buy_credit(&config, &store, to, amount.clone(), 1);
        assert!(res.is_ok());
        let account = res.unwrap();
        let credit_sold = amount.clone() * &config.token_credit_rate;
        assert_eq!(account.credit_free, credit_sold);
        assert_eq!(account.gas_allowance, amount);
        assert_eq!(state.credit_sold, credit_sold);
        let account_back = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(account, account_back);
    }

    #[test]
    fn test_buy_credit_negative_amount() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(-1);

        let res = state.buy_credit(&config, &store, recipient, amount, 1);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap().msg(), "token amount must be positive");
    }

    #[test]
    fn test_buy_credit_at_capacity() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let recipient = new_address();
        let amount = TokenAmount::from_whole(1);

        state.capacity_used = config.blob_capacity;
        let res = state.buy_credit(&config, &store, recipient, amount, 1);
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

        let config = HokuConfig::default();

        // No limit or expiry
        let res = state.approve_credit(&config, &store, from, to, current_epoch, None, None, None);
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, None);
        assert_eq!(approval.gas_fee_limit, None);
        assert_eq!(approval.expiry, None);
        check_approvals_match(&state, &store, from, to, approval);

        // Add credit limit
        let limit = 1_000_000_000_000_000_000u64;
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            current_epoch,
            Some(Credit::from_whole(limit)),
            None,
            None,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, Some(Credit::from_whole(limit)));
        assert_eq!(approval.gas_fee_limit, None);
        assert_eq!(approval.expiry, None);
        check_approvals_match(&state, &store, from, to, approval);

        // Add gas fee limit
        let limit = 1_000_000_000_000_000_000u64;
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            current_epoch,
            None,
            Some(TokenAmount::from_atto(limit)),
            None,
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, None);
        assert_eq!(approval.gas_fee_limit, Some(TokenAmount::from_atto(limit)));
        assert_eq!(approval.expiry, None);
        check_approvals_match(&state, &store, from, to, approval);

        // Add ttl
        let ttl = ChainEpoch::from(config.blob_min_ttl);
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            current_epoch,
            Some(Credit::from_whole(limit)),
            None,
            Some(ttl),
        );
        assert!(res.is_ok());
        let approval = res.unwrap();
        assert_eq!(approval.credit_limit, Some(Credit::from_whole(limit)));
        assert_eq!(approval.gas_fee_limit, None);
        assert_eq!(approval.expiry, Some(ttl + current_epoch));
        check_approvals_match(&state, &store, from, to, approval);
    }

    #[test]
    fn test_approve_credit_invalid_ttl() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let config = HokuConfig::default();
        let ttl = ChainEpoch::from(config.blob_min_ttl - 1);
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            current_epoch,
            None,
            None,
            Some(ttl),
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("minimum approval TTL is {}", config.blob_min_ttl)
        );
    }

    #[test]
    fn test_approve_credit_insufficient_credit() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();
        let current_epoch = 1;

        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, from, amount.clone(), current_epoch)
            .unwrap();
        let res = state.approve_credit(&config, &store, from, to, current_epoch, None, None, None);
        assert!(res.is_ok());

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &config,
            &store,
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
        let approval = account.approvals_to.get(&to.to_string()).unwrap();
        assert_eq!(account.credit_committed, approval.credit_used);

        // Try to update approval with a limit below what's already been committed
        let limit = 1_000u64;
        let res = state.approve_credit(
            &config,
            &store,
            from,
            to,
            current_epoch,
            Some(Credit::from_whole(limit)),
            None,
            None,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!(
                "limit cannot be less than amount of already used credits ({})",
                approval.credit_used
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

        let config = HokuConfig::default();
        let res = state.approve_credit(&config, &store, from, to, current_epoch, None, None, None);
        assert!(res.is_ok());

        // Check the account approvals
        let from_account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(from_account.approvals_to.len(), 1);
        let to_account = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(to_account.approvals_from.len(), 1);

        // Remove the approval
        let res = state.revoke_credit(&store, from, to);
        assert!(res.is_ok());
        let from_account = state.get_account(&store, from).unwrap().unwrap();
        assert_eq!(from_account.approvals_to.len(), 0);
        let to_account = state.get_account(&store, to).unwrap().unwrap();
        assert_eq!(to_account.approvals_from.len(), 0);
    }

    #[test]
    fn test_revoke_credit_account_not_found() {
        setup_logs();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let from = new_address();
        let to = new_address();

        let res = state.revoke_credit(&store, from, to);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().msg(),
            format!("{} not found in accounts", from)
        );
    }

    #[test]
    fn test_debit_accounts_delete_from_disc() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        debit_accounts_delete_from_disc(
            &config,
            &store,
            state,
            origin,
            origin,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_debit_accounts_delete_from_disc_with_approval() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                subscriber,
                origin,
                current_epoch,
                None,
                None,
                None,
            )
            .unwrap();
        debit_accounts_delete_from_disc(
            &config,
            &store,
            state,
            origin,
            subscriber,
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn debit_accounts_delete_from_disc<BS: Blockstore>(
        config: &HokuConfig,
        store: &BS,
        mut state: State,
        origin: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let mut credit_amount =
            Credit::from_atto(token_amount.atto().clone()) * &config.token_credit_rate;

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let ttl1 = ChainEpoch::from(config.blob_min_ttl);
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
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

        let stats = state.get_stats(config, TokenAmount::zero());
        // Using a credit delegation creates both the from and to account
        let expected_num_accounts = if using_approval { 2 } else { 1 };
        assert_eq!(stats.num_accounts, expected_num_accounts);
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size);

        // Set to status pending
        let res = state.set_blob_pending(&store, subscriber, hash, id1.clone(), source);
        assert!(res.is_ok());
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 1);
        assert_eq!(stats.bytes_resolving, size);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            config,
            &store,
            subscriber,
            finalize_epoch,
            hash,
            id1.clone(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(ttl1 as u64 * size)
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Add the same blob but this time uses a different subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let ttl2 = ChainEpoch::from(config.blob_min_ttl);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
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

        let stats = state.get_stats(config, TokenAmount::zero());
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
            Credit::from_whole(ttl2 as u64 * size),
        );
        credit_amount -= Credit::from_whole((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Check the subscription group
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 2);

        // Debit all accounts at an epoch between the two expiries (3601-3621)
        let debit_epoch = ChainEpoch::from(config.blob_min_ttl + 11);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            Credit::from_whole((ttl2 - (debit_epoch - add2_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check the subscription group
        let blob = state.get_blob(&store, hash).unwrap().unwrap();
        let group = blob.subscribers.get(&subscriber.to_string()).unwrap();
        assert_eq!(group.subscriptions.len(), 1); // the first subscription was deleted

        // Debit all accounts at an epoch greater than group expiry (3621)
        let debit_epoch = ChainEpoch::from(config.blob_min_ttl + 31);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(!deletes_from_disc.is_empty()); // blob is marked for deletion

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // the second debit reduces this to zero
            Credit::from_whole(0),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, 0);

        // Check state
        assert_eq!(state.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(
            state.credit_debited,
            token_amount * &config.token_credit_rate - &account.credit_free
        );
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 0);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, origin, subscriber);
        }
    }

    #[test]
    fn test_add_blob_refund() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        add_blob_refund(
            &config,
            &store,
            state,
            origin,
            origin,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_add_blob_refund_with_approval() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                subscriber,
                origin,
                current_epoch,
                None,
                None,
                None,
            )
            .unwrap();
        add_blob_refund(
            &config,
            &store,
            state,
            origin,
            subscriber,
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn add_blob_refund<BS: Blockstore>(
        config: &HokuConfig,
        store: &BS,
        mut state: State,
        origin: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let token_credit_rate = BigInt::from(1_000_000_000_000_000_000u64);
        let mut credit_amount = token_amount.clone() * &config.token_credit_rate;

        // Add blob with default a subscription ID
        let (hash1, size1) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add1_epoch,
            hash1,
            new_metadata_hash(),
            id1.clone(),
            size1,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
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
            Credit::from_whole(config.blob_min_ttl as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1);

        assert!(state
            .set_account_status(
                config,
                &store,
                subscriber,
                TtlStatus::Extended,
                current_epoch
            )
            .is_ok());

        // Add another blob past the first blob's expiry
        let (hash2, size2) = new_hash(2048);
        let add2_epoch = ChainEpoch::from(config.blob_min_ttl + 11);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add2_epoch,
            hash2,
            new_metadata_hash(),
            id2.clone(),
            size2,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 2);
        assert_eq!(stats.bytes_added, size1 + size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(config.blob_min_ttl + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be refunded
            Credit::from_whole(config.blob_min_ttl as u64 * size2 - overcharge),
        );
        credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1 + size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            (token_amount.clone() * &token_credit_rate)
                - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Add the first (now expired) blob again
        let add3_epoch = ChainEpoch::from(config.blob_min_ttl + 21);
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add3_epoch,
            hash1,
            new_metadata_hash(),
            id1.clone(),
            size1,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
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
            Credit::from_whole(
                (config.blob_min_ttl - (add3_epoch - add2_epoch)) as u64 * size2
                    + config.blob_min_ttl as u64 * size1
            ),
        );
        credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size1);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1 + size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            token_amount.clone() * &token_credit_rate
                - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 2);
        assert_eq!(state.added.len(), 2);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, origin, subscriber);
        }
    }

    #[test]
    fn test_add_blob_same_hash_same_account() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        add_blob_same_hash_same_account(
            &config,
            &store,
            state,
            origin,
            origin,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_add_blob_same_hash_same_account_with_approval() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                subscriber,
                origin,
                current_epoch,
                None,
                None,
                None,
            )
            .unwrap();
        add_blob_same_hash_same_account(
            &config,
            &store,
            state,
            origin,
            subscriber,
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn add_blob_same_hash_same_account<BS: Blockstore>(
        config: &HokuConfig,
        store: &BS,
        mut state: State,
        origin: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let mut credit_amount =
            Credit::from_atto(token_amount.atto().clone()) * &config.token_credit_rate;

        assert!(state
            .set_account_status(
                config,
                &store,
                subscriber,
                TtlStatus::Extended,
                current_epoch
            )
            .is_ok());

        // Add blob with default a subscription ID
        let (hash, size) = new_hash(1024);
        let add1_epoch = current_epoch;
        let id1 = SubscriptionId::default();
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add1_epoch,
            hash,
            new_metadata_hash(),
            id1.clone(),
            size,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add1_epoch);
        assert_eq!(sub.expiry, add1_epoch + config.blob_min_ttl);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != origin {
            assert_eq!(sub.delegate, Some(origin));
        }

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
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
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Set to status pending
        let res = state.set_blob_pending(&store, subscriber, hash, id1.clone(), source);
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 1);
        assert_eq!(stats.bytes_resolving, size);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Finalize as resolved
        let finalize_epoch = ChainEpoch::from(11);
        let res = state.finalize_blob(
            config,
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
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Add the same blob again with a default subscription ID
        let add2_epoch = ChainEpoch::from(21);
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add2_epoch,
            hash,
            new_metadata_hash(),
            id1.clone(),
            size,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add1_epoch); // added should not change
        assert_eq!(sub.expiry, add2_epoch + config.blob_min_ttl);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != origin {
            assert_eq!(sub.delegate, Some(origin));
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
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= Credit::from_whole((add2_epoch - add1_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Add the same blob again but use a different subscription ID
        let add3_epoch = ChainEpoch::from(31);
        let id2 = SubscriptionId::new("foo").unwrap();
        let source = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add3_epoch,
            hash,
            new_metadata_hash(),
            id2.clone(),
            size,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());
        let (sub, _) = res.unwrap();
        assert_eq!(sub.added, add3_epoch);
        assert_eq!(sub.expiry, add3_epoch + config.blob_min_ttl);
        assert_eq!(sub.source, source);
        assert!(!sub.failed);
        if subscriber != origin {
            assert_eq!(sub.delegate, Some(origin));
        }

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
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
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= Credit::from_whole((add3_epoch - add2_epoch) as u64 * size);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size); // not changed

        // Debit all accounts
        let debit_epoch = ChainEpoch::from(41);
        let deletes_from_disc = state.debit_accounts(&store, debit_epoch).unwrap();
        assert!(deletes_from_disc.is_empty());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, debit_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            Credit::from_whole((config.blob_min_ttl - (debit_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 2);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Delete the default subscription ID
        let delete_epoch = ChainEpoch::from(51);
        let res = state.delete_blob(&store, origin, subscriber, delete_epoch, hash, id1.clone());
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
        assert_eq!(sub.expiry, add3_epoch + config.blob_min_ttl);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, delete_epoch);
        assert_eq!(
            account.credit_committed, // debit reduces this
            Credit::from_whole((config.blob_min_ttl - (delete_epoch - add3_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size); // not changed

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            (token_amount.clone() * &config.token_credit_rate)
                - (&account.credit_free + &account.credit_committed)
        );
        assert_eq!(state.capacity_used, size);

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, origin, subscriber);
        }
    }

    #[test]
    fn test_finalize_blob_from_bad_state() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, subscriber, amount.clone(), current_epoch)
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &config,
            &store,
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
            &config,
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
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, subscriber, amount.clone(), current_epoch)
            .unwrap();

        // Add a blob
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &config,
            &store,
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
            &config,
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
        assert_eq!(state.expiries.len(&store).unwrap(), 1);
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, subscriber, amount.clone(), current_epoch)
            .unwrap();
        let credit_amount = amount * &config.token_credit_rate;

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &config,
            &store,
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
            &config,
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
        assert_eq!(account.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, 0); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(state.credit_debited, Credit::from_whole(0));
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(&store).unwrap(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_finalize_blob_failed_refund() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, subscriber, amount.clone(), current_epoch)
            .unwrap();
        let mut credit_amount = amount.clone() * &config.token_credit_rate;

        assert!(state
            .set_account_status(
                &config,
                &store,
                subscriber,
                TtlStatus::Extended,
                current_epoch
            )
            .is_ok());

        // Add a blob
        let add_epoch = current_epoch;
        let (hash, size) = new_hash(1024);
        let source = new_pk();
        let res = state.add_blob(
            &config,
            &store,
            subscriber,
            subscriber,
            add_epoch,
            hash,
            new_metadata_hash(),
            SubscriptionId::default(),
            size,
            Some(config.blob_min_ttl),
            source,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(config.blob_min_ttl as u64 * size),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(state.credit_debited, Credit::from_whole(0));
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
            Credit::from_whole((config.blob_min_ttl - (debit_epoch - add_epoch)) as u64 * size),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed);
        assert_eq!(
            state.credit_debited,
            Credit::from_whole((debit_epoch - add_epoch) as u64 * size)
        );
        assert_eq!(state.capacity_used, account.capacity_used);

        // Set to status pending
        let res =
            state.set_blob_pending(&store, subscriber, hash, SubscriptionId::default(), source);
        assert!(res.is_ok());

        // Finalize as failed
        let finalize_epoch = ChainEpoch::from(21);
        let res = state.finalize_blob(
            &config,
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
        assert_eq!(account.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(
            account.credit_free,
            amount.clone() * &config.token_credit_rate
        ); // credit was refunded
        assert_eq!(account.capacity_used, 0); // capacity was released

        // Check state
        assert_eq!(state.credit_committed, Credit::from_whole(0)); // credit was released
        assert_eq!(state.credit_debited, Credit::from_whole(0)); // credit was refunded and released
        assert_eq!(state.capacity_used, 0); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(&store).unwrap(), 1); // remains until the blob is explicitly deleted
        assert_eq!(state.added.len(), 0);
        assert_eq!(state.pending.len(), 0);
    }

    #[test]
    fn test_delete_blob_refund() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(&config, &store, origin, token_amount.clone(), current_epoch)
            .unwrap();
        delete_blob_refund(
            &config,
            &store,
            state,
            origin,
            origin,
            current_epoch,
            token_amount,
            false,
        );
    }

    #[test]
    fn test_delete_blob_refund_with_approval() {
        setup_logs();
        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let origin = new_address();
        let subscriber = new_address();
        let current_epoch = ChainEpoch::from(1);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &config,
                &store,
                subscriber,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();
        state
            .approve_credit(
                &config,
                &store,
                subscriber,
                origin,
                current_epoch,
                None,
                None,
                None,
            )
            .unwrap();
        delete_blob_refund(
            &config,
            &store,
            state,
            origin,
            subscriber,
            current_epoch,
            token_amount,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_blob_refund<BS: Blockstore>(
        config: &HokuConfig,
        store: &BS,
        mut state: State,
        origin: Address,
        subscriber: Address,
        current_epoch: ChainEpoch,
        token_amount: TokenAmount,
        using_approval: bool,
    ) {
        let mut credit_amount = token_amount * &config.token_credit_rate;

        // Add a blob
        let add1_epoch = current_epoch;
        let (hash1, size1) = new_hash(1024);
        let source1 = new_pk();
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add1_epoch,
            hash1,
            new_metadata_hash(),
            SubscriptionId::default(),
            size1,
            Some(config.blob_min_ttl),
            source1,
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Finalize as resolved
        let res = state.set_blob_pending(
            &store,
            subscriber,
            hash1,
            SubscriptionId::default(),
            source1,
        );
        assert!(res.is_ok());
        let finalize_epoch = ChainEpoch::from(current_epoch + 1);
        let res = state.finalize_blob(
            config,
            &store,
            subscriber,
            finalize_epoch,
            hash1,
            SubscriptionId::default(),
            BlobStatus::Resolved,
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 1);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add1_epoch);
        assert_eq!(
            account.credit_committed,
            Credit::from_whole(config.blob_min_ttl as u64 * size1),
        );
        credit_amount -= &account.credit_committed;
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1);

        // Add another blob past the first blob expiry
        // This will trigger a debit on the account
        let add2_epoch = ChainEpoch::from(config.blob_min_ttl + 10);
        let (hash2, size2) = new_hash(2048);
        let res = state.add_blob(
            config,
            &store,
            origin,
            subscriber,
            add2_epoch,
            hash2,
            new_metadata_hash(),
            SubscriptionId::default(),
            size2,
            Some(config.blob_min_ttl),
            new_pk(),
            TokenAmount::zero(),
        );
        assert!(res.is_ok());

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
        assert_eq!(stats.num_blobs, 2);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
        assert_eq!(stats.num_added, 1);
        assert_eq!(stats.bytes_added, size2);

        // Check the account balance
        let account = state.get_account(&store, subscriber).unwrap().unwrap();
        assert_eq!(account.last_debit_epoch, add2_epoch);
        let blob1_expiry = ChainEpoch::from(config.blob_min_ttl + add1_epoch);
        let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
        assert_eq!(
            account.credit_committed, // this includes an overcharge that needs to be refunded
            Credit::from_whole(config.blob_min_ttl as u64 * size2 - overcharge),
        );
        credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size2);
        assert_eq!(account.credit_free, credit_amount);
        assert_eq!(account.capacity_used, size1 + size2);

        // Delete the first blob
        let delete_epoch = ChainEpoch::from(config.blob_min_ttl + 20);
        let delete_from_disc = state
            .delete_blob(
                &store,
                origin,
                subscriber,
                delete_epoch,
                hash1,
                SubscriptionId::default(),
            )
            .unwrap();
        assert!(delete_from_disc);

        // Check stats
        let stats = state.get_stats(config, TokenAmount::zero());
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
            Credit::from_whole(config.blob_min_ttl as u64 * size2),
        );
        assert_eq!(account.credit_free, credit_amount); // not changed
        assert_eq!(account.capacity_used, size2);

        // Check state
        assert_eq!(state.credit_committed, account.credit_committed); // credit was released
        assert_eq!(
            state.credit_debited,
            Credit::from_whole(config.blob_min_ttl as u64 * size1)
        );
        assert_eq!(state.capacity_used, size2); // capacity was released

        // Check indexes
        assert_eq!(state.expiries.len(store).unwrap(), 1);
        assert_eq!(state.added.len(), 1);
        assert_eq!(state.pending.len(), 0);

        // Check approval
        if using_approval {
            check_approval_used(&state, store, origin, subscriber);
        }
    }

    #[test]
    fn test_if_blobs_ttl_exceeds_accounts_ttl_should_error() {
        setup_logs();

        let config = HokuConfig::default();
        const YEAR: ChainEpoch = 365 * 24 * 60 * 60;

        // Test cases structure
        struct TestCase {
            name: &'static str,
            account_ttl_status: TtlStatus,
            blob_ttl: Option<ChainEpoch>,
            should_succeed: bool,
            expected_account_ttl: ChainEpoch,
            expected_blob_ttl: ChainEpoch,
        }

        // Define test cases
        let test_cases = vec![
            TestCase {
                name: "Reduced status rejects even minimum TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: Some(config.blob_min_ttl),
                should_succeed: false,
                expected_account_ttl: 0,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Reduced status rejects no TTL",
                account_ttl_status: TtlStatus::Reduced,
                blob_ttl: Some(config.blob_min_ttl),
                should_succeed: false,
                expected_account_ttl: 0,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Default status allows default TTL",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(config.blob_default_ttl),
                should_succeed: true,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: config.blob_default_ttl,
            },
            TestCase {
                name: "Default status sets no TTL to default without auto renew",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: None,
                should_succeed: true,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: config.blob_default_ttl,
            },
            TestCase {
                name: "Default status preserves given TTL if it's less than default",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(config.blob_default_ttl - 1),
                should_succeed: true,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: config.blob_default_ttl - 1,
            },
            TestCase {
                name: "Default status rejects TTLs higher than default",
                account_ttl_status: TtlStatus::Default,
                blob_ttl: Some(config.blob_default_ttl + 1),
                should_succeed: false,
                expected_account_ttl: config.blob_default_ttl,
                expected_blob_ttl: 0,
            },
            TestCase {
                name: "Extended status allows any TTL",
                account_ttl_status: TtlStatus::Extended,
                blob_ttl: Some(YEAR),
                should_succeed: true,
                expected_account_ttl: ChainEpoch::MAX,
                expected_blob_ttl: YEAR,
            },
        ];

        // Run all test cases
        for tc in test_cases {
            let config = HokuConfig::default();
            let store = MemoryBlockstore::default();
            let mut state = State::new(&store).unwrap();
            let subscriber = new_address();
            let current_epoch = ChainEpoch::from(1);
            let amount = TokenAmount::from_whole(10);

            state
                .buy_credit(&config, &store, subscriber, amount.clone(), current_epoch)
                .unwrap();
            state
                .set_account_status(
                    &config,
                    &store,
                    subscriber,
                    tc.account_ttl_status,
                    current_epoch,
                )
                .unwrap();

            let (hash, size) = new_hash(1024);
            let res = state.add_blob(
                &config,
                &store,
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

            let account_ttl = state
                .get_account_max_ttl(&config, &store, subscriber)
                .unwrap();
            assert_eq!(
                account_ttl, tc.expected_account_ttl,
                "Test case '{}' has unexpected account TTL (expected {}, got {})",
                tc.name, tc.expected_account_ttl, account_ttl
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
                        tc.blob_ttl.map_or_else(|| "none".to_string(), |ttl| ttl.to_string()), tc.account_ttl_status.get_max_ttl(config.blob_default_ttl),
                    ),
                    "Test case '{}' failed with unexpected error message",
                    tc.name
                );
            }
        }
    }

    #[test]
    fn test_set_ttl_status() {
        setup_logs();

        let config = HokuConfig::default();

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
                expected_ttl: config.blob_default_ttl,
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

            // Initialize the account if needed
            if tc.initial_ttl_status.is_some() {
                state
                    .set_account_status(
                        &config,
                        &store,
                        account,
                        tc.initial_ttl_status.unwrap(),
                        current_epoch,
                    )
                    .unwrap();
            }

            // Change TTL status
            let res = state.set_account_status(
                &config,
                &store,
                account,
                tc.new_ttl_status,
                current_epoch,
            );
            assert!(
                res.is_ok(),
                "Test case '{}' failed to set TTL status",
                tc.name
            );

            // Verify max TTL
            let max_ttl = state.get_account_max_ttl(&config, &store, account).unwrap();
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
        let config = HokuConfig::default();

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
            limit: Option<usize>, // None means process all at once
        }

        let test_cases = vec![
            TestCase {
                name: "Set to zero with Reduced status",
                account_ttl: TtlStatus::Reduced,
                expected_ttls: vec![0, 0, 0, 0, 0],
                limit: None,
            },
            TestCase {
                name: "Set to default with Default status",
                account_ttl: TtlStatus::Default,
                expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, DAY],
                limit: None,
            },
            TestCase {
                name: "Set to extended with Extended status",
                account_ttl: TtlStatus::Extended,
                expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, YEAR],
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
                .buy_credit(&config, &store, addr, token, current_epoch)
                .unwrap();

            // Set extended TTL status to allow adding all blobs
            state
                .set_account_status(&config, &store, addr, TtlStatus::Extended, current_epoch)
                .unwrap();

            // Add blobs
            let mut blob_hashes = Vec::new();
            let mut total_cost = Credit::zero();
            let mut expected_credits = Credit::zero();
            for (i, ttl) in blobs_ttls.iter().enumerate() {
                let size = (i + 1) * 1024;
                let (hash, _) = new_hash(size);
                let size = size as u64;
                let id = SubscriptionId::try_from(format!("blob-{}", i)).unwrap();
                let source = new_pk();
                blob_hashes.push(hash);

                state
                    .add_blob(
                        &config,
                        &store,
                        addr,
                        addr,
                        current_epoch,
                        hash,
                        new_metadata_hash(),
                        id.clone(),
                        size,
                        *ttl,
                        source,
                        TokenAmount::zero(),
                    )
                    .unwrap();
                state
                    .set_blob_pending(&store, addr, hash, id.clone(), source)
                    .unwrap();
                state
                    .finalize_blob(
                        &config,
                        &store,
                        addr,
                        current_epoch,
                        hash,
                        id,
                        BlobStatus::Resolved,
                    )
                    .unwrap();

                total_cost += Credit::from_whole(
                    state.get_storage_cost(ttl.unwrap_or(config.blob_default_ttl), &size),
                );
                expected_credits +=
                    Credit::from_whole(state.get_storage_cost(tc.expected_ttls[i], &size));
            }

            let account = state.get_account(&store, addr).unwrap().unwrap();
            assert_eq!(
                account.credit_committed, total_cost,
                "Test case '{}' failed: committed credits don't match",
                tc.name
            );

            state
                .set_account_status(&config, &store, addr, tc.account_ttl, current_epoch)
                .unwrap();

            let res =
                state.trim_blob_expiries(&config, &store, addr, current_epoch, None, tc.limit);
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
        let config = HokuConfig::default();

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
                    &config,
                    &store,
                    addr,
                    TokenAmount::from_whole(1000),
                    current_epoch,
                )
                .unwrap();
            state
                .set_account_status(&config, &store, addr, TtlStatus::Extended, current_epoch)
                .unwrap();

            // Add 5 blobs with different sizes to ensure different hashes
            for i in 0..5 {
                let (hash, size) = new_hash((i + 1) * 1024);
                let id = SubscriptionId::try_from(format!("blob-{}", i)).unwrap();
                let source = new_pk();
                state
                    .add_blob(
                        &config,
                        &store,
                        addr,
                        addr,
                        current_epoch,
                        hash,
                        new_metadata_hash(),
                        id.clone(),
                        size,
                        Some(7200), // 2 hours
                        source,
                        TokenAmount::zero(),
                    )
                    .unwrap();
                state
                    .set_blob_pending(&store, addr, hash, id.clone(), source)
                    .unwrap();
                state
                    .finalize_blob(
                        &config,
                        &store,
                        addr,
                        current_epoch,
                        hash,
                        id,
                        BlobStatus::Resolved,
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
                .set_account_status(&config, &store, addr, TtlStatus::Reduced, current_epoch)
                .unwrap();

            let res = state.trim_blob_expiries(
                &config,
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

        let config = HokuConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let account1 = new_address();
        let account2 = new_address();
        let current_epoch = ChainEpoch::from(1);

        // Setup accounts with credits and Extended TTL status to allow adding all blobs
        state
            .buy_credit(
                &config,
                &store,
                account1,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .buy_credit(
                &config,
                &store,
                account2,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .set_account_status(
                &config,
                &store,
                account1,
                TtlStatus::Extended,
                current_epoch,
            )
            .unwrap();
        state
            .set_account_status(
                &config,
                &store,
                account2,
                TtlStatus::Extended,
                current_epoch,
            )
            .unwrap();

        // Add blobs for both accounts
        let mut blob_hashes_account1 = Vec::new();
        let mut blob_hashes_account2 = Vec::new();
        for i in 0..3 {
            let (hash, size) = new_hash((i + 1) * 1024);
            let id = SubscriptionId::try_from(format!("blob-1-{}", i)).unwrap();
            let source = new_pk();
            blob_hashes_account1.push(hash);
            state
                .add_blob(
                    &config,
                    &store,
                    account1,
                    account1,
                    current_epoch,
                    hash,
                    new_metadata_hash(),
                    id.clone(),
                    size,
                    Some(7200), // 2 hours
                    source,
                    TokenAmount::zero(),
                )
                .unwrap();
            state
                .set_blob_pending(&store, account1, hash, id.clone(), source)
                .unwrap();
            state
                .finalize_blob(
                    &config,
                    &store,
                    account1,
                    current_epoch,
                    hash,
                    id,
                    BlobStatus::Resolved,
                )
                .unwrap();
        }
        for i in 0..3 {
            let (hash, size) = new_hash((i + 1) * 1024);
            let id = SubscriptionId::try_from(format!("blob-2-{}", i)).unwrap();
            let source = new_pk();
            blob_hashes_account2.push(hash);
            state
                .add_blob(
                    &config,
                    &store,
                    account2,
                    account2,
                    current_epoch,
                    hash,
                    new_metadata_hash(),
                    id.clone(),
                    size,
                    Some(7200), // 2 hours
                    source,
                    TokenAmount::zero(),
                )
                .unwrap();
            state
                .set_blob_pending(&store, account2, hash, id.clone(), source)
                .unwrap();
            state
                .finalize_blob(
                    &config,
                    &store,
                    account2,
                    current_epoch,
                    hash,
                    id,
                    BlobStatus::Resolved,
                )
                .unwrap();
        }

        // Change TTL status for account1 and adjust blobs
        state
            .set_account_status(&config, &store, account1, TtlStatus::Reduced, current_epoch)
            .unwrap();
        let res = state.trim_blob_expiries(&config, &store, account1, current_epoch, None, None);
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

    #[test]
    fn test_simulate_one_day() {
        setup_logs();

        let config = HokuConfig {
            blob_credit_debit_interval: ChainEpoch::from(60),
            blob_min_ttl: ChainEpoch::from(10),
            ..Default::default()
        };

        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        struct TestBlob {
            hash: Hash,
            metadata_hash: Hash,
            size: u64,
            added: Option<ChainEpoch>,
            resolve: Option<ChainEpoch>,
        }

        fn generate_test_blobs(count: i64, min_size: usize, max_size: usize) -> Vec<TestBlob> {
            let mut blobs = Vec::new();
            let mut rng = rand::thread_rng();

            for _ in 0..count {
                let size = rng.gen_range(min_size..=max_size);
                let (hash, size) = new_hash(size);
                blobs.push(TestBlob {
                    hash,
                    metadata_hash: new_metadata_hash(),
                    size,
                    added: None,
                    resolve: None,
                });
            }
            blobs
        }

        fn generate_test_users<BS: Blockstore>(
            config: &HokuConfig,
            store: &BS,
            state: &mut State,
            credit_tokens: TokenAmount,
            count: i64,
        ) -> Vec<Address> {
            let mut users = Vec::new();
            for _ in 0..count {
                let user = new_address();
                state
                    .buy_credit(config, &store, user, credit_tokens.clone(), 0)
                    .unwrap();
                users.push(user);
            }
            users
        }

        // Test params
        let epochs: i64 = 360; // num. epochs to run test for
        let user_pool_size: i64 = 10; // some may not be used, some will be used more than once
        let blob_pool_size: i64 = epochs; // some may not be used, some will be used more than once
        let min_ttl = config.blob_min_ttl;
        let max_ttl = epochs;
        let min_size = 8;
        let max_size = 1024;
        let add_intervals = [1, 2, 4, 8, 10, 12, 15, 20]; // used to add at random intervals
        let max_resolve_epochs = 30; // max num. epochs in future to resolve
        let debit_interval: i64 = config.blob_credit_debit_interval; // interval at which to debit all accounts
        let percent_fail_resolve = 0.1; // controls % of subscriptions that fail resolve

        // Set up store and state
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let mut rng = rand::thread_rng();

        // Get some users
        let credit_tokens = TokenAmount::from_whole(100); // buy a lot
        let user_credit: Credit = credit_tokens.clone() * &config.token_credit_rate;
        let users = generate_test_users(&config, &store, &mut state, credit_tokens, user_pool_size);

        // Get some blobs.
        let mut blobs = generate_test_blobs(blob_pool_size, min_size, max_size);

        // Map of resolve epochs to set of blob indexes
        #[allow(clippy::type_complexity)]
        let mut resolves: BTreeMap<
            ChainEpoch,
            HashMap<Address, HashMap<usize, (SubscriptionId, PublicKey, Credit)>>,
        > = BTreeMap::new();

        // Walk epochs.
        // We go for twice the paramaterized epochs to ensure all subscriptions can expire.
        let mut num_added = 0;
        let mut num_readded = 0;
        let mut num_resolved = 0;
        let mut num_failed = 0;
        let mut credit_used: HashMap<Address, Credit> = HashMap::new();
        for epoch in 1..=epochs * 2 {
            if epoch <= epochs {
                let add_interval = add_intervals.choose(&mut rng).unwrap().to_owned();
                if epoch % add_interval == 0 {
                    // Add a random blob with a random user
                    let blob_index = rng.gen_range(0..blobs.len());
                    let blob = unsafe { blobs.get_unchecked_mut(blob_index) };
                    if blob.added.is_none() {
                        let user_index = rng.gen_range(0..users.len());
                        let user = users[user_index];
                        let sub_id = new_subscription_id(7);
                        let ttl = rng.gen_range(min_ttl..=max_ttl);
                        let source = new_pk();
                        let res = state.add_blob(
                            &config,
                            &store,
                            user,
                            user,
                            epoch,
                            blob.hash,
                            blob.metadata_hash,
                            sub_id.clone(),
                            blob.size,
                            Some(ttl),
                            source,
                            TokenAmount::zero(),
                        );
                        assert!(res.is_ok());
                        if blob.added.is_none() {
                            num_added += 1;
                            warn!(
                                "added new blob {} at epoch {} with ttl {}",
                                blob.hash, epoch, ttl
                            );
                        } else {
                            warn!(
                                "added new sub to blob {} at epoch {} with ttl {}",
                                blob.hash, epoch, ttl
                            );
                            num_readded += 1;
                        }
                        blob.added = Some(epoch);

                        // Determine how much credit should get committed for this blob
                        let credit = Credit::from_whole(state.get_storage_cost(ttl, &blob.size));
                        // Track credit amount for user, assuming the whole committed amount gets debited
                        credit_used
                            .entry(user)
                            .and_modify(|c| c.add_assign(&credit))
                            .or_insert(credit.clone());

                        // Schedule a resolve to happen in the future
                        let resolve = rng.gen_range(1..=max_resolve_epochs) + epoch;
                        resolves
                            .entry(resolve)
                            .and_modify(|entry| {
                                entry
                                    .entry(user)
                                    .and_modify(|subs| {
                                        subs.insert(
                                            blob_index,
                                            (sub_id.clone(), source, credit.clone()),
                                        );
                                    })
                                    .or_insert(HashMap::from([(
                                        blob_index,
                                        (sub_id.clone(), source, credit.clone()),
                                    )]));
                            })
                            .or_insert(HashMap::from([(
                                user,
                                HashMap::from([(blob_index, (sub_id, source, credit))]),
                            )]));
                    }
                }
            }

            // Resolve blob(s)
            if let Some(users) = resolves.get(&epoch) {
                for (user, index) in users {
                    for (i, (sub_id, source, credit)) in index {
                        let blob = unsafe { blobs.get_unchecked(*i) };
                        let fail = rng.gen_bool(percent_fail_resolve);
                        let status = if fail {
                            num_failed += 1;
                            credit_used
                                .entry(*user)
                                .and_modify(|c| c.sub_assign(credit));
                            BlobStatus::Failed
                        } else {
                            num_resolved += 1;
                            BlobStatus::Resolved
                        };
                        // Simulate the chain putting this blob into pending state, which is
                        // required before finalization.
                        state
                            .set_blob_pending(&store, *user, blob.hash, sub_id.clone(), *source)
                            .unwrap();
                        state
                            .finalize_blob(
                                &config,
                                &store,
                                *user,
                                epoch,
                                blob.hash,
                                sub_id.clone(),
                                status,
                            )
                            .unwrap();
                    }
                }
            }

            // Every debit interval epochs we debit all acounts
            if epoch % debit_interval == 0 {
                let deletes_from_disc = state.debit_accounts(&store, epoch).unwrap();
                warn!(
                    "deleting {} blobs at epoch {}",
                    deletes_from_disc.len(),
                    epoch
                );
            }
        }

        let mut total_credit_used = Credit::zero();
        for (_, credit) in credit_used.clone() {
            total_credit_used.add_assign(&credit);
        }

        debug!("credit used: {}", total_credit_used);
        debug!("num. blobs added: {}", num_added);
        debug!("num. blobs re-added: {}", num_readded);
        debug!("num. blobs resolved: {}", num_resolved);
        debug!("num. blobs failed: {}", num_failed);

        // Check the account balances
        for (i, user) in users.iter().enumerate() {
            let account = state.get_account(&store, *user).unwrap().unwrap();
            debug!("account {}: {:#?}", i, account);
            assert_eq!(account.capacity_used, 0);
            assert_eq!(account.credit_committed, Credit::zero());
            let credit_used = credit_used.get(user).unwrap();
            assert_eq!(account.credit_free, &user_credit - credit_used);
        }

        // Check state.
        // Everything should be empty except for credit_debited.
        let stats = state.get_stats(&config, TokenAmount::zero());
        debug!("stats: {:#?}", stats);
        assert_eq!(stats.capacity_used, 0);
        assert_eq!(stats.credit_committed, Credit::zero());
        assert_eq!(stats.credit_debited, total_credit_used);
        assert_eq!(stats.num_blobs, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
    }
}
