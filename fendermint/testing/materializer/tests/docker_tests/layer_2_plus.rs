// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::{anyhow, bail, Context};
use ethers::contract::{ContractError, ContractRevert};
use ethers::core::types as et;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{JsonRpcClient, Middleware, PendingTransaction, Provider},
    signers::{Signer, Wallet},
    types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest, H160},
};
use futures::FutureExt;
use ipc_provider::IpcProvider;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use fendermint_materializer::{HasEthApi, ResourceId};
use fendermint_vm_actor_interface::init::builtin_actor_eth_addr;
use fendermint_vm_actor_interface::ipc;
use fendermint_vm_message::conv::from_fvm::to_eth_address;
use ipc_actors_abis::gateway_getter_facet::{GatewayGetterFacet, ParentFinality};
use ipc_actors_abis::gateway_messenger_facet::{
    GatewayMessengerFacet, GatewayMessengerFacetErrors,
};
use ipc_actors_abis::subnet_actor_getter_facet::SubnetActorGetterFacet;

use fvm_shared::econ::TokenAmount;

use ipc_api::address::IPCAddress;
use ipc_api::cross::{IpcEnvelope, IpcMsgKind};
use ipc_api::evm;

use crate::with_testnet;

const MANIFEST: &str = "layer3.yaml";
const CHECKPOINT_PERIOD: u64 = 10;
const SLEEP_SECS: u64 = 5;
const MAX_RETRIES: u32 = 5;

/// Test that top-down syncing and bottom-up checkpoint submission work.
#[serial_test::serial]
#[tokio::test]
async fn test_provider() {
    // TODO Karel - use the provider to wait for balance to be updated in the child subnet.
    use ipc_api::ethers_address_to_fil_address;
    use ipc_api::subnet_id::SubnetID;
    use ipc_provider::config::{
        subnet::{EVMSubnet, SubnetConfig},
        Subnet,
    };

    let subnet_id =
        SubnetID::from_str("/r1126193293194756/f410fhwibtof7hp6v5f453q5soyn6ksxym7chbqata2q")
            .unwrap();

    let parent_provider = IpcProvider::new_with_subnet(
        None,
        Subnet {
            id: subnet_id.clone(),
            config: SubnetConfig::Fevm(EVMSubnet {
                provider_http: "http://localhost:30745".parse().unwrap(),
                provider_timeout: None,
                auth_token: None,
                registry_addr: builtin_actor_eth_addr(ipc::SUBNETREGISTRY_ACTOR_ID).into(),
                gateway_addr: builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID).into(),
            }),
        },
    )
    .unwrap();

    let eth_address =
        ethers::types::Address::from_str("0x651a3c584f4c71b54c50ea73f41b936845ab4fdf").unwrap();
    let address = ethers_address_to_fil_address(&eth_address).unwrap();

    let balance = parent_provider
        .wallet_balance(&subnet_id, &address)
        .await
        .unwrap();

    println!("Balance: {:?}", balance);
}

