// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{cell::RefCell, sync::Arc};

use anyhow::{anyhow, Context};

use cid::Cid;
use fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR;
use fendermint_vm_core::chainid::HasChainID;
use fendermint_vm_message::query::ActorState;
use fvm::state_tree::StateTree;
use fvm::{engine::MultiEngine, executor::ApplyRet};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, chainid::ChainID, clock::ChainEpoch, ActorID};
use num_traits::Zero;

use crate::fvm::state::FvmCheckState;
use crate::fvm::{store::ReadOnlyBlockstore, FvmMessage};

use super::{FvmExecState, FvmStateParams};

type CheckState<DB> = Arc<tokio::sync::Mutex<Option<FvmCheckState<DB>>>>;

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
    /// Lazy loaded execution state.
    exec_state: RefCell<Option<FvmExecState<ReadOnlyBlockstore<DB>>>>,
    /// Lazy locked check state.
    check_state: CheckState<DB>,
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
        check_state: CheckState<DB>,
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
            exec_state: RefCell::new(None),
            check_state,
        };

        Ok(state)
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
    pub fn actor_state(
        &self,
        use_cache: bool,
        addr: &Address,
    ) -> anyhow::Result<Option<(ActorID, ActorState)>> {
        self.with_exec_state(use_cache, |exec_state| {
            let state_tree = exec_state.state_tree_mut();
            get_actor_state(state_tree, addr)
        })
    }

    /// Get the pending state of an actor, which is what we use for checking transaction submissions
    /// and gets reset after reach block commit.
    ///
    /// This is very similar to the `actor_state` method, but works on the `check_state`,
    /// so that we can take transactions in the mempool into account. The drawback is that
    /// it involves locking a single instance of an in-memory `StateTree` (it cannot be
    /// cloned or shared between threads), and therefore we should only use it if we really
    /// need to.
    ///
    /// If there is no check state (because nobody sent a transaction), fall back on actor state.
    pub async fn pending_state(
        // Consumed because `.lock().await` would not work if this was just a reference,
        // since `FvmQueryState` is not `Sync`.
        self,
        addr: &Address,
    ) -> anyhow::Result<(Self, Option<(ActorID, ActorState)>)> {
        // Release the lock ASAP.
        let state = {
            let mut guard = self.check_state.lock().await;

            if let Some(check_state) = guard.as_mut() {
                get_actor_state(check_state.state_tree_mut(), addr).map(Some)?
            } else {
                None
            }
        };

        let ret = if let Some(ret) = state {
            ret
        } else {
            self.actor_state(false, addr)?
        };

        Ok((self, ret))
    }

    /// Run a "read-only" message.
    ///
    /// The results are never going to be flushed, so it's semantically read-only,
    /// but it might write into the buffered block store the FVM creates. Running
    /// multiple such messages results in their buffered effects stacking up if
    /// `use_cache` is enabled. If `use_cache` is not enabled, the results of the
    /// execution are not cached.
    pub fn call(&self, mut msg: FvmMessage, use_cache: bool) -> anyhow::Result<ApplyRet> {
        self.with_exec_state(use_cache, |s| {
            // If the sequence is zero, treat it as a signal to use whatever is in the state.
            if msg.sequence.is_zero() {
                let state_tree = s.state_tree_mut();
                if let Some(id) = state_tree.lookup_id(&msg.from)? {
                    state_tree.get_actor(id)?.map(|st| {
                        msg.sequence = st.sequence;
                        st
                    });
                }
            }
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

impl<DB> HasChainID for FvmQueryState<DB>
where
    DB: Blockstore + 'static,
{
    fn chain_id(&self) -> ChainID {
        ChainID::from(self.state_params.chain_id)
    }
}

fn get_actor_state<DB>(
    state_tree: &StateTree<DB>,
    addr: &Address,
) -> anyhow::Result<Option<(ActorID, ActorState)>>
where
    DB: Blockstore,
{
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
}
