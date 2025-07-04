// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod staking;

use std::sync::Arc;

use async_trait::async_trait;
use fendermint_actor_gas_market_eip1559::Constants;
use fendermint_contract_test::Tester;
use fendermint_crypto::{PublicKey, SecretKey};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::gas_market::GAS_MARKET_ACTOR_ADDR;
use fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR;
use fendermint_vm_core::Timestamp;
use fendermint_vm_genesis::{Account, Actor, ActorMeta, Genesis, PermissionMode, SignerAddr};
use fendermint_vm_interpreter::fvm::bottomup::BottomUpManager;
use fendermint_vm_interpreter::fvm::store::memory::MemoryBlockstore;
use fendermint_vm_interpreter::fvm::topdown::TopDownManager;
use fendermint_vm_interpreter::fvm::upgrades::{Upgrade, UpgradeScheduler};
use fendermint_vm_interpreter::fvm::FvmMessagesInterpreter;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::conv::from_fvm;
use fendermint_vm_message::signed::SignedMessage;
use fendermint_vm_topdown::voting::VoteTally;
use fendermint_vm_topdown::Toggle;
use fvm_shared::chainid::ChainID;

use fvm::executor::{ApplyKind, Executor};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use fvm_shared::version::NetworkVersion;
use lazy_static::lazy_static;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tendermint_rpc::Client;

lazy_static! {
    static ref SECRET: SecretKey = rand_secret_key();
    static ref SECRET2: SecretKey = rand_secret_key();
    static ref ADDR: Address = Address::new_secp256k1(&SECRET.public_key().serialize()).unwrap();
    static ref ADDR2: Address = Address::new_secp256k1(&SECRET2.public_key().serialize()).unwrap();
}

const CHAIN_NAME: &str = "mychain";
type I = FvmMessagesInterpreter<MemoryBlockstore, NeverCallClient>;

// returns a seeded secret key which is guaranteed to be the same every time
fn rand_secret_key() -> SecretKey {
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
    let validator = rand_secret_key().public_key();

    let bottom_up_manager = BottomUpManager::new(NeverCallClient, None);
    let finality_provider = Arc::new(Toggle::disabled());
    let vote_tally = VoteTally::empty();
    let top_down_manager = TopDownManager::new(finality_provider, vote_tally);

    let interpreter: FvmMessagesInterpreter<MemoryBlockstore, _> = FvmMessagesInterpreter::new(
        bottom_up_manager,
        top_down_manager,
        upgrade_scheduler,
        false,
        200,
        1.05,
        1.05,
    );

    // We use a random actor ID for ipc_contracts_owner as it's not used for
    // anything in this test. This is important because we want to make sure
    // that the actor ID isn't accidentally used as a valid actor ID in the
    // test.
    let ipc_contracts_owner = Address::new_id(10000);
    let ipc_contracts_owner = from_fvm::to_eth_address(&ipc_contracts_owner)
        .unwrap()
        .unwrap();

    let genesis = Genesis {
        chain_name: CHAIN_NAME.to_string(),
        chain_id: None,
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
        ipc_contracts_owner,
    };
    (Tester::new(interpreter, genesis).await.unwrap(), validator)
}

