// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{cell::RefCell, sync::Arc};

use anyhow::{anyhow, Context};

use cid::Cid;
use fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR;
use fendermint_vm_message::query::ActorState;
use fvm::{engine::MultiEngine, executor::ApplyRet, state_tree::StateTree};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, ActorID};
use num_traits::Zero;

use crate::fvm::{store::ReadOnlyBlockstore, FvmMessage};

use super::{FvmExecState, FvmStateParams};

/// The state over which we run queries. These can interrogate the IPLD block store or the state tree.
pub struct FvmQueryState<DB>
where
    DB: Blockstore + 'static,
{
    /// A read-only wrapper around the blockstore, to make sure we aren't
    /// accidentally committing any state. Any writes by the FVM will be
    /// buffered; as long as we don't call `flush()` we should be fine.
    store: ReadOnlyBlockstore<DB>,
    /// Multi-engine for potential message execution.
    multi_engine: Arc<MultiEngine>,
    /// Height of block at which we are executing the queries.
    block_height: ChainEpoch,
    /// State at the height we want to query.
    state_params: FvmStateParams,
    /// Lazy loaded state tree.
    state_tree: RefCell<Option<StateTree<ReadOnlyBlockstore<DB>>>>,
    /// Lazy loaded execution state.
    exec_state: RefCell<Option<FvmExecState<ReadOnlyBlockstore<DB>>>>,
}

impl<DB> FvmQueryState<DB>
where
    DB: Blockstore + Clone + 'static,
{
    pub fn new(
        blockstore: DB,
        multi_engine: Arc<MultiEngine>,
        block_height: ChainEpoch,
        state_params: FvmStateParams,
    ) -> anyhow::Result<Self> {
        // Sanity check that the blockstore contains the supplied state root.
        if !blockstore
            .has(&state_params.state_root)
            .context("failed to load state-root")?
        {
            return Err(anyhow!(
                "blockstore doesn't have the state-root {}",
                state_params.state_root
            ));
        }

        let state = Self {
            store: ReadOnlyBlockstore::new(blockstore),
            multi_engine,
            block_height,
            state_params,
            // NOTE: Not loading a state tree in case it's not needed; it would initialize the HAMT.
            state_tree: RefCell::new(None),
            exec_state: RefCell::new(None),
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

        let state_tree =
            StateTree::new_from_root(self.store.clone(), &self.state_params.state_root)?;

        let res = f(&state_tree);
        *cache = Some(state_tree);
        res
    }

    /// If we know the query is over the state, cache the state tree.
    /// If `use_cache` is enabled, the result of the execution will be
    /// buffered in the cache, if not the result is returned but not cached.
    fn with_exec_state<T, F>(&self, use_cache: bool, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut FvmExecState<ReadOnlyBlockstore<DB>>) -> anyhow::Result<T>,
    {
        let mut cache = self.exec_state.borrow_mut();
        if use_cache {
            if let Some(exec_state) = cache.as_mut() {
                return f(exec_state);
            }
        }

        let mut exec_state = FvmExecState::new(
            self.store.clone(),
            self.multi_engine.as_ref(),
            self.block_height,
            self.state_params.clone(),
        )
        .context("error creating execution state")?;

        let res = f(&mut exec_state);
        if use_cache {
            *cache = Some(exec_state);
        }
        res
    }

    /// Read a CID from the underlying IPLD store.
    pub fn store_get(&self, key: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        self.store.get(key)
    }

    /// Get the state of an actor, if it exists.
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

    /// Run a "read-only" message.
    ///
    /// The results are never going to be flushed, so it's semantically read-only,
    /// but it might write into the buffered block store the FVM creates. Running
    /// multiple such messages results in their buffered effects stacking up if
    /// `use_cache` is enabled. If `use_cache` is not enabled, the results of the
    /// execution are not cached.
    pub fn call(&self, mut msg: FvmMessage, use_cache: bool) -> anyhow::Result<ApplyRet> {
        // If the sequence is zero, treat it as a signal to use whatever is in the state.
        if msg.sequence.is_zero() {
            if let Some((_, state)) = self.actor_state(&msg.from)? {
                msg.sequence = state.sequence;
            }
        }
        self.with_exec_state(use_cache, |s| {
            if msg.from == SYSTEM_ACTOR_ADDR {
                // Explicit execution requires `from` to be an account kind.
                s.execute_implicit(msg)
            } else {
                s.execute_explicit(msg)
            }
        })
    }

    pub fn state_params(&self) -> &FvmStateParams {
        &self.state_params
    }
}
