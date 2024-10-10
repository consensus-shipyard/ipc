// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod staking;

use anyhow::Context;
use async_trait::async_trait;
fendermint_actor_gas_market_eip1559::{Reading, SetConstants};
use fendermint_contract_test::Tester;
use fendermint_crypto::{PublicKey, SecretKey};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::gas_market::GAS_MARKET_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fendermint_vm_core::Timestamp;
use fendermint_vm_genesis::{Account, Actor, ActorMeta, Genesis, PermissionMode, SignerAddr};
use fendermint_vm_interpreter::fvm::gas_market::GasMarket;
use fendermint_vm_interpreter::fvm::state::FvmExecState;
use fendermint_vm_interpreter::fvm::store::memory::MemoryBlockstore;
use fendermint_vm_interpreter::fvm::upgrades::{Upgrade, UpgradeScheduler};
use fendermint_vm_interpreter::fvm::FvmMessageInterpreter;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::version::NetworkVersion;
use lazy_static::lazy_static;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tendermint_rpc::Client;

lazy_static! {
    static ref ADDR: Address =
        Address::new_secp256k1(&my_secret_key().public_key().serialize()).unwrap();
    static ref ADDR2: Address =
        Address::new_secp256k1(&my_secret_key().public_key().serialize()).unwrap();
}
const CHAIN_NAME: &str = "mychain";
type I = FvmMessageInterpreter<MemoryBlockstore, NeverCallClient>;

// returns a seeded secret key which is guaranteed to be the same every time
fn my_secret_key() -> SecretKey {
    SecretKey::random(&mut StdRng::seed_from_u64(123))
}

/// Creates a default tester with validator public key
async fn default_tester() -> (Tester<I>, PublicKey) {
    tester_with_upgrader(UpgradeScheduler::new()).await
}

/// Creates a default tester with validator public key
async fn tester_with_upgrader(
    upgrade_scheduler: UpgradeScheduler<MemoryBlockstore>,
) -> (Tester<I>, PublicKey) {
    let validator = my_secret_key().public_key();

    let interpreter: FvmMessageInterpreter<MemoryBlockstore, _> =
        FvmMessageInterpreter::new(NeverCallClient, None, 1.05, 1.05, false, upgrade_scheduler);

    let genesis = Genesis {
        chain_name: CHAIN_NAME.to_string(),
        timestamp: Timestamp(0),
        network_version: NetworkVersion::V21,
        base_fee: TokenAmount::zero(),
        power_scale: 0,
        validators: Vec::new(),
        accounts: vec![
            Actor {
                meta: ActorMeta::Account(Account {
                    owner: SignerAddr(*ADDR),
                }),
                balance: TokenAmount::from_whole(100),
            },
            Actor {
                meta: ActorMeta::Account(Account {
                    owner: SignerAddr(*ADDR2),
                }),
                balance: TokenAmount::from_whole(10),
            },
        ],
        eam_permission_mode: PermissionMode::Unrestricted,
        ipc: None,
    };
    (Tester::new(interpreter, genesis).await.unwrap(), validator)
}

