// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod staking;

use anyhow::{Context, Result};
use cid::Cid;
use std::future::Future;
use std::sync::Arc;
use tendermint_rpc::HttpClient;

use fvm::engine::MultiEngine;
use fvm_shared::bigint::Zero;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::version::NetworkVersion;

use fendermint_vm_actor_interface::system;
use fendermint_vm_core::Timestamp;
use fendermint_vm_genesis::{Genesis, PermissionMode};
use fendermint_vm_interpreter::fvm::bundle::{bundle_path, custom_actors_bundle_path};
use fendermint_vm_interpreter::fvm::state::{
    FvmExecState, FvmGenesisState, FvmStateParams, FvmUpdatableParams,
};
use fendermint_vm_interpreter::fvm::store::memory::MemoryBlockstore;
use fendermint_vm_interpreter::fvm::upgrade_scheduler::UpgradeScheduler;
use fendermint_vm_interpreter::fvm::{FvmApplyRet, FvmGenesisOutput, FvmMessage, PowerUpdates};
use fendermint_vm_interpreter::GenesisInterpreter;
use fendermint_vm_interpreter::{
    fvm::{bundle::contracts_path, upgrade_scheduler::Upgrade, FvmMessageInterpreter},
    ExecInterpreter,
};

#[derive(Clone)]
struct Tester<I> {
    interpreter: Arc<I>,
    state_store: Arc<MemoryBlockstore>,
    multi_engine: Arc<MultiEngine>,
    exec_state: Arc<tokio::sync::Mutex<Option<FvmExecState<MemoryBlockstore>>>>,
    state_params: FvmStateParams,
}

impl<I> Tester<I>
where
    I: GenesisInterpreter<
        State = FvmGenesisState<MemoryBlockstore>,
        Genesis = Genesis,
        Output = FvmGenesisOutput,
    >,
    I: ExecInterpreter<
        State = FvmExecState<MemoryBlockstore>,
        Message = FvmMessage,
        BeginOutput = FvmApplyRet,
        DeliverOutput = FvmApplyRet,
        EndOutput = PowerUpdates,
    >,
{
    fn state_store_clone(&self) -> MemoryBlockstore {
        self.state_store.as_ref().clone()
    }

    pub fn new(interpreter: I, state_store: MemoryBlockstore) -> Self {
        Self {
            interpreter: Arc::new(interpreter),
            state_store: Arc::new(state_store),
            multi_engine: Arc::new(MultiEngine::new(1)),
            exec_state: Arc::new(tokio::sync::Mutex::new(None)),
            state_params: FvmStateParams {
                timestamp: Timestamp(0),
                state_root: Cid::default(),
                network_version: NetworkVersion::V21,
                base_fee: TokenAmount::zero(),
                circ_supply: TokenAmount::zero(),
                chain_id: 0,
                power_scale: 0,
            },
        }
    }

    async fn init_genesis(&mut self) -> anyhow::Result<()> {
        let bundle_path = bundle_path();
        let bundle = std::fs::read(&bundle_path)
            .with_context(|| format!("failed to read bundle: {}", bundle_path.to_string_lossy()))?;

        let custom_actors_bundle_path = custom_actors_bundle_path();
        let custom_actors_bundle =
            std::fs::read(&custom_actors_bundle_path).with_context(|| {
                format!(
                    "failed to read custom actors_bundle: {}",
                    custom_actors_bundle_path.to_string_lossy()
                )
            })?;

        let state = FvmGenesisState::new(
            self.state_store_clone(),
            self.multi_engine.clone(),
            &bundle,
            &custom_actors_bundle,
        )
        .await
        .context("failed to create genesis state")?;

        let genesis = Genesis {
            chain_name: "test".to_string(),
            timestamp: Timestamp(0),
            network_version: NetworkVersion::V21,
            base_fee: TokenAmount::zero(),
            power_scale: 0,
            validators: Vec::new(),
            accounts: Vec::new(),
            eam_permission_mode: PermissionMode::Unrestricted,
            ipc: None,
        };

        let (state, out) = self
            .interpreter
            .init(state, genesis)
            .await
            .context("failed to init from genesis")?;

        let state_root = state.commit().context("failed to commit genesis state")?;

        self.state_params = FvmStateParams {
            state_root,
            timestamp: out.timestamp,
            network_version: out.network_version,
            base_fee: out.base_fee,
            circ_supply: out.circ_supply,
            chain_id: out.chain_id.into(),
            power_scale: out.power_scale,
        };

        Ok(())
    }

    /// Take the execution state, update it, put it back, return the output.
    async fn modify_exec_state<T, F, R>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(FvmExecState<MemoryBlockstore>) -> R,
        R: Future<Output = Result<(FvmExecState<MemoryBlockstore>, T)>>,
    {
        let mut guard = self.exec_state.lock().await;
        let state = guard.take().expect("exec state empty");

        let (state, ret) = f(state).await?;

        *guard = Some(state);

        Ok(ret)
    }

    /// Put the execution state during block execution. Has to be empty.
    async fn put_exec_state(&self, state: FvmExecState<MemoryBlockstore>) {
        let mut guard = self.exec_state.lock().await;
        assert!(guard.is_none(), "exec state not empty");
        *guard = Some(state);
    }

    /// Take the execution state during block execution. Has to be non-empty.
    async fn take_exec_state(&self) -> FvmExecState<MemoryBlockstore> {
        let mut guard = self.exec_state.lock().await;
        guard.take().expect("exec state empty")
    }

    async fn begin_block(&self, block_height: ChainEpoch) -> Result<()> {
        // generate a random block hash
        let block_hash: [u8; 32] = [0; 32];

        let db = self.state_store.as_ref().clone();
        let mut state_params = self.state_params.clone();
        state_params.timestamp = Timestamp(block_height as u64);

        let state = FvmExecState::new(db, self.multi_engine.as_ref(), block_height, state_params)
            .context("error creating new state")?
            .with_block_hash(block_hash);

        self.put_exec_state(state).await;

        let res = self
            .modify_exec_state(|s| self.interpreter.begin(s))
            .await
            .unwrap();
        println!("Ret begin apply_ret : {:?}", res.apply_ret);

        Ok(())
    }

    async fn end_block(&self, _block_height: ChainEpoch) -> Result<()> {
        let _ret = self
            .modify_exec_state(|s| self.interpreter.end(s))
            .await
            .context("end failed")?;

        Ok(())
    }

    async fn commit(&mut self) -> Result<()> {
        let exec_state = self.take_exec_state().await;

        let (
            state_root,
            FvmUpdatableParams {
                power_scale,
                circ_supply,
            },
            _,
        ) = exec_state.commit().context("failed to commit FVM")?;

        self.state_params.state_root = state_root;
        self.state_params.power_scale = power_scale;
        self.state_params.circ_supply = circ_supply;

        Ok(())
    }
}

