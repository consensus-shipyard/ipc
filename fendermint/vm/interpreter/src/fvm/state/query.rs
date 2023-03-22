// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::cell::RefCell;

use anyhow::{anyhow, Context};

use cid::Cid;
use fendermint_vm_message::query::ActorState;
use fvm::state_tree::StateTree;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, ActorID};

use super::ReadOnlyBlockstore;

/// The state over which we run queries. These can interrogate the IPLD block store or the state tree.
pub struct FvmQueryState<DB>
where
    DB: Blockstore + 'static,
{
    store: ReadOnlyBlockstore<DB>,
    state_root: Cid,
    state_tree: RefCell<Option<StateTree<ReadOnlyBlockstore<DB>>>>,
}

impl<DB> FvmQueryState<DB>
where
    DB: Blockstore + Clone + 'static,
{
    pub fn new(blockstore: DB, state_root: Cid) -> anyhow::Result<Self> {
        // Sanity check that the blockstore contains the supplied state root.
        if !blockstore
            .has(&state_root)
            .context("failed to load state-root")?
        {
            return Err(anyhow!(
                "blockstore doesn't have the state-root {}",
                state_root
            ));
        }

        let state = Self {
            store: ReadOnlyBlockstore(blockstore),
            state_root,
            // NOTE: Not loading a state tree in case it's not needed; it would initialize the HAMT.
            state_tree: RefCell::new(None),
        };

        Ok(state)
    }

    /// If we know the query is over the state, cache the state tree.
    fn with_state_tree<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&StateTree<ReadOnlyBlockstore<DB>>) -> anyhow::Result<T>,
    {
        let mut cache = self.state_tree.borrow_mut();
        if let Some(state_tree) = cache.as_ref() {
            return f(state_tree);
        }
        let state_tree = StateTree::new_from_root(self.store.clone(), &self.state_root)?;
        let res = f(&state_tree);
        *cache = Some(state_tree);
        res
    }

    /// Read a CID from the underlying IPLD store.
    pub fn store_get(&self, key: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        self.store.get(key)
    }

    pub fn actor_state(&self, addr: &Address) -> anyhow::Result<Option<(ActorID, ActorState)>> {
        self.with_state_tree(|state_tree| {
            if let Some(id) = state_tree.lookup_id(addr)? {
                Ok(state_tree.get_actor(id)?.map(|st| {
                    let st = ActorState {
                        code: st.code,
                        state: st.state,
                        sequence: st.sequence,
                        balance: st.balance,
                        delegated_address: st.delegated_address,
                    };
                    (id, st)
                }))
            } else {
                Ok(None)
            }
        })
    }
}
