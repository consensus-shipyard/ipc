// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_vm_genesis::Genesis;
use fvm::state_tree::StateTree;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::state::StateTreeVersion;

/// A state we create for the execution of genesis initialisation.
pub struct FvmGenesisState<DB>
where
    DB: Blockstore + 'static,
{
    pub state_tree: StateTree<DB>,
}

impl<DB> FvmGenesisState<DB>
where
    DB: Blockstore + 'static,
{
    pub fn new(blockstore: DB) -> anyhow::Result<Self> {
        // Create an empty state tree.
        let state_tree = StateTree::new(blockstore, StateTreeVersion::V5)?;
        let state = Self { state_tree };
        Ok(state)
    }

    /// Initialize actor states from the Genesis spec.
    pub fn create_genesis_actors(&mut self, _genesis: &Genesis) {
        todo!()
    }

    /// Flush the data to the block store.
    pub fn commit(mut self) -> anyhow::Result<Cid> {
        let root = self.state_tree.flush()?;
        Ok(root)
    }
}
