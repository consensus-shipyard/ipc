// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{HashMap, HashSet};

use cid::Cid;
use fendermint_vm_genesis::PowerScale;
use fvm::{
    call_manager::DefaultCallManager,
    engine::MultiEngine,
    executor::{ApplyFailure, ApplyKind, ApplyRet, DefaultExecutor, Executor},
    machine::{DefaultMachine, Machine, Manifest, NetworkConfig},
    state_tree::StateTree,
    DefaultKernel,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{
    address::Address, chainid::ChainID, clock::ChainEpoch, econ::TokenAmount, error::ExitCode,
    message::Message, receipt::Receipt, version::NetworkVersion, ActorID,
};
use serde::{Deserialize, Serialize};

use crate::fvm::externs::FendermintExterns;
use fendermint_vm_core::{chainid::HasChainID, Timestamp};

pub type BlockHash = [u8; 32];

pub type ActorAddressMap = HashMap<ActorID, Address>;

/// The result of the message application bundled with any delegated addresses of event emitters.
pub type ExecResult = anyhow::Result<(ApplyRet, ActorAddressMap)>;

/// Parts of the state which evolve during the lifetime of the chain.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct FvmStateParams {
    pub state_root: Cid,
    pub timestamp: Timestamp,
    pub network_version: NetworkVersion,
    pub base_fee: TokenAmount,
    pub circ_supply: TokenAmount,
    /// The [`ChainID`] is stored here to hint at the possibility that
    /// a chain ID might change during the lifetime of a chain, in case
    /// there is a fork, or perhaps a subnet migration in IPC.
    ///
    /// How exactly that would be communicated is uknown at this point.
    pub chain_id: u64,
    pub power_scale: PowerScale,
}

pub type MachineBlockstore<DB> = <DefaultMachine<DB, FendermintExterns> as Machine>::Blockstore;

/// A state we create for the execution of all the messages in a block.
pub struct FvmExecState<DB>
where
    DB: Blockstore + 'static,
{
    executor:
        DefaultExecutor<DefaultKernel<DefaultCallManager<DefaultMachine<DB, FendermintExterns>>>>,

    /// Hash of the block currently being executed. For queries and checks this is empty.
    ///
    /// The main motivation to add it here was to make it easier to pass in data to the
    /// execution interpreter without having to add yet another piece to track at the app level.
    block_hash: Option<BlockHash>,

    /// Conversion between collateral and voting power.
    power_scale: PowerScale,
}

impl<DB> FvmExecState<DB>
where
    DB: Blockstore + 'static,
{
    /// Create a new FVM execution environment.
    ///
    /// Calling this can be very slow unless we run in `--release` mode, because the [DefaultExecutor]
    /// pre-loads builtin-actor CIDs and wasm in debug mode is slow to instrument.
    pub fn new(
        blockstore: DB,
        multi_engine: &MultiEngine,
        block_height: ChainEpoch,
        params: FvmStateParams,
    ) -> anyhow::Result<Self> {
        let mut nc = NetworkConfig::new(params.network_version);
        nc.chain_id = ChainID::from(params.chain_id);

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

        Ok(Self {
            executor,
            block_hash: None,
            power_scale: params.power_scale,
        })
    }

    /// Set the block hash during execution.
    pub fn with_block_hash(mut self, block_hash: BlockHash) -> Self {
        self.block_hash = Some(block_hash);
        self
    }

    /// Execute message implicitly.
    pub fn execute_implicit(&mut self, msg: Message) -> ExecResult {
        self.execute_message(msg, ApplyKind::Implicit)
    }

    /// Execute message explicitly.
    pub fn execute_explicit(&mut self, msg: Message) -> ExecResult {
        self.execute_message(msg, ApplyKind::Explicit)
    }

    pub fn execute_message(&mut self, msg: Message, kind: ApplyKind) -> ExecResult {
        if let Err(e) = msg.check() {
            return Ok(check_error(e));
        }

        // TODO: We could preserve the message length by changing the input type.
        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let ret = self.executor.execute_message(msg, kind, raw_length)?;
        let addrs = self.emitter_delegated_addresses(&ret)?;
        Ok((ret, addrs))
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

    /// Identity of the block being executed, if we are indeed executing any blocks.
    pub fn block_hash(&self) -> Option<BlockHash> {
        self.block_hash
    }

    /// The timestamp of the currently executing block.
    pub fn timestamp(&self) -> Timestamp {
        Timestamp(self.executor.context().timestamp)
    }

    /// Conversion between collateral and voting power.
    pub fn power_scale(&self) -> PowerScale {
        self.power_scale
    }

    /// Get a mutable reference to the underlying [StateTree].
    pub fn state_tree_mut(&mut self) -> &mut StateTree<MachineBlockstore<DB>> {
        self.executor.state_tree_mut()
    }

    /// Built-in actor manifest to inspect code CIDs.
    pub fn builtin_actors(&self) -> &Manifest {
        self.executor.builtin_actors()
    }

    /// The [ChainID] from the network configuration.
    pub fn chain_id(&self) -> ChainID {
        self.executor.context().network.chain_id
    }

    /// Collect all the event emitters' delegated addresses, for those who have any.
    fn emitter_delegated_addresses(&self, apply_ret: &ApplyRet) -> anyhow::Result<ActorAddressMap> {
        let emitter_ids = apply_ret
            .events
            .iter()
            .map(|e| e.emitter)
            .collect::<HashSet<_>>();

        let mut emitters = HashMap::default();

        for id in emitter_ids {
            if let Some(actor) = self.executor.state_tree().get_actor(id)? {
                if let Some(addr) = actor.delegated_address {
                    emitters.insert(id, addr);
                }
            }
        }

        Ok(emitters)
    }
}

impl<DB> HasChainID for FvmExecState<DB>
where
    DB: Blockstore,
{
    fn chain_id(&self) -> ChainID {
        self.executor.context().network.chain_id
    }
}

/// The FVM would return an error from `DefaultExecutor::preflight_message` if it was called
/// with a message that doesn't pass basic checks, for example it has no gas limit, as opposed
/// to returning an `ApplyRet`. This would cause our application to fail.
/// I'm not sure if it's intentional, or how Lotus handles it, it's not desireable to crash
/// because such messages can be included by malicious validators or user queries. We could
/// use ABCI++ to filter out messages from blocks, but that doesn't affect queries, so we
/// might as well encode it as an error. To keep the types simpler, let's fabricate an `ApplyRet`.
fn check_error(e: anyhow::Error) -> (ApplyRet, ActorAddressMap) {
    let zero = TokenAmount::from_atto(0);
    let ret = ApplyRet {
        msg_receipt: Receipt {
            exit_code: ExitCode::SYS_ASSERTION_FAILED,
            return_data: RawBytes::default(),
            gas_used: 0,
            events_root: None,
        },
        penalty: zero.clone(),
        miner_tip: zero.clone(),
        base_fee_burn: zero.clone(),
        over_estimation_burn: zero.clone(),
        refund: zero,
        gas_refund: 0,
        gas_burned: 0,
        failure_info: Some(ApplyFailure::PreValidation(format!("{:#}", e))),
        exec_trace: Vec::new(),
        events: Vec::new(),
    };
    (ret, Default::default())
}
