// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::credit::CreditApproval;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use recall_ipld::{hamt, hamt::map::TrackedFlushResult};

/// HAMT wrapper tracking [`CreditApproval`]s by account address.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Approvals {
    /// The HAMT root.
    pub root: hamt::Root<Address, CreditApproval>,
    /// The size of the collection.
    size: u64,
}

impl Approvals {
    /// Returns a approval collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, CreditApproval>::new(store, "credit_approvals")?;
        Ok(Self { root, size: 0 })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, CreditApproval>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<Address, CreditApproval>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    /// The size of the collection.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
