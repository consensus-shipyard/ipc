// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::anyhow;
use fendermint_actor_blobs_shared::params::GetStatsReturn;
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, Hash, PublicKey, Subscription,
};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use num_traits::{ToPrimitive, Zero};

const MIN_TTL: ChainEpoch = 3600; // one hour

/// The state represents all accounts and stored blobs.
/// TODO: use raw HAMTs
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total free storage capacity of the subnet.
    pub capacity_free: BigInt,
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
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: BTreeMap<Hash, HashSet<(Address, PublicKey)>>,
}

impl State {
    pub fn new(capacity: u64, credit_debit_rate: u64) -> anyhow::Result<Self> {
        Ok(Self {
            capacity_free: BigInt::from(capacity),
            capacity_used: BigInt::zero(),
            credit_sold: BigInt::zero(),
            credit_committed: BigInt::zero(),
            credit_debited: BigInt::zero(),
            credit_debit_rate,
            accounts: HashMap::new(),
            blobs: HashMap::new(),
            pending: BTreeMap::new(),
        })
    }

    pub fn get_stats(&self, balance: TokenAmount) -> anyhow::Result<GetStatsReturn> {
        Ok(GetStatsReturn {
            balance,
            capacity_free: self.capacity_free.clone(),
            capacity_used: self.capacity_used.clone(),
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            credit_debit_rate: self.credit_debit_rate,
            num_accounts: self.accounts.len() as u64,
            num_blobs: self.blobs.len() as u64,
            num_resolving: self.pending.len() as u64,
        })
    }

