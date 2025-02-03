// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::Account;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use recall_ipld::hamt;
use recall_ipld::hamt::map::TrackedFlushResult;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AccountsState {
    pub root: hamt::Root<Address, Account>,
    size: u64,
}

impl AccountsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, Account>::new(store, "accounts")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Address, Account>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Address, Account>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}
