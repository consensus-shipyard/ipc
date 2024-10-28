// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::{anyhow, bail, Context};
use ethers::contract::{ContractError, ContractRevert};
use ethers::core::types as et;
use ethers::types::transaction::response;
use ethers::types::U256;
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

use ipc_actors_abis::cross_messenger::{
    self, CrossMessenger, FvmAddress, Ipcaddress, SubnetID as IPCSubnetID,
};

use fvm_shared::econ::TokenAmount;

use ipc_api::address::IPCAddress;
use ipc_api::cross::{IpcEnvelope, IpcMsgKind};
use ipc_api::evm;
use ipc_api::subnet_id::SubnetID;

use crate::with_testnet;

const MANIFEST: &str = "layer3.yaml";
const CHECKPOINT_PERIOD: u64 = 10;
const SLEEP_SECS: u64 = 5;
const MAX_RETRIES: u32 = 5;

use fvm_shared::address::{Address, Payload};

/// Convert the ipc SubnetID type to a vec of evm addresses. It extracts all the children addresses
/// in the subnet id and turns them as a vec of evm addresses.
pub fn subnet_id_to_evm_addresses(
    subnet: &SubnetID,
) -> anyhow::Result<Vec<ethers::types::Address>> {
    let children = subnet.children();
    children
        .iter()
        .map(|addr| payload_to_evm_address(addr.payload()))
        .collect::<anyhow::Result<_>>()
}

/// Util function to convert Fil address payload to evm address. Only delegated address is supported.
pub fn payload_to_evm_address(payload: &Payload) -> anyhow::Result<ethers::types::Address> {
    match payload {
        Payload::Delegated(delegated) => {
            let slice = delegated.subaddress();
            Ok(ethers::types::Address::from_slice(&slice[0..20]))
        }
        _ => Err(anyhow!("address provided is not delegated")),
    }
}

#[serial_test::serial]
#[tokio::test]
async fn test_cross_messages() {
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

                let parent_cross_messenger = CrossMessenger::new(
                    builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID),
                    parent_provider.clone(),
                );

                let call = parent_cross_messenger.invoke_cross_message(
                    Ipcaddress {
                        subnet_id: IPCSubnetID {
                            root: root_network.subnet_id.root_id(),
                            route: Default::default(),
                        },
                        raw_address: Default::default(),
                    },
                    Ipcaddress {
                        subnet_id: IPCSubnetID {
                            root: subnet.subnet_id.root_id(),
                            route: subnet_id_to_evm_addresses(&subnet.subnet_id)?,
                        },
                        raw_address: Default::default(),
                    },
                    U256::from(3),
                );

                let response = call.call().await;

                match response {
                    Ok(response) => {
                        println!("Response: {:?}", response);
                    }
                    Err(ContractError::Revert(revert_reason)) => {
                        println!("Revert reason: {:?}", revert_reason);
                    }
                    Err(e) => {
                        // Generic error handler for other cases
                        println!(
                            "An error occurred while sending the contract message: {:?}",
                            e
                        );
                    }
                }

                // let from = IPCAddress::new(&root_network.subnet_id, &charlie.fvm_addr())?;
                // let to = IPCAddress::new(&subnet.subnet_id, &bob.fvm_addr())?;

                // parent_cross_messenger.invoke_cross_message(
                //     from.try_into().unwrap(),
                //     to.try_into(),
                //     TokenAmount::from_nano(10),
                // );

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
