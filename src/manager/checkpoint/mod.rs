// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::config::{ReloadableConfig, Subnet};
use crate::lotus::client::{DefaultLotusJsonRPCClient, LotusJsonRPCClient};
use crate::lotus::LotusClient;
use crate::manager::checkpoint::manager::bottomup::BottomUpCheckpointManager;
use crate::manager::checkpoint::manager::topdown::TopDownCheckpointManager;
use crate::manager::checkpoint::manager::CheckpointManager;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use futures_util::future::join_all;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_identity::Wallet;
use ipc_sdk::subnet_id::SubnetID;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::select;
use tokio::time::sleep;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

pub use manager::*;
mod manager;
mod proof;

const TASKS_PROCESS_THRESHOLD_SEC: u64 = 15;
const SUBMISSION_LOOK_AHEAD_EPOCH: ChainEpoch = 50;

pub struct CheckpointSubsystem {
    /// The subsystem uses a `ReloadableConfig` to ensure that, at all, times, the subnets under
    /// management are those in the latest version of the config.
    config: Arc<ReloadableConfig>,
    wallet_store: Arc<RwLock<Wallet>>,
}

impl CheckpointSubsystem {
    /// Creates a new `CheckpointSubsystem` with a configuration `config`.
    pub fn new(config: Arc<ReloadableConfig>, wallet_store: Arc<RwLock<Wallet>>) -> Self {
        Self {
            config,
            wallet_store,
        }
    }
}

