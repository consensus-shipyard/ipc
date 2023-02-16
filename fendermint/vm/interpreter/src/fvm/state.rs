// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};

use cid::Cid;
use fvm::{
    call_manager::DefaultCallManager,
    engine::{EngineConfig, EnginePool},
    executor::{ApplyKind, ApplyRet, DefaultExecutor, Executor},
    machine::{DefaultMachine, Machine, NetworkConfig},
    state_tree::StateTree,
    DefaultKernel,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount, message::Message, version::NetworkVersion};

use crate::Timestamp;

use super::externs::FendermintExterns;

/// A state we create for the execution of all the messages in a block.
pub struct FvmState<DB>
where
    DB: Blockstore + 'static,
{
    executor:
        DefaultExecutor<DefaultKernel<DefaultCallManager<DefaultMachine<DB, FendermintExterns>>>>,
}

impl<DB> FvmState<DB>
where
    DB: Blockstore + 'static,
{
    pub fn new(
        blockstore: DB,
        block_height: ChainEpoch,
        block_timestamp: Timestamp,
        network_version: NetworkVersion,
        initial_state_root: Cid,
        base_fee: TokenAmount,
        circ_supply: TokenAmount,
    ) -> anyhow::Result<Self> {
        let nc = NetworkConfig::new(network_version);

        // TODO: Configure:
        // * circ_supply; by default it's for Filecoin
        // * base_fee; by default it's zero
        let mut mc = nc.for_epoch(block_height, block_timestamp.0, initial_state_root);
        mc.set_base_fee(base_fee);
        mc.set_circulating_supply(circ_supply);

        let ec = EngineConfig::from(&nc);
        let engine = EnginePool::new_default(ec)?;
        let machine = DefaultMachine::new(&mc, blockstore, FendermintExterns)?;
        let executor = DefaultExecutor::new(engine, machine)?;

        Ok(Self { executor })
    }

    /// Execute message implicitly.
    pub fn execute_implicit(&mut self, msg: Message) -> anyhow::Result<ApplyRet> {
        self.execute_message(msg, ApplyKind::Implicit)
    }

    /// Execute message explicitly.
    pub fn execute_explicit(&mut self, msg: Message) -> anyhow::Result<ApplyRet> {
        self.execute_message(msg, ApplyKind::Explicit)
    }

    pub fn execute_message(&mut self, msg: Message, kind: ApplyKind) -> anyhow::Result<ApplyRet> {
        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        self.executor.execute_message(msg, kind, raw_length)
    }

    /// Commit the state. It must not fail, but we're returning a result so that error
    /// handling can be done in the application root.
    ///
    /// For now this is not part of the `Interpreter` because it's not clear what atomic
    /// semantics we can hope to provide if the middlewares call each other: did it go
    /// all the way down, or did it stop somewhere? Easier to have one commit of the state
    /// as a whole.
    pub fn commit(mut self) -> anyhow::Result<Cid> {
        self.executor.flush()
    }

    /// The currently executing block height.
    pub fn block_height(&self) -> ChainEpoch {
        self.executor.context().epoch
    }
}

pub struct ReadOnlyBlockstore<DB>(DB);

impl<DB> Blockstore for ReadOnlyBlockstore<DB>
where
    DB: Blockstore,
{
    fn get(&self, k: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        self.0.get(k)
    }

    fn put_keyed(&self, _k: &Cid, _block: &[u8]) -> anyhow::Result<()> {
        panic!("never intended to use put on the read-only blockstore")
    }
}

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

        let state = FvmCheckState { state_tree };

        Ok(state)
    }
}
