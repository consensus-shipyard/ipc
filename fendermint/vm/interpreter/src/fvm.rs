// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::marker::PhantomData;

use anyhow::{anyhow, Context};
use async_trait::async_trait;

use cid::Cid;
use fendermint_vm_actor_interface::{cron, system};
use fvm::{
    call_manager::DefaultCallManager,
    engine::{EngineConfig, EnginePool},
    executor::{ApplyRet, DefaultExecutor, Executor},
    machine::{DefaultMachine, Machine, NetworkConfig},
    state_tree::StateTree,
    DefaultKernel,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{
    address::Address, clock::ChainEpoch, econ::TokenAmount, error::ExitCode,
    version::NetworkVersion, BLOCK_GAS_LIMIT,
};

use crate::{externs::FendermintExterns, CheckInterpreter, Interpreter, Timestamp};

pub type FvmMessage = fvm_shared::message::Message;

/// The return value extended with some things from the message that
/// might not be available to the caller, because of the message lookups
/// and transformations that happen along the way, e.g. where we need
/// a field, we might just have a CID.
pub struct FvmApplyRet {
    pub apply_ret: ApplyRet,
    pub gas_limit: u64,
}

/// Transaction check results are expressed by the exit code, so that hopefully
/// they would result in the same error code if they were applied.
pub struct FvmCheckRet {
    pub sender: Address,
    pub gas_limit: u64,
    pub exit_code: ExitCode,
}

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
}

/// Interpreter working on already verified unsigned messages.
#[derive(Clone)]
pub struct FvmMessageInterpreter<DB> {
    _phantom_db: PhantomData<DB>,
}

impl<DB> FvmMessageInterpreter<DB> {
    pub fn new() -> Self {
        Self {
            _phantom_db: PhantomData,
        }
    }
}

impl<DB> Default for FvmMessageInterpreter<DB> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<DB> Interpreter for FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync,
{
    type State = FvmState<DB>;
    type Message = FvmMessage;
    type BeginOutput = FvmApplyRet;
    type DeliverOutput = FvmApplyRet;
    type EndOutput = ();

    async fn begin(
        &self,
        mut state: Self::State,
    ) -> anyhow::Result<(Self::State, Self::BeginOutput)> {
        // Block height (FVM epoch) as sequence is intentional
        let height = state.executor.context().epoch;
        // Arbitrarily large gas limit for cron (matching how Forest does it, which matches Lotus).
        // XXX: Our blocks are not necessarily expected to be 30 seconds apart, so the gas limit might be wrong.
        let gas_limit = BLOCK_GAS_LIMIT * 10000;
        // Cron.
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: cron::CRON_ACTOR_ADDR,
            sequence: height as u64,
            gas_limit,
            method_num: cron::Method::EpochTick as u64,
            params: Default::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;

        let apply_ret =
            state
                .executor
                .execute_message(msg, fvm::executor::ApplyKind::Implicit, raw_length)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to apply block cron message: {}", err);
        }

        let ret = FvmApplyRet {
            apply_ret,
            gas_limit,
        };

        Ok((state, ret))
    }

    async fn deliver(
        &self,
        mut state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let gas_limit = msg.gas_limit;

        let apply_ret =
            state
                .executor
                .execute_message(msg, fvm::executor::ApplyKind::Explicit, raw_length)?;

        let ret = FvmApplyRet {
            apply_ret,
            gas_limit,
        };

        Ok((state, ret))
    }

    async fn end(&self, state: Self::State) -> anyhow::Result<(Self::State, Self::EndOutput)> {
        // TODO: Epoch transitions for checkpointing.
        Ok((state, ()))
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
    state_tree: StateTree<ReadOnlyBlockstore<DB>>,
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

#[async_trait]
impl<DB> CheckInterpreter for FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync,
{
    type State = FvmCheckState<DB>;
    type Message = FvmMessage;
    type Output = FvmCheckRet;

    /// Check that:
    /// * sender exists
    /// * sender nonce matches the message sequence
    /// * sender has enough funds to cover the gas cost
    async fn check(
        &self,
        mut state: Self::State,
        msg: Self::Message,
        _is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        let checked = |state, exit_code| {
            let ret = FvmCheckRet {
                sender: msg.from,
                gas_limit: msg.gas_limit,
                exit_code,
            };
            Ok((state, ret))
        };

        // NOTE: This would be a great place for let-else, but clippy runs into a compilation bug.
        if let Some(id) = state.state_tree.lookup_id(&msg.from)? {
            if let Some(mut actor) = state.state_tree.get_actor(id)? {
                let balance_needed = msg.gas_fee_cap * msg.gas_limit;
                if actor.balance < balance_needed || actor.sequence != msg.sequence {
                    return checked(state, ExitCode::SYS_SENDER_STATE_INVALID);
                } else {
                    actor.sequence += 1;
                    actor.balance -= balance_needed;
                    state.state_tree.set_actor(id, actor);
                    return checked(state, ExitCode::OK);
                }
            }
        }
        return checked(state, ExitCode::SYS_SENDER_INVALID);
    }
}