#[async_trait]
impl IntoSubsystem<anyhow::Error> for CheckpointSubsystem {
    async fn run(self, subsys: SubsystemHandle) -> anyhow::Result<()> {
        // Each event in this channel is notification of a new config.
        let mut config_chan = self.config.new_subscriber();

        loop {
            // Load the latest config.
            let config = self.config.get_config();
            let (top_down_managers, bottom_up_managers) = match setup_managers_from_config(
                &config.subnets,
                self.wallet_store.clone(),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Please check configuration! Cannot start the checkpoint subsystem due to config error: {e:}. Update and reload config.");
                    match config_chan.recv().await {
                        Ok(_) => continue,
                        Err(e) => {
                            // this should seldom happen, but good to report it.
                            return Err(anyhow!(
                                "config update notification channel closed unexpected: {e:}"
                            ));
                        }
                    }
                }
            };

            loop {
                select! {
                    _ = process_managers(&top_down_managers) => {},
                    _ = process_managers(&bottom_up_managers) => {},
                    r = config_chan.recv() => {
                        log::info!("Config changed, reloading checkpointing subsystem");
                        match r {
                            // Config updated, return to caller
                            Ok(_) => { break; },
                            Err(_) => {
                                return Err(anyhow!("Config channel unexpectedly closed, shutting down checkpointing subsystem"))
                            },
                        }
                    }
                    _ = subsys.on_shutdown_requested() => {
                        log::info!("Shutting down checkpointing subsystem");
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn handle_err_response(manager: &impl CheckpointManager, response: anyhow::Result<()>) {
    if response.is_err() {
        log::error!("manger {manager:} had error: {:}", response.unwrap_err());
    }
}

async fn setup_managers_from_config(
    subnets: &HashMap<SubnetID, Subnet>,
    wallet_store: Arc<RwLock<Wallet>>,
) -> Result<(
    Vec<TopDownCheckpointManager>,
    Vec<BottomUpCheckpointManager<DefaultLotusJsonRPCClient>>,
)> {
    let mut bottom_up_managers = vec![];
    let mut top_down_managers = vec![];

    for s in subnets.values() {
        log::info!("config checkpoint manager for subnet: {:}", s.id);

        // We filter for subnets that have at least one account and for which the parent subnet
        // is also in the configuration.
        if s.accounts.is_empty() {
            log::info!("no accounts in subnet: {:}, not managing checkpoints", s.id);
            continue;
        }

        let parent = if let Some(p) = s.id.parent() && subnets.contains_key(&p) {
            subnets.get(&p).unwrap()
        } else {
            log::info!("subnet has no parent configured: {:}, not managing checkpoints", s.id);
            continue
        };

        bottom_up_managers.push(
            BottomUpCheckpointManager::new(
                LotusJsonRPCClient::from_subnet_with_wallet_store(parent, wallet_store.clone()),
                parent.clone(),
                LotusJsonRPCClient::from_subnet_with_wallet_store(s, wallet_store.clone()),
                s.clone(),
            )
            .await?,
        );

        top_down_managers.push(
            TopDownCheckpointManager::new(
                LotusJsonRPCClient::from_subnet_with_wallet_store(parent, wallet_store.clone()),
                parent.clone(),
                LotusJsonRPCClient::from_subnet_with_wallet_store(s, wallet_store.clone()),
                s.clone(),
            )
            .await?,
        );
    }

    log::info!(
        "we are managing checkpoints for {:} number of bottom up subnets",
        bottom_up_managers.len()
    );
    log::info!(
        "we are managing checkpoints for {:} number of top down subnets",
        top_down_managers.len()
    );

    Ok((top_down_managers, bottom_up_managers))
}

async fn process_managers<T: CheckpointManager>(managers: &[T]) -> anyhow::Result<()> {
    // Tracks the start time of the processing, will use this to determine should sleep
    let start_time = Instant::now();

    let futures = managers
        .iter()
        .map(|manager| async {
            let response = submit_till_current_epoch(manager).await;
            handle_err_response(manager, response);
        })
        .collect::<Vec<_>>();

    join_all(futures).await;

    sleep_or_continue(start_time).await;

    Ok(())
}

async fn sleep_or_continue(start_time: Instant) {
    let elapsed = start_time.elapsed().as_secs();
    if elapsed < TASKS_PROCESS_THRESHOLD_SEC {
        sleep(Duration::from_secs(TASKS_PROCESS_THRESHOLD_SEC - elapsed)).await
    }
}

/// Attempts to submit checkpoints from the last executed epoch all the way to the current epoch for
/// all the validators in the provided manager.
async fn submit_till_current_epoch(manager: &impl CheckpointManager) -> Result<()> {
    if !manager.presubmission_check().await? {
        log::info!("subnet in manager: {manager:} not ready to submit checkpoint");
        return Ok(());
    }

    // we might have to obtain the list of validators as some validators might leave the subnet
    // we can improve the performance by caching if this slows down the process significantly.
    let validators = child_validators(manager).await?;
    let period = manager.checkpoint_period();

    let last_executed_epoch = manager.last_executed_epoch().await?;
    let current_epoch = manager.current_epoch().await?;

    log::debug!(
        "latest epoch {:?}, last executed epoch: {:?} for checkpointing: {:}",
        current_epoch,
        last_executed_epoch,
        manager,
    );

    let mut next_epoch = last_executed_epoch + period;
    let cut_off_epoch = std::cmp::min(
        current_epoch,
        SUBMISSION_LOOK_AHEAD_EPOCH + last_executed_epoch,
    );

    // Instead of loop all the way to `current_epoch`, we loop till `cut_off_epoch`.
    // Reason because if the current epoch is significantly greater than last_executed_epoch and there
    // are lots of validators in the network, loop all the way to current epoch might have some outdated
    // data. Set a cut off epoch such that validators can sync with chain more regularly.
    while next_epoch < cut_off_epoch {
        // now we process each validator
        for validator in &validators {
            log::debug!("submit checkpoint for validator: {validator:?} in manager: {manager:}");

            if !manager
                .should_submit_in_epoch(validator, next_epoch)
                .await?
            {
                log::debug!(
                    "next submission epoch {next_epoch:?} already voted for validator: {:?} in manager: {manager:}",
                    validator.to_string()
                );
                continue;
            }

            log::debug!(
                "next submission epoch {next_epoch:} not voted for validator: {validator:} in manager: {manager:}, should vote"
            );

            manager.submit_checkpoint(next_epoch, validator).await?;

            log::info!("checkpoint at epoch {next_epoch:} submitted for validator {validator:} in manager: {manager:}");
        }

        // increment next epoch
        next_epoch += period;
    }

    log::info!("process checkpoint from epoch: {last_executed_epoch:} to {current_epoch:} in manager: {manager:}");

    Ok(())
}

/// Obtain the validators in the subnet from the parent subnet of the manager
async fn child_validators(manager: &impl CheckpointManager) -> anyhow::Result<Vec<Address>> {
    let parent_client = manager.parent_client();
    let parent_head = parent_client.chain_head().await?;

    // A key assumption we make now is that each block has exactly one tip set. We panic
    // if this is not the case as it violates our assumption.
    // TODO: update this logic once the assumption changes (i.e., mainnet)
    assert_eq!(parent_head.cids.len(), 1);

    let cid_map = parent_head.cids.first().unwrap().clone();
    let parent_tip_set = Cid::try_from(cid_map)?;
    let child_subnet = manager.child_subnet();

    let subnet_actor_state = parent_client
        .ipc_read_subnet_actor_state(&child_subnet.id, parent_tip_set)
        .await?;

    match subnet_actor_state.validator_set.validators {
        None => Ok(vec![]),
        Some(validators) => {
            let mut vs = vec![];
            for v in validators {
                let addr = Address::from_str(&v.addr)?;
                if child_subnet.accounts.contains(&addr) {
                    vs.push(addr);
                }
            }
            Ok(vs)
        }
    }
}
