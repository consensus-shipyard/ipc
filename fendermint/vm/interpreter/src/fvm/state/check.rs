// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};

use cid::Cid;
use fvm::state_tree::StateTree;
use fvm_ipld_blockstore::Blockstore;

use super::ReadOnlyBlockstore;

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
    pub fn new(blockstore: DB, initial_state_root: Cid) -> anyhow::Result<Self> {
        // Sanity check that the blockstore contains the supplied state root.
        if !blockstore
            .has(&initial_state_root)
            .context("failed to load initial state-root")?
        {
            return Err(anyhow!(
                "blockstore doesn't have the initial state-root {}",
                initial_state_root
            ));
        }

        // Create a new state tree from the supplied root.
        let state_tree = {
            let bstore = ReadOnlyBlockstore(blockstore);
            StateTree::new_from_root(bstore, &initial_state_root)?
        };

        let state = Self { state_tree };

        Ok(state)
    }
}
