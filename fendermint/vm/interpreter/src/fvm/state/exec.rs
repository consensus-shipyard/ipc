// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm::{
    call_manager::DefaultCallManager,
    engine::MultiEngine,
    executor::{ApplyKind, ApplyRet, DefaultExecutor, Executor},
    machine::{DefaultMachine, Machine, NetworkConfig},
    DefaultKernel,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount, message::Message, version::NetworkVersion};
use serde::{Deserialize, Serialize};

use crate::fvm::externs::FendermintExterns;
use crate::Timestamp;

/// Parts of the state which evolve during the lifetime of the chain.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FvmStateParams {
    pub state_root: Cid,
    pub timestamp: Timestamp,
    pub network_version: NetworkVersion,
    pub base_fee: TokenAmount,
    pub circ_supply: TokenAmount,
}

/// A state we create for the execution of all the messages in a block.
pub struct FvmExecState<DB>
where
    DB: Blockstore + 'static,
{
    executor:
        DefaultExecutor<DefaultKernel<DefaultCallManager<DefaultMachine<DB, FendermintExterns>>>>,
}

impl<DB> FvmExecState<DB>
where
    DB: Blockstore + 'static,
{
    pub fn new(
        blockstore: DB,
        multi_engine: &MultiEngine,
        block_height: ChainEpoch,
        params: FvmStateParams,
    ) -> anyhow::Result<Self> {
        let nc = NetworkConfig::new(params.network_version);

        // TODO: Configure:
        // * circ_supply; by default it's for Filecoin
        // * base_fee; by default it's zero
        let mut mc = nc.for_epoch(block_height, params.timestamp.0, params.state_root);
        mc.set_base_fee(params.base_fee);
        mc.set_circulating_supply(params.circ_supply);

        // Creating a new machine every time is prohibitively slow.
        // let ec = EngineConfig::from(&nc);
        // let engine = EnginePool::new_default(ec)?;

        let engine = multi_engine.get(&nc)?;
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
        // TODO: We could preserve the message length by changing the input type.
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

    /// The height of the currently executing block.
    pub fn block_height(&self) -> ChainEpoch {
        self.executor.context().epoch
    }

    /// The timestamp of the currently executing block.
    pub fn timestamp(&self) -> Timestamp {
        Timestamp(self.executor.context().timestamp)
    }
}
