// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::{anyhow, bail, Context};
use ethers::core::types as et;
use ethers::providers::Middleware;
use fendermint_materializer::{HasEthApi, ResourceId};
use std::sync::Arc;
use std::time::Duration;

use crate::docker_tests::make_middleware;
use crate::make_testnet;
use fendermint_vm_actor_interface::init::builtin_actor_eth_addr;
use fendermint_vm_actor_interface::ipc;
use fendermint_vm_message::conv::from_fvm::to_eth_address;
use ipc_actors_abis::gateway_getter_facet::{GatewayGetterFacet, ParentFinality};
use ipc_actors_abis::gateway_manager_facet::{FvmAddress, GatewayManagerFacet};
use ipc_actors_abis::subnet_actor_getter_facet::{SubnetActorGetterFacet, SubnetID};

const MANIFEST: &str = "layer2.yaml";
const CHECKPOINT_PERIOD: u64 = 10;
const SLEEP_SECS: u64 = 5;
const MAX_RETRIES: u32 = 5;

/// Test that top-down syncing and bottom-up checkpoint submission work.
#[serial_test::serial]
#[tokio::test]
async fn test_topdown_and_bottomup() -> Result<(), anyhow::Error> {
    let (testnet, cleanup) = make_testnet(MANIFEST, |manifest| {
        // Try to make sure the bottom-up checkpoint period is quick enough for reasonable test runtime.
        let subnet = manifest
            .subnets
            .get_mut(&ResourceId::from("england"))
            .expect("subnet not found");

        subnet.bottom_up_checkpoint.period = CHECKPOINT_PERIOD;
    })
    .await?;

    let res = {
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

        // Gateway actor on the child
        let england_gateway = GatewayGetterFacet::new(
            builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID),
            london_provider.clone(),
        );

        // Subnet actor on the parent
        let england_subnet = SubnetActorGetterFacet::new(
            to_eth_address(&england.subnet_id.subnet_actor())
                .and_then(|a| a.ok_or_else(|| anyhow!("not an eth address")))?,
            brussels_provider.clone(),
        );

        // Query the latest committed parent finality and compare to the parent.
        {
            let mut retry = 0;
            loop {
                let finality: ParentFinality = england_gateway
                    .get_latest_parent_finality()
                    .call()
                    .await
                    .context("failed to get parent finality")?;

                // If the latest finality is not zero it means the syncer is working,
                if finality.height.is_zero() {
                    if retry < MAX_RETRIES {
                        eprintln!("waiting for syncing with the parent...");
                        tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
                        retry += 1;
                        continue;
                    }
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
                break;
            }
        }

        // Check that the parent knows about a checkpoint submitted from the child.
        {
            let mut retry = 0;
            loop {
                // NOTE: The implementation of the following method seems like a nonsense;
                //       I don't know if there is a way to ask the gateway what the latest
                //       checkpoint is, so we'll just have to go to the parent directly.
                // let (has_checkpoint, epoch, _): (bool, et::U256, _) = england_gateway
                //     .get_current_bottom_up_checkpoint()
                //     .call()
                //     .await
                //     .context("failed to get current bottomup checkpoint")?;
                let ckpt_height: et::U256 = england_subnet
                    .last_bottom_up_checkpoint_height()
                    .call()
                    .await
                    .context("failed to query last checkpoint height")?;

                if !ckpt_height.is_zero() {
                    break;
                }

                if retry < MAX_RETRIES {
                    eprintln!("waiting for a checkpoint to be submitted...");
                    tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
                    retry += 1;
                    continue;
                }

                bail!("hasn't submitted a bottom-up checkpoint");
            }
        }

        Ok(())
    };

    cleanup(res.is_err(), testnet).await;
    res
}

