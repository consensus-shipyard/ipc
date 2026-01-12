// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::GetStatsReturn;
use fendermint_actor_ipc_storage_config_shared::IPCStorageConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::econ::TokenAmount;

pub mod accounts;
pub mod blobs;
pub mod credit;
pub mod operators;

use accounts::Accounts;
use blobs::{Blobs, DeleteBlobStateParams};
use credit::Credits;
use operators::Operators;

/// The state represents all accounts and stored blobs.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// Struct containing credit-related state.
    pub credits: Credits,
    /// HAMT containing all accounts keyed by actor ID address.
    pub accounts: Accounts,
    /// HAMT containing all blobs keyed by blob hash.
    pub blobs: Blobs,
    /// Registry of node operators for blob storage.
    pub operators: Operators,
}

impl State {
    /// Creates a new [`State`].
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        Ok(Self {
            credits: Credits::default(),
            accounts: Accounts::new(store)?,
            blobs: Blobs::new(store)?,
            operators: Operators::new(store)?,
        })
    }

    /// Returns stats about the current actor state.
    pub fn get_stats(&self, config: &IPCStorageConfig, balance: TokenAmount) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(config.blob_capacity),
            capacity_used: self.blobs.bytes_size(),
            credit_sold: self.credits.credit_sold.clone(),
            credit_committed: self.credits.credit_committed.clone(),
            credit_debited: self.credits.credit_debited.clone(),
            token_credit_rate: config.token_credit_rate.clone(),
            num_accounts: self.accounts.len(),
            num_blobs: self.blobs.len(),
            num_added: self.blobs.added.len(),
            bytes_added: self.blobs.added.bytes_size(),
            num_resolving: self.blobs.pending.len(),
            bytes_resolving: self.blobs.pending.bytes_size(),
        }
    }
}
