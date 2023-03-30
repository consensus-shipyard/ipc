// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};

use cid::Cid;
use fvm::state_tree::StateTree;
use fvm_ipld_blockstore::Blockstore;

use crate::fvm::store::ReadOnlyBlockstore;

/// A state we create for the execution of all the messages in a block.
pub struct FvmCheckState<DB>
where
    DB: Blockstore + 'static,
{
    pub state_tree: StateTree<ReadOnlyBlockstore<DB>>,
}

impl<DB> FvmCheckState<DB>
where
    DB: Blockstore + 'static,
{
    pub fn new(blockstore: DB, state_root: Cid) -> anyhow::Result<Self> {
        // Sanity check that the blockstore contains the supplied state root.
        if !blockstore
            .has(&state_root)
            .context("failed to load initial state-root")?
        {
            return Err(anyhow!(
                "blockstore doesn't have the initial state-root {}",
                state_root
            ));
        }

        // Create a new state tree from the supplied root.
        let state_tree = {
            let bstore = ReadOnlyBlockstore::new(blockstore);
            StateTree::new_from_root(bstore, &state_root)?
        };

        let state = Self { state_tree };

        Ok(state)
    }
}