#[tokio::test]
async fn test_gas_market_base_fee_oscillation() {
    let (mut tester, _) = default_tester().await;

    let num_msgs = 10;
    let total_gas_limit = 6178630;
    let base_gas_limit = total_gas_limit / num_msgs;

    let mut gas_constants = SetConstants::default();
    gas_constants.block_gas_limit = total_gas_limit;

    let messages = (0..num_msgs)
        .map(|i| Message {
            version: 0,
            from: ADDR.clone(),
            to: Address::new_id(10),
            sequence: i,
            value: TokenAmount::from_atto(1),
            method_num: 0,
            params: Default::default(),
            gas_limit: base_gas_limit,
            gas_fee_cap: Default::default(),
            gas_premium: TokenAmount::from_atto(1),
        })
        .collect::<Vec<Message>>();

    // iterate over all the upgrades
    let height = 1;
    tester.begin_block(height).await.unwrap();
    tester
        .modify_exec_state(|mut state| async {
            state.gas_market_mut().set_constants(gas_constants);
            Ok((state, ()))
        })
        .await
        .unwrap();
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();

    let height = 2;
    tester.begin_block(height).await.unwrap();
    let before_reading = tester
        .modify_exec_state(|mut state| async {
            let reading = current_reading(&mut state, height)?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    tester.execute_msgs(messages).await.unwrap();
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();

    let height = 3;
    tester.begin_block(height).await.unwrap();
    let post_full_block_reading = tester
        .modify_exec_state(|mut state| async {
            let reading = current_reading(&mut state, height)?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();
    assert!(
        before_reading.base_fee < post_full_block_reading.base_fee,
        "base fee should have increased"
    );

    let height = 4;
    tester.begin_block(height).await.unwrap();
    let post_empty_block_reading = tester
        .modify_exec_state(|mut state| async {
            let reading = current_reading(&mut state, height)?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();
    assert!(
        post_empty_block_reading.base_fee < post_full_block_reading.base_fee,
        "base fee should have decreased"
    );
}

#[tokio::test]
async fn test_gas_market_premium_distribution() {
    let (mut tester, validator) = default_tester().await;
    let evm_address = Address::from(EthAddress::new_secp256k1(&validator.serialize()).unwrap());

    let num_msgs = 10;
    let total_gas_limit = 62306300;
    let premium = 1;
    let base_gas_limit = total_gas_limit / num_msgs;

    let messages = (0..num_msgs)
        .map(|i| Message {
            version: 0,
            from: ADDR.clone(),
            to: ADDR2.clone(),
            sequence: i,
            value: TokenAmount::from_atto(1),
            method_num: 0,
            params: Default::default(),
            gas_limit: base_gas_limit,
            gas_fee_cap: TokenAmount::from_atto(base_gas_limit),
            gas_premium: TokenAmount::from_atto(premium),
        })
        .collect::<Vec<Message>>();

    // iterate over all the upgrades
    let height = 1;
    tester
        .begin_block_with_validator(height, Some(validator))
        .await
        .unwrap();
    let initial_balance = tester
        .modify_exec_state(|state| async {
            let tree = state.state_tree();
            let balance = tree
                .get_actor_by_address(&evm_address)?
                .map(|v| v.balance)
                .unwrap_or(TokenAmount::zero());
            Ok((state, balance))
        })
        .await
        .unwrap();
    assert_eq!(initial_balance, TokenAmount::zero());

    tester.execute_msgs(messages).await.unwrap();
    tester.end_block(height).await.unwrap();
    let final_balance = tester
        .modify_exec_state(|state| async {
            let tree = state.state_tree();
            let balance = tree
                .get_actor_by_address(&evm_address)?
                .map(|v| v.balance)
                .unwrap_or(TokenAmount::zero());
            Ok((state, balance))
        })
        .await
        .unwrap();
    tester.commit().await.unwrap();

    assert!(
        final_balance > initial_balance,
        "validator balance should have increased"
    )
}

#[tokio::test]
async fn test_gas_market_upgrade() {
    let mut upgrader = UpgradeScheduler::new();

    let total_gas_limit = 100;
    upgrader
        .add(
            Upgrade::new(CHAIN_NAME, 1, Some(1), |state| {
                println!(
                    "[Upgrade at height {}] Update gas market params",
                    state.block_height()
                );

                let mut gas_constants = SetConstants::default();
                gas_constants.block_gas_limit = 100;

                state.gas_market_mut().set_constants(gas_constants);

                Ok(())
            })
            .unwrap(),
        )
        .unwrap();

    let (mut tester, _) = tester_with_upgrader(upgrader).await;

    let height = 1;
    tester.begin_block(height).await.unwrap();
    let reading = tester
        .modify_exec_state(|mut state| async {
            let reading = current_reading(&mut state, height)?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    assert_ne!(
        reading.block_gas_limit, total_gas_limit,
        "gas limit should not equal at start"
    );
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();

    let height = 2;
    tester.begin_block(height).await.unwrap();
    let reading = tester
        .modify_exec_state(|mut state| async {
            let reading = current_reading(&mut state, height)?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    assert_eq!(
        reading.block_gas_limit, total_gas_limit,
        "gas limit should equal after upgrade"
    );
}

pub fn current_reading(
    state: &mut FvmExecState<MemoryBlockstore>,
    block_height: ChainEpoch,
) -> anyhow::Result<Reading> {
    let msg = Message {
        from: system::SYSTEM_ACTOR_ADDR,
        to: GAS_MARKET_ACTOR_ADDR,
        sequence: block_height as u64,
        // exclude this from gas restriction
        gas_limit: i64::MAX as u64,
        method_num: fendermint_actor_gas_market::Method::CurrentReading as u64,
        params: fvm_ipld_encoding::RawBytes::default(),
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let (apply_ret, _) = state.execute_implicit(msg)?;

    if let Some(err) = apply_ret.failure_info {
        anyhow::bail!("failed to read gas market state: {}", err);
    }

    let r = fvm_ipld_encoding::from_slice::<Reading>(&apply_ret.msg_receipt.return_data)
        .context("failed to parse gas market readying")?;
    Ok(r)
}

#[derive(Clone)]
struct NeverCallClient;

#[async_trait]
impl Client for NeverCallClient {
    async fn perform<R>(&self, _request: R) -> Result<R::Output, tendermint_rpc::Error>
    where
        R: tendermint_rpc::SimpleRequest,
    {
        todo!()
    }
}