#[tokio::test]
async fn test_gas_market_base_fee_oscillation() {
    let (mut tester, _) = default_tester().await;

    let chain_id = ChainID::from(tester.state_params().chain_id);
    let num_msgs = 10;
    let block_gas_limit = 6178630;
    let base_gas_limit = block_gas_limit / num_msgs;

    let messages = (0..num_msgs)
        .map(|i| {
            let msg = Message {
                version: 0,
                from: *ADDR,
                to: Address::new_id(10),
                sequence: i,
                value: TokenAmount::from_atto(1),
                method_num: 0,
                params: Default::default(),
                gas_limit: base_gas_limit,
                gas_fee_cap: Default::default(),
                gas_premium: TokenAmount::from_atto(1),
            };
            ChainMessage::Signed(SignedMessage::new_secp256k1(msg, &SECRET, &chain_id).unwrap())
        })
        .collect::<Vec<ChainMessage>>();

    let producer = rand_secret_key().public_key();

    // block 1: set the gas constants
    let height = 1;
    tester.begin_block(height, producer).await.unwrap();

    tester
        .modify_exec_state(|mut state| async {
            state.execute_with_executor(|executor| {
                // cannot capture updated_block_gas_limit due to Upgrade::new wanting a fn pointer.
                let msg = custom_gas_limit(block_gas_limit);
                executor.execute_message(msg, ApplyKind::Implicit, 0)?;
                Ok(())
            })?;

            Ok((state, ()))
        })
        .await
        .unwrap();

    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();

    //
    let height = 2;
    tester.begin_block(height, producer).await.unwrap();
    let before_reading = tester
        .modify_exec_state(|mut state| async {
            let reading = state.read_gas_market()?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    tester.execute_msgs(messages).await.unwrap();
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();

    let height = 3;
    tester.begin_block(height, producer).await.unwrap();
    let post_full_block_reading = tester
        .modify_exec_state(|mut state| async {
            let reading = state.read_gas_market()?;
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
    tester.begin_block(height, producer).await.unwrap();
    let post_empty_block_reading = tester
        .modify_exec_state(|mut state| async {
            let reading = state.read_gas_market()?;
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

    let chain_id = ChainID::from(tester.state_params().chain_id);
    let num_msgs = 10;
    let total_gas_limit = 62306300;
    let premium = 1;
    let base_gas_limit = total_gas_limit / num_msgs;

    let messages = (0..num_msgs)
        .map(|i| {
            let msg = Message {
                version: 0,
                from: *ADDR,
                to: *ADDR2,
                sequence: i,
                value: TokenAmount::from_atto(1),
                method_num: 0,
                params: Default::default(),
                gas_limit: base_gas_limit,
                gas_fee_cap: TokenAmount::from_atto(base_gas_limit),
                gas_premium: TokenAmount::from_atto(premium),
            };
            ChainMessage::Signed(SignedMessage::new_secp256k1(msg, &SECRET, &chain_id).unwrap())
        })
        .collect::<Vec<ChainMessage>>();

    let proposer = rand_secret_key().public_key();

    // iterate over all the upgrades
    let height = 1;
    tester.begin_block(height, proposer).await.unwrap();
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

    // Initial block gas limit is determined by the default constants.
    let initial_block_gas_limit = Constants::default().block_gas_limit;
    let updated_block_gas_limit = 200;

    // Attach an upgrade at epoch 2 that changes the gas limit to 200.
    upgrader
        .add(
            Upgrade::new(CHAIN_NAME, 2, Some(1), move |state| {
                println!(
                    "[Upgrade at height {}] Update gas market params",
                    state.block_height()
                );
                state.execute_with_executor(|executor| {
                    // cannot capture updated_block_gas_limit due to Upgrade::new wanting a fn pointer.
                    let msg = custom_gas_limit(200);
                    executor.execute_message(msg, ApplyKind::Implicit, 0)?;
                    Ok(())
                })
            })
            .unwrap(),
        )
        .unwrap();

    // Create a new tester with the upgrader attached.
    let (mut tester, _) = tester_with_upgrader(upgrader).await;

    let producer = rand_secret_key().public_key();

    // At height 1, simply read the block gas limit and ensure it's the default.
    let height = 1;
    tester.begin_block(height, producer).await.unwrap();
    let reading = tester
        .modify_exec_state(|mut state| async {
            let reading = state.read_gas_market()?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    assert_eq!(
        reading.block_gas_limit, initial_block_gas_limit,
        "block gas limit should be the default as per constants"
    );
    tester.end_block(height).await.unwrap();
    tester.commit().await.unwrap();

    // The upgrade above should have updated the gas limit to 200.
    let height = 2;
    tester.begin_block(height, producer).await.unwrap();
    let reading = tester
        .modify_exec_state(|mut state| async {
            let reading = state.read_gas_market()?;
            Ok((state, reading))
        })
        .await
        .unwrap();
    assert_eq!(
        reading.block_gas_limit, updated_block_gas_limit,
        "gas limit post-upgrade should be {updated_block_gas_limit}"
    );
}

fn custom_gas_limit(block_gas_limit: u64) -> Message {
    let gas_constants = fendermint_actor_gas_market_eip1559::SetConstants {
        block_gas_limit,
        ..Default::default()
    };

    Message {
        version: 0,
        from: SYSTEM_ACTOR_ADDR,
        to: GAS_MARKET_ACTOR_ADDR,
        sequence: 0,
        value: Default::default(),
        method_num: fendermint_actor_gas_market_eip1559::Method::SetConstants as u64,
        params: RawBytes::serialize(&gas_constants).unwrap(),
        gas_limit: 10000000,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    }
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