#[test]
fn testest() {
    let rt: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new().expect("create tokio runtime for init");

    let mut us = UpgradeScheduler::default();

    let chain_id = 1942764459484029;

    us.add(Upgrade {
        chain_id,
        block_height: 1,
        migration: |state| {
            println!(
                "Migration at height {} just prints out chainmetadata actor",
                state.block_height()
            );

            let actor = state
                .state_tree_mut()
                .get_actor(system::SYSTEM_ACTOR_ID)
                .unwrap();
            print!("chainmetadata actor: {:?}", actor);
            Ok(())
        },
    })
    .unwrap();

    us.add(Upgrade {
        chain_id,
        block_height: 2,
        migration: |state| {
            println!(
                "Migration at height {} deletes chainmetadata actor",
                state.block_height()
            );

            state.state_tree_mut().delete_actor(system::SYSTEM_ACTOR_ID);

            Ok(())
        },
    })
    .unwrap();

    us.add(Upgrade {
        chain_id,
        block_height: 3,
        migration: |state| {
            println!(
                "Migration at height {} confirms the chainmetadata actor is deleted",
                state.block_height()
            );

            let actor = state
                .state_tree_mut()
                .get_actor(system::SYSTEM_ACTOR_ID)
                .unwrap();
            print!("chainmetadata actor after delete: {:?}", actor);
            Ok(())
        },
    })
    .unwrap();

    let interpreter: FvmMessageInterpreter<MemoryBlockstore, HttpClient> =
        FvmMessageInterpreter::new(
            tendermint_rpc::HttpClient::new("http://127.0.0.1:26657").unwrap(),
            None,
            contracts_path(),
            1.05,
            1.05,
            false,
            Some(us),
        );

    let mut tester = Tester::new(interpreter, MemoryBlockstore::new());
    rt.block_on(tester.init_genesis()).unwrap();

    // iterate over 3 blocks
    for block_height in 1..=3 {
        rt.block_on(tester.begin_block(block_height)).unwrap();
        rt.block_on(tester.end_block(block_height)).unwrap();
        rt.block_on(tester.commit()).unwrap();
    }
}
