// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod staking;

use anyhow::{Context, Result};
use cid::Cid;
use ethers::types::{H160, U256};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use ethers::contract::abigen;
use fvm::engine::MultiEngine;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::version::NetworkVersion;
use tendermint_rpc::HttpClient;

use fendermint_crypto::SecretKey;
use fendermint_vm_actor_interface::eam;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_core::Timestamp;
use fendermint_vm_genesis::{Account, Actor, ActorMeta, Genesis, PermissionMode, SignerAddr};
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
use fendermint_vm_message::chain::ChainMessage;

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

    async fn init(&mut self, genesis: Genesis) -> anyhow::Result<()> {
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
        // TODO: generate block hash based on input
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

const CONTRACT_HEX: &str = include_str!("../../contracts/SimpleCoin.bin");
abigen!(SimpleCoin, "../contracts/SimpleCoin.abi");

fn eth_addr_to_h160(eth_addr: &EthAddress) -> H160 {
    ethers::core::types::Address::from_slice(&eth_addr.0)
}

// returns a seeded secret key which is guaranteed to be the same every time
fn my_secret_key() -> SecretKey {
    SecretKey::random(&mut StdRng::seed_from_u64(123))
}

#[tokio::test]
async fn testest() {
    use bytes::Bytes;
    use fendermint_rpc::message::{GasParams, MessageFactory};
    use lazy_static::lazy_static;

    lazy_static! {
        /// Default gas params based on the testkit.
        static ref GAS_PARAMS: GasParams = GasParams {
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::default(),
            gas_premium: TokenAmount::default(),
        };
    }

    let mut upgrade_scheduler = UpgradeScheduler::new();
    upgrade_scheduler
        .add(
            Upgrade::new("mychain".to_string(), 1, |state| {
                println!(
                    "!!! Running migration at height {}: Deploy simple contract",
                    state.block_height()
                );

                // create a message for deploying the contract
                let mut mf = MessageFactory::new_secp256k1(my_secret_key(), 1, state.chain_id());
                let message = match mf
                    .fevm_create(
                        Bytes::from(
                            hex::decode(CONTRACT_HEX)
                                .context("error parsing contract")
                                .unwrap(),
                        ),
                        Bytes::default(),
                        TokenAmount::default(),
                        GAS_PARAMS.clone(),
                    )
                    .unwrap()
                {
                    ChainMessage::Signed(signed) => signed.into_message(),
                    _ => panic!("unexpected message type"),
                };

                // execute the message
                let (apply_ret, _) = state.execute_implicit(message).unwrap();
                println!("Execute message returned: {:?}", apply_ret);
                if let Some(err) = apply_ret.failure_info {
                    anyhow::bail!("failed to deploy contract: {}", err);
                }

                // parse the return value
                let res = fvm_ipld_encoding::from_slice::<eam::CreateReturn>(
                    &apply_ret.msg_receipt.return_data,
                )
                .map_err(|e| panic!("error parsing as CreateReturn: {e}"))
                .unwrap();

                let contract_delegated_addr = res.delegated_address();
                println!("Contract delegated_address: {}", contract_delegated_addr);
                let contract_robus_addr = res.delegated_address();
                println!("Contract robust_address: {}", contract_robus_addr);

                Ok(())
            })
            .unwrap(),
        )
        .unwrap();

    upgrade_scheduler
        .add(
            Upgrade::new("mychain".to_string(), 2, |state| {
                println!(
                    "!!! Running migration at height {}: Sends a balance",
                    state.block_height()
                );

                let contract_delegated_address =
                    Address::from_str("f410fnz5jdky3zzcj6pejqkomkggw72pcuvkpihz2rwa").unwrap();

                // build the calldata for the send_coin function
                let (client, _mock) = ethers::providers::Provider::mocked();
                let calldata = SimpleCoin::new(EthAddress::from_id(101), client.into())
                    .send_coin(
                        // the address we are sending the balance to (which is us in this case)
                        eth_addr_to_h160(&EthAddress::from(my_secret_key().public_key())),
                        // the amount we are sending
                        U256::from(1000),
                    )
                    .calldata()
                    .expect("calldata should contain function and parameters");

                // create a message for sending the balance
                let mut mf = MessageFactory::new_secp256k1(my_secret_key(), 1, state.chain_id());
                let message = match mf
                    .fevm_invoke(
                        contract_delegated_address,
                        calldata.0,
                        TokenAmount::default(),
                        GAS_PARAMS.clone(),
                    )
                    .unwrap()
                {
                    ChainMessage::Signed(signed) => signed.into_message(),
                    _ => panic!("unexpected message type"),
                };

                // execute the message
                let (apply_ret, _) = state.execute_implicit(message).unwrap();
                println!("Ret apply_ret : {:?}", apply_ret);

                if let Some(err) = apply_ret.failure_info {
                    anyhow::bail!("failed to send balance: {}", err);
                }

                Ok(())
            })
            .unwrap(),
        )
        .unwrap();

    upgrade_scheduler
        .add(
            Upgrade::new("mychain".to_string(), 3, |state| {
                println!(
                    "!!! Running migration at height {}: Returns a balance",
                    state.block_height()
                );

                let contract_delegated_address =
                    Address::from_str("f410fnz5jdky3zzcj6pejqkomkggw72pcuvkpihz2rwa").unwrap();

                // build the calldata for the get_balance function
                let (client, _mock) = ethers::providers::Provider::mocked();
                let calldata = SimpleCoin::new(EthAddress::from_id(0), client.into())
                    .get_balance(eth_addr_to_h160(&EthAddress::from(
                        my_secret_key().public_key(),
                    )))
                    .calldata()
                    .expect("calldata should contain function and parameters");

                let mut mf = MessageFactory::new_secp256k1(my_secret_key(), 1, state.chain_id());
                let message = match mf
                    .fevm_invoke(
                        contract_delegated_address,
                        calldata.0,
                        TokenAmount::default(),
                        GAS_PARAMS.clone(),
                    )
                    .unwrap()
                {
                    ChainMessage::Signed(signed) => signed.into_message(),
                    _ => panic!("unexpected message type"),
                };

                let (apply_ret, _) = state.execute_implicit(message).unwrap();
                println!("Ret apply_ret : {:?}", apply_ret);

                if let Some(err) = apply_ret.failure_info {
                    anyhow::bail!("failed to get balance: {}", err);
                }

                Ok(())
            })
            .unwrap(),
        )
        .unwrap();

    let interpreter: FvmMessageInterpreter<MemoryBlockstore, HttpClient> =
        FvmMessageInterpreter::new(
            tendermint_rpc::HttpClient::new("http://127.0.0.1:26657").unwrap(),
            None,
            contracts_path(),
            1.05,
            1.05,
            false,
            upgrade_scheduler,
        );

    let mut tester = Tester::new(interpreter, MemoryBlockstore::new());

    // include test actor with some balance
    let actor = Actor {
        meta: ActorMeta::Account(Account {
            owner: SignerAddr(
                Address::new_secp256k1(&my_secret_key().public_key().serialize()).unwrap(),
            ),
        }),
        balance: TokenAmount::from_atto(1000),
    };

    let genesis = Genesis {
        chain_name: "mychain".to_string(),
        timestamp: Timestamp(0),
        network_version: NetworkVersion::V21,
        base_fee: TokenAmount::zero(),
        power_scale: 0,
        validators: Vec::new(),
        accounts: vec![actor],
        eam_permission_mode: PermissionMode::Unrestricted,
        ipc: None,
    };

    tester.init(genesis).await.unwrap();

    // iterate over all the upgrades
    for block_height in 1..=3 {
        tester.begin_block(block_height).await.unwrap();
        tester.end_block(block_height).await.unwrap();
        tester.commit().await.unwrap();
    }
}