/// Test that bottom-up checkpoint submission and execution work.
#[serial_test::serial]
#[tokio::test]
async fn test_bottomup_batch_execution() -> Result<(), anyhow::Error> {
    let (testnet, cleanup) = make_testnet(MANIFEST, |manifest| {
        // Try to make sure the bottom-up checkpoint period is quick enough for reasonable test runtime.
        let subnet = manifest
            .subnets
            .get_mut(&ResourceId::from("england"))
            .expect("subnet not found");

        subnet.bottom_up_checkpoint.period = CHECKPOINT_PERIOD;
    })
    .await?;

    let res = {
        let brussels = testnet.node(&testnet.root().node("brussels"))?;
        let london = testnet.node(&testnet.root().subnet("england").node("london"))?;
        let england = testnet.subnet(&testnet.root().subnet("england"))?;

        let london_provider = london
            .ethapi_http_provider()?
            .ok_or_else(|| anyhow!("ethapi should be enabled"))?;

        let brussels_provider = Arc::new(
            brussels
                .ethapi_http_provider()?
                .ok_or_else(|| anyhow!("ethapi should be enabled"))?,
        );

        // Subnet actor on the parent
        let england_subnet = SubnetActorGetterFacet::new(
            to_eth_address(&england.subnet_id.subnet_actor())
                .and_then(|a| a.ok_or_else(|| anyhow!("not an eth address")))?,
            brussels_provider.clone(),
        );

        // Prepare account 1.
        let sender1 = testnet.account_mod_nth(1);
        let middleware_sender1 = make_middleware(london_provider.clone(), sender1, None)
            .await
            .context("make_middleware")?;

        // Prepare account 2.
        let sender2 = testnet.account_mod_nth(2);
        let middleware_sender2 = make_middleware(london_provider.clone(), sender2, None)
            .await
            .context("make_middleware")?;

        // Gateway actor on the child
        let london_provider = Arc::new(london_provider);
        let england_gateway = GatewayManagerFacet::new(
            builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID),
            london_provider.clone(),
        );

        // Use the `release` function on gateway to trigger bottom-up batch with 2 messages.
        let tx1 = england_gateway
            .release(FvmAddress::default())
            .value(1)
            .from(sender1.eth_addr())
            .tx;
        let tx2 = england_gateway
            .release(FvmAddress::default())
            .value(1)
            .from(sender2.eth_addr())
            .tx;

        let fut1 = async {
            let pending = middleware_sender1
                .send_transaction(tx1, None)
                .await
                .context("failed to send txn 1")?;
            pending
                .interval(Duration::from_millis(50))
                .confirmations(1)
                .await?
                .context("tx1 not confirmed")
        };
        let fut2 = async {
            let pending = middleware_sender2
                .send_transaction(tx2, None)
                .await
                .context("failed to send txn 2")?;
            pending
                .interval(Duration::from_millis(50))
                .confirmations(1)
                .await?
                .context("tx2 not confirmed")
        };
        let (res1, res2) = tokio::join!(fut1, fut2);
        res1?;
        res2?;

        // Check that the parent knows about a checkpoint submitted from the child.
        {
            let mut retry = 0;
            loop {
                let ckpt_height: et::U256 = england_subnet
                    .last_bottom_up_checkpoint_height()
                    .call()
                    .await
                    .context("failed to query last checkpoint height")?;

                if !ckpt_height.is_zero() {
                    break;
                }

                if retry < MAX_RETRIES {
                    eprintln!("waiting for a checkpoint to be submitted...");
                    tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
                    retry += 1;
                    continue;
                }

                bail!("hasn't submitted a bottom-up checkpoint");
            }
        }

        // Check that the bottom-up batch execution isn't pending.
        {
            let mut retry = 0;
            loop {
                let pending = england_subnet
                    .list_pending_bottom_up_batch_commitments(SubnetID {
                        root: england.subnet_id.root_id(),
                        route: england
                            .subnet_id
                            .children()
                            .iter()
                            .map(|x| to_eth_address(x).unwrap().unwrap())
                            .collect(),
                    })
                    .call()
                    .await
                    .context("failed to query last checkpoint height")?;

                if pending.is_empty() {
                    break;
                }

                if retry < MAX_RETRIES {
                    eprintln!("waiting for a bottom-up batch to be executed...");
                    tokio::time::sleep(Duration::from_secs(SLEEP_SECS)).await;
                    retry += 1;
                    continue;
                }

                bail!("hasn't executed bottom-up batch");
            }
        }

        Ok(())
    };

    cleanup(res.is_err(), testnet).await;
    res
}
