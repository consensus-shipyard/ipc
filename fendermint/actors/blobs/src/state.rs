// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeSet, HashMap};

use anyhow::anyhow;
use cid::Cid;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use num_traits::{ToPrimitive, Zero};

use crate::GetStatsReturn;

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
    /// TODO: add list of blobs to account
    pub accounts: HashMap<Address, Account>,
    /// Map containing all blobs.
    /// TODO: After merging Iroh branch, this should be HashMap<iroh_base::Hash, Blob>
    pub blobs: HashMap<Vec<u8>, Blob>,
    /// Set of currently resolving blob hashes.
    pub resolving: BTreeSet<Vec<u8>>,
}

/// The stored representation of a credit account.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: BigInt,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: BigInt,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: BigInt,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
}

/// The stored representation of a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// TODO: add subs
    //pub subs: HashMap<Address, Subscription>,
    /// Whether the blob has been resolved.
    /// TODO: change to enum: resolving, resolved, failed
    pub resolved: bool,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Subscription {
    /// Expiry block.
    pub expiry: ChainEpoch,
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
            resolving: BTreeSet::new(),
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
            num_resolving: self.resolving.len() as u64,
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

    // TODO: check for already existing blob _for the sender_
    pub fn add_blob(
        &mut self,
        sender: Address,
        current_epoch: ChainEpoch,
        cid: Cid,
        size: u64,
        expiry: ChainEpoch,
    ) -> anyhow::Result<Account> {
        if expiry <= current_epoch {
            return Err(anyhow!("expiry must be in the future"));
        }

        match self.accounts.get_mut(&sender) {
            Some(account) => {
                // Check free credit
                let size = BigInt::from(size);
                let required_credit = (expiry as u64) * &size;
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
                let debit = (debit_blocks as u64) * &account.capacity_used;
                self.credit_debited += &debit;
                self.credit_committed -= &debit;
                account.credit_committed -= &debit;
                account.last_debit_epoch = current_epoch;

                // Account for new size and move free credit to committed credit
                self.capacity_used += &size;
                account.capacity_used += &size;
                self.credit_committed += &required_credit;
                account.credit_committed += &required_credit;
                account.credit_free -= &required_credit;

                let key = cid.to_bytes();
                self.resolving.insert(key.clone());
                self.blobs.insert(
                    key,
                    Blob {
                        size: size.to_u64().unwrap(),
                        expiry,
                        resolved: false,
                    },
                );

                Ok(account.clone())
            }
            None => Err(anyhow!("account {} not found", sender)),
        }
    }

    pub fn get_resolving_blobs(&self) -> anyhow::Result<BTreeSet<Vec<u8>>> {
        Ok(self.resolving.clone())
    }

    pub fn is_blob_resolving(&self, cid: Cid) -> anyhow::Result<bool> {
        let key = cid.to_bytes();
        let resolving = self.resolving.contains(&key);
        Ok(resolving)
    }

    // TODO: Need method for unresolving, ie, if a blob can't be fetched, the account
    // shouldn't have to pay for it since there's no way to know who's at fault (account user or too
    // many bad validators).
    pub fn resolve_blob(&mut self, cid: Cid) -> anyhow::Result<()> {
        let key = cid.to_bytes();
        self.resolving.remove(&key);
        match self.blobs.get_mut(&key) {
            Some(blob) => {
                blob.resolved = true;
                Ok(())
            }
            // Don't error here in case the key was deleted before the value was resolved.
            None => Ok(()),
        }
    }

    // TODO: Reverse accounting in add and return Account.
    // We need a syscall to delete the actual data (or at least untangle it from new data).
    pub fn delete_blob(&mut self, cid: Cid) -> anyhow::Result<()> {
        let key = cid.to_bytes();
        self.blobs.remove(&key);
        Ok(())
    }

    pub fn get_blob(&self, cid: Cid) -> anyhow::Result<Option<Blob>> {
        let key = cid.to_bytes();
        let blob = self.blobs.get(&key).cloned();
        Ok(blob)
    }
}