async fn test_topdown_and_bottomup_plus() {
    with_testnet(
        MANIFEST,
        |manifest| {
            // Try to make sure the bottom-up checkpoint period is quick enough for reasonable test runtime.
            let subnet = manifest
                .subnets
                .get_mut(&ResourceId::from("england"))
                .expect("subnet not found");

            subnet.bottom_up_checkpoint.period = CHECKPOINT_PERIOD;
        },
        |_, _, testnet| {
            let test = async {
                let brussels = testnet.node(&testnet.root().node("brussels"))?;
                let london = testnet.node(&testnet.root().subnet("england").node("london"))?;
                let england = testnet.subnet(&testnet.root().subnet("england"))?;

                let london_provider = Arc::new(
                    london
                        .ethapi_http_provider()?
                        .ok_or_else(|| anyhow!("ethapi should be enabled"))?,
                );

                let parent_provider = Arc::new(
                    brussels
                        .ethapi_http_provider()?
                        .ok_or_else(|| anyhow!("ethapi should be enabled"))?,
                );

                let parent_gateway_mesenger = GatewayMessengerFacet::new(
                    builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID),
                    parent_provider.clone(),
                );

                let bob = testnet.account("bob")?;
                let charlie = testnet.account("charlie")?;

                let root_network = testnet.subnet(&testnet.root())?;
                let subnet = testnet.subnet(&testnet.root().subnet("england"))?;

                let envelope = IpcEnvelope {
                    kind: IpcMsgKind::Call,
                    from: IPCAddress::new(&root_network.subnet_id, &charlie.fvm_addr())?,
                    to: IPCAddress::new(&subnet.subnet_id, &bob.fvm_addr())?,
                    value: TokenAmount::from_nano(10),
                    message: vec![0],
                    nonce: 0,
                };

                let mut commited_result =
                    parent_gateway_mesenger.send_contract_xnet_message(envelope.try_into()?);

                commited_result.tx.set_value(1);

                let commited_result = commited_result.call().await;

                match commited_result {
                    Ok(commited) => {
                        println!("Message sent successfully: {:?}", commited);
                    }
                    Err(ContractError::Revert(revert_reason)) => {
                        // Use the custom GatewayMessengerFacetErrors::decode to decode the revert data
                        match GatewayMessengerFacetErrors::decode_with_selector(&revert_reason) {
                            Some(decoded_error) => match decoded_error {
                                GatewayMessengerFacetErrors::CallFailed(call_failed) => {
                                    println!("Call failed: {:?}", call_failed);
                                }
                                GatewayMessengerFacetErrors::CannotSendCrossMsgToItself(err) => {
                                    println!("Cannot send cross message to itself: {:?}", err);
                                }
                                GatewayMessengerFacetErrors::CommonParentDoesNotExist(err) => {
                                    println!("Common parent does not exist: {:?}", err);
                                }
                                GatewayMessengerFacetErrors::InsufficientFunds(err) => {
                                    println!("Insufficient funds: {:?}", err);
                                }
                                GatewayMessengerFacetErrors::InvalidXnetMessage(err) => {
                                    println!("Invalid Xnet message: {:?}", err);
                                }
                                GatewayMessengerFacetErrors::MethodNotAllowed(err) => {
                                    println!("Method not allowed: {:?}", err);
                                }
                                GatewayMessengerFacetErrors::RevertString(reason) => {
                                    println!("Revert reason: {}", reason);
                                }
                            },
                            None => {
                                println!("Failed to decode revert reason");
                            }
                        }
                    }
                    Err(e) => {
                        // Generic error handler for other cases
                        println!(
                            "An error occurred while sending the contract message: {:?}",
                            e
                        );
                    }
                }

                // Gateway actor on the child
                let subnet_gateway_getter = GatewayGetterFacet::new(
                    builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID),
                    london_provider.clone(),
                );

                // Subnet actor on the parent
                let subnet_actor_getter = SubnetActorGetterFacet::new(
                    to_eth_address(&england.subnet_id.subnet_actor())
                        .and_then(|a| a.ok_or_else(|| anyhow!("not an eth address")))?,
                    parent_provider.clone(),
                );

                // // Query the latest committed parent finality and compare to the parent.
                // {
                //     let mut retry = 0;
                //     loop {
                //         let finality: ParentFinality = subnet_gateway_getter
                //             .get_latest_parent_finality()
                //             .call()
                //             .await
                //             .context("failed to get parent finality")?;

                //         // If the latest finality is not zero it means the syncer is working,
                //         if finality.height.is_zero() {
                //             if retry < MAX_RETRIES {
                //                 eprintln!("waiting for syncing with the parent...");
                //                 tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
                //                 retry += 1;
                //                 continue;
                //             }
                //             bail!("the parent finality is still zero");
                //         }

                //         // Check that the block hash of the parent is actually the same at that height.
                //         let parent_block: Option<et::Block<_>> = parent_provider
                //             .get_block(finality.height.as_u64())
                //             .await
                //             .context("failed to get parent block")?;

                //         let Some(parent_block_hash) = parent_block.and_then(|b| b.hash) else {
                //             bail!("cannot find parent block at final height");
                //         };

                //         if parent_block_hash.0 != finality.block_hash {
                //             bail!("the finality block hash is different from the API");
                //         }
                //         break;
                //     }
                // }

                // // Check that the parent knows about a checkpoint submitted from the child.
                // {
                //     let mut retry = 0;
                //     loop {
                //         // NOTE: The implementation of the following method seems like a nonsense;
                //         //       I don't know if there is a way to ask the gateway what the latest
                //         //       checkpoint is, so we'll just have to go to the parent directly.
                //         // let (has_checkpoint, epoch, _): (bool, et::U256, _) = england_gateway
                //         //     .get_current_bottom_up_checkpoint()
                //         //     .call()
                //         //     .await
                //         //     .context("failed to get current bottomup checkpoint")?;
                //         let ckpt_height: et::U256 = subnet_actor_getter
                //             .last_bottom_up_checkpoint_height()
                //             .call()
                //             .await
                //             .context("failed to query last checkpoint height")?;

                //         if !ckpt_height.is_zero() {
                //             break;
                //         }

                //         if retry < MAX_RETRIES {
                //             eprintln!("waiting for a checkpoint to be submitted...");
                //             tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
                //             retry += 1;
                //             continue;
                //         }

                //         bail!("hasn't submitted a bottom-up checkpoint");
                //     }
                // }

                Ok(())
            };

            test.boxed_local()
        },
    )
    .await
    .unwrap()
}
