use std::sync::Arc;
use std::time::Duration;

// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::{anyhow, bail, Context};
use ethers::core::types as et;
use ethers::providers::Middleware;
use fendermint_vm_actor_interface::init::builtin_actor_eth_addr;
use fendermint_vm_message::conv::from_fvm::to_eth_address;
use futures::FutureExt;

use fendermint_materializer::{HasEthApi, ResourceId};
use fendermint_vm_actor_interface::ipc;
use ipc_actors_abis::gateway_getter_facet::{GatewayGetterFacet, ParentFinality};
use ipc_actors_abis::subnet_actor_getter_facet::SubnetActorGetterFacet;

use crate::with_testnet;

const MANIFEST: &str = "layer2.yaml";
const CHECKPOINT_PERIOD: u64 = 10;

/// Test that top-down syncing and bottom-up checkpoint submission work.
#[serial_test::serial]
#[tokio::test]
async fn test_topdown_and_bottomup() {
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

                let brussels_provider = Arc::new(
                    brussels
                        .ethapi_http_provider()?
                        .ok_or_else(|| anyhow!("ethapi should be enabled"))?,
                );

                let england_gateway = GatewayGetterFacet::new(
                    builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID),
                    london_provider.clone(),
                );

                let england_subnet = SubnetActorGetterFacet::new(
                    to_eth_address(&england.subnet_id.subnet_actor())
                        .ok_or_else(|| anyhow!("not an eth address"))?,
                    brussels_provider.clone(),
                );

                // Allow a bit of time for the subnet to sync with the parent.
                tokio::time::sleep(Duration::from_secs(5)).await;

                // Query the latest committed parent finality and compare to the parent.
                {
                    let finality: ParentFinality = england_gateway
                        .get_latest_parent_finality()
                        .call()
                        .await
                        .context("failed to get parent finality")?;

                    // If the latest finality is not zero it means the syncer is working,
                    if finality.height.is_zero() {
                        bail!("the parent finality is still zero");
                    }

                    // Check that the block hash of the parent is actually the same at that height.
                    let parent_block: Option<et::Block<_>> = brussels_provider
                        .get_block(finality.height.as_u64())
                        .await
                        .context("failed to get parent block")?;

                    let Some(parent_block_hash) = parent_block.and_then(|b| b.hash) else {
                        bail!("cannot find parent block at final height");
                    };

                    if parent_block_hash.0 != finality.block_hash {
                        bail!("the finality block hash is different from the API");
                    }
                }

                // Check that the parent knows about a checkpoint submitted from the child.
                {
                    let mut retry = 0;
                    loop {
                        let (has_checkpoint, epoch, _): (bool, et::U256, _) = england_gateway
                            .get_current_bottom_up_checkpoint()
                            .call()
                            .await
                            .context("failed to get current bottomup checkpoint")?;

                        if has_checkpoint {
                            if epoch.as_u64() < CHECKPOINT_PERIOD * 2 {
                                // Allow a bit of time for the checkpoint to be submitted.
                                tokio::time::sleep(Duration::from_secs(5)).await;
                            }
                            break;
                        }

                        if retry < 5 {
                            retry += 1;
                            eprintln!("waiting for a checkpoint to be produced...");
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }

                        bail!("hasn't produced a bottom-up checkpoint");
                    }

                    let ckpt_height: et::U256 = england_subnet
                        .last_bottom_up_checkpoint_height()
                        .call()
                        .await
                        .context("failed to query last checkpoint height")?;

                    if ckpt_height.is_zero() {
                        bail!("hasn't submitted a bottom-up checkpoint")
                    }
                }

                Ok(())
            };

            test.boxed_local()
        },
    )
    .await
    .unwrap()
}
