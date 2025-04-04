// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::{anyhow, bail, Context};
use ethers::core::types as et;
use std::sync::Arc;
use std::time::Duration;

use fendermint_materializer::{HasEthApi, ResourceId};
use fendermint_vm_message::conv::from_fvm::to_eth_address;
use ipc_actors_abis::subnet_actor_getter_facet::SubnetActorGetterFacet;

use crate::make_testnet;

const MANIFEST: &str = "layer2.yaml";
const CHECKPOINT_PERIOD: u64 = 2;
const SLEEP_SECS: u64 = 5;
/// Keep a slightly bigger number of retries to account for node bootstrap time
const MAX_RETRIES: u32 = 10;

/// Test that bottom-up checkpoint submission work.
#[serial_test::serial]
#[tokio::test]
async fn test_bottomup() -> Result<(), anyhow::Error> {
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
        let england = testnet.subnet(&testnet.root().subnet("england"))?;

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