    pub fn buy_credit(
        &mut self,
        address: Address,
        amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<Account> {
        let credits = self.credit_debit_rate * amount.atto();

        // Don't sell credits if we're at storage capacity
        // TODO: This should be more nuanced, i.e., pick some min block duration and storage amount
        // at which to stop selling credits. Say there's only 1 byte of capcity left,
        // we don't want to sell a bunch of credits even though they could be used if the account
        // wants to store 1 byte at a time, which is unlikely :)
        if self.capacity_used == self.capacity_free {
            return Err(anyhow!("credits not available (subnet has reach capacity)"));
        }
        self.credit_sold += &credits;

        match self.accounts.get_mut(&address) {
            Some(account) => {
                account.credit_free += &credits;
                Ok(account.clone())
            }
            None => {
                let account = Account {
                    capacity_used: BigInt::zero(),
                    credit_free: credits.clone(),
                    credit_committed: BigInt::zero(),
                    last_debit_epoch: current_epoch,
                };
                self.accounts.insert(address, account.clone());
                Ok(account)
            }
        }
    }

    pub fn get_account(&self, address: Address) -> anyhow::Result<Option<Account>> {
        let account = self.accounts.get(&address).cloned();
        Ok(account)
    }

    // TODO: expiry should be optional, ie, pay for as long as there's credit, but we have to
    // keep some continuous amount of committed credit, say one month?
    // TODO: we are currenlty charging for block bytes during blob resolution. we should
    // probably only charge for blocks after the blob is resolved.
    pub fn add_blob(
        &mut self,
        sender: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        size: u64,
        ttl: ChainEpoch,
        source: PublicKey,
    ) -> anyhow::Result<Account> {
        if ttl < MIN_TTL {
            return Err(anyhow!("minimum blob TTL is {}", MIN_TTL));
        }
        let expiry = current_epoch + ttl;

        match self.accounts.get_mut(&sender) {
            Some(account) => {
                let size = BigInt::from(size);
                // Capacity updates and required credit depend on whether the sender is already
                // subcribing to this blob
                let mut new_capacity = BigInt::zero();
                let mut new_account_capacity = BigInt::zero();
                let required_credit: BigInt;
                let (blob, pending) = if let Some(blob) = self.blobs.get(&hash) {
                    // We could get_mut to begin with, but the logic below is simpler if we
                    // have transactional control
                    let mut blob = blob.clone();
                    if let Some(sub) = blob.subs.get(&sender) {
                        let mut sub = sub.clone();
                        // Required credit can be negative if sender is reducing expiry
                        required_credit = (expiry - sub.expiry) as u64 * &size;
                        sub.expiry = expiry;
                        // Overwrite source allows sender to retry resolving
                        sub.source = source;
                    } else {
                        // One or more accounts have already committed credit.
                        // However, we still need to reserve the full required credit from the new
                        // subscriber, as the existing account(s) may decide to change the
                        // expiry or cancel.
                        required_credit = ttl as u64 * &size;
                        new_account_capacity = size.clone();
                        // Add new subscription
                        blob.subs.insert(sender, Subscription { expiry, source });
                    }
                    let pending = match blob.status {
                        BlobStatus::Added(_) | BlobStatus::Failed => {
                            // It's pending or failed, reset with current epoch
                            blob.status = BlobStatus::Added(current_epoch);
                            true
                        }
                        BlobStatus::Resolved => false,
                    };
                    (blob, pending)
                } else {
                    required_credit = ttl as u64 * &size;
                    new_capacity = size.clone();
                    new_account_capacity = size.clone();
                    // Create new blob
                    (
                        Blob {
                            size: size.to_u64().unwrap(),
                            subs: HashMap::from([(sender, Subscription { expiry, source })]),
                            status: BlobStatus::Added(current_epoch),
                        },
                        true,
                    )
                };

                if account.credit_free < required_credit {
                    return Err(anyhow!(
                        "account {} has insufficient credit (available: {}; required: {})",
                        sender,
                        account.credit_free,
                        required_credit
                    ));
                }

                // Debit for existing usage
                let debit_blocks = current_epoch - account.last_debit_epoch;
                let debit = debit_blocks as u64 * &account.capacity_used;
                self.credit_debited += &debit;
                self.credit_committed -= &debit;
                account.credit_committed -= &debit;
                account.last_debit_epoch = current_epoch;

                // Account for new size and move free credit to committed credit
                self.capacity_used += &new_capacity;
                account.capacity_used += &new_account_capacity;
                self.credit_committed += &required_credit;
                account.credit_committed += &required_credit;
                account.credit_free -= &required_credit;

                // Add/update pending with hash and its source
                if pending {
                    self.pending
                        .entry(hash)
                        .and_modify(|sources| {
                            sources.insert((sender, source));
                        })
                        .or_insert(HashSet::from([(sender, source)]));
                }

                // Add/update blob
                self.blobs.insert(hash, blob);

                Ok(account.clone())
            }
            None => Err(anyhow!("account {} not found", sender)),
        }
    }

    pub fn get_blob(&self, hash: Hash) -> anyhow::Result<Option<Blob>> {
        let blob = self.blobs.get(&hash).cloned();
        Ok(blob)
    }

    pub fn get_pending_blobs(
        &self,
    ) -> anyhow::Result<BTreeMap<Hash, HashSet<(Address, PublicKey)>>> {
        Ok(self.pending.clone())
    }

    pub fn finalize_blob(
        &mut self,
        from: Address,
        hash: Hash,
        status: BlobStatus,
    ) -> anyhow::Result<()> {
        if matches!(status, BlobStatus::Added(_)) {
            return Err(anyhow!(
                "finalized status of blob {} must be 'resolved' or 'failed'",
                hash
            ));
        }
        if let Some(blob) = self.blobs.get_mut(&hash) {
            match blob.status {
                BlobStatus::Added(added_epoch) => {
                    match self.accounts.get_mut(&from) {
                        Some(account) => {
                            if let Some(sub) = blob.subs.get(&from) {
                                if matches!(status, BlobStatus::Failed) {
                                    let size = BigInt::from(blob.size);
                                    // We're not going to make a debit, but we need to refund
                                    // any spent credits that may have been used on this
                                    // blob in the event the last debit is later than the
                                    // added epoch.
                                    if account.last_debit_epoch > added_epoch {
                                        let refund_blocks = account.last_debit_epoch - added_epoch;
                                        let refund = refund_blocks as u64 * &size;
                                        account.credit_free += &refund; // re-mint spent credit
                                        self.credit_debited -= &refund;
                                    }

                                    // Account for reclaimed size and move committed credit to
                                    // free credit
                                    self.capacity_used -= &size;
                                    account.capacity_used -= &size;
                                    if sub.expiry > account.last_debit_epoch {
                                        let reclaim_credit =
                                            (sub.expiry - account.last_debit_epoch) * &size;
                                        self.credit_committed -= &reclaim_credit;
                                        account.credit_committed -= &reclaim_credit;
                                        account.credit_free += &reclaim_credit;
                                    }

                                    // Delete subscription
                                    blob.subs.remove(&from);
                                }
                            } else {
                                return Err(anyhow!(
                                    "finalizing address {} is not subscribed to blob {}",
                                    from,
                                    hash
                                ));
                            }
                        }
                        None => return Err(anyhow!("account {} not found", from)),
                    }
                    blob.status = status;
                }
                BlobStatus::Resolved | BlobStatus::Failed => {
                    // No-op, already finalized
                }
            }
        }
        // Remove from pending
        self.pending.remove(&hash);
        Ok(())
    }

    pub fn delete_blob(
        &mut self,
        sender: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
    ) -> anyhow::Result<(Account, bool)> {
        match self.accounts.get_mut(&sender) {
            Some(account) => {
                let debit_epoch: ChainEpoch;
                let mut reclaim_capacity = BigInt::zero();
                let mut reclaim_account_capacity = BigInt::zero();
                let mut reclaim_credit = BigInt::zero();
                let delete_blob = if let Some(blob) = self.blobs.get(&hash) {
                    // We could get_mut to begin with, but the logic below is simpler if we
                    // have transactional control
                    let mut blob = blob.clone();
                    if let Some(sub) = blob.subs.get(&sender) {
                        let size = BigInt::from(blob.size);
                        // Since the charge will be for all the account's blobs, we can only
                        // account for capacity up to _this_ blob's expiry if it is less than
                        // the current epoch.
                        debit_epoch = sub.expiry.min(current_epoch);
                        // Determine if the account needs a capacity reduction
                        match blob.status {
                            BlobStatus::Added(_) | BlobStatus::Resolved => {
                                reclaim_account_capacity = size.clone();
                                if blob.subs.is_empty() {
                                    reclaim_capacity = size.clone();
                                }
                                // We can refund credits if expiry is in the future
                                if debit_epoch == current_epoch {
                                    reclaim_credit = (sub.expiry - debit_epoch) * &size;
                                }
                            }
                            BlobStatus::Failed => {
                                // No-op, capacity and committed credits have already been returned
                            }
                        }
                        // Clean up state
                        blob.subs.remove(&sender);
                        if blob.subs.is_empty() {
                            self.blobs.remove(&hash);
                            true
                        } else {
                            self.blobs.insert(hash, blob);
                            false
                        }
                    } else {
                        return Err(anyhow!(
                            "sender {} is not subscribed to blob {}",
                            sender,
                            hash
                        ));
                    }
                } else {
                    return Err(anyhow!("blob {} not found", hash));
                };

                // Debit for existing usage.
                // It could be possible that debit epoch is less than the last debit,
                // in which case we don't need to do anything.
                if debit_epoch > account.last_debit_epoch {
                    let debit_blocks = debit_epoch - account.last_debit_epoch;
                    let debit = debit_blocks as u64 * &account.capacity_used;
                    self.credit_debited += &debit;
                    self.credit_committed -= &debit;
                    account.credit_committed -= &debit;
                    account.last_debit_epoch = debit_epoch;
                }

                // Account for reclaimed size and move committed credit to free credit
                self.capacity_used -= &reclaim_capacity;
                account.capacity_used -= &reclaim_account_capacity;
                self.credit_committed -= &reclaim_credit;
                account.credit_committed -= &reclaim_credit;
                account.credit_free += &reclaim_credit;

                // Maybe remove hash and its source from pending
                if delete_blob {
                    self.pending.remove(&hash);
                }

                Ok((account.clone(), delete_blob))
            }
            None => Err(anyhow!("account {} not found", sender)),
        }
    }
}
