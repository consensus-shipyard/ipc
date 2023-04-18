// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use ipc_sdk::subnet_id::SubnetID;
use tokio::select;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

use crate::config::{ReloadableConfig, Subnet};
use crate::manager::bottomup::manage_bottomup_checkpoints;
use crate::manager::topdown::manage_topdown_checkpoints;

/// The frequency at which to check a new chain head.
pub(crate) const CHAIN_HEAD_REQUEST_PERIOD: Duration = Duration::from_secs(10);

/// The `CheckpointSubsystem`. When run, it actively monitors subnets and submits checkpoints.
pub struct CheckpointSubsystem {
    /// The subsystem uses a `ReloadableConfig` to ensure that, at all, times, the subnets under
    /// management are those in the latest version of the config.
    config: Arc<ReloadableConfig>,
}

impl CheckpointSubsystem {
    /// Creates a new `CheckpointSubsystem` with a configuration `config`.
    pub fn new(config: Arc<ReloadableConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl IntoSubsystem<anyhow::Error> for CheckpointSubsystem {
    /// Runs the checkpoint subsystem, which actively monitors subnets and submits checkpoints.
    /// For each (account, subnet) that exists in the config, the subnet is monitored and checkpoints
    /// are submitted at the appropriate epochs.
    async fn run(self, subsys: SubsystemHandle) -> Result<()> {
        // Each event in this channel is notification of a new config.
        let mut config_chan = self.config.new_subscriber();

        loop {
            // Load the latest config.
            let config = self.config.get_config();

            // Create a top-down and a bottom-up checkpoint manager future for each (child, parent)
            // subnet pair under and collect them in a `FuturesUnordered` set.
            let mut manage_subnet_futures = FuturesUnordered::new();
            let stop_subnet_managers = Arc::new(Notify::new());
            let subnets_to_manage = subnets_to_manage(&config.subnets);
            log::debug!("We have {} subnets to manage", subnets_to_manage.len());

            for (child, parent) in subnets_to_manage {
                manage_subnet_futures.push(tokio::spawn(manage_bottomup_checkpoints(
                    (child.clone(), parent.clone()),
                    stop_subnet_managers.clone(),
                )));
                manage_subnet_futures.push(tokio::spawn(manage_topdown_checkpoints(
                    (child.clone(), parent.clone()),
                    stop_subnet_managers.clone(),
                )));
            }

            // Spawn a task to drive the `manage_subnet` futures.
            let manage_subnets_task = tokio::spawn(async move {
                loop {
                    match manage_subnet_futures.next().await {
                        Some(Err(e)) => {
                            panic!("Panic in manage_subnet: {:#}", e);
                        }
                        Some(Ok(r)) => {
                            if let Err(e) = r {
                                log::error!("Error in manage_subnet: {:#}", e);
                            }
                        }
                        None => {
                            log::debug!("All manage_subnet futures have finished");
                            break;
                        }
                    }
                }
            });

            // Watch for shutdown requests and config changes.
            let is_shutdown = select! {
                _ = subsys.on_shutdown_requested() => {
                    log::info!("Shutting down checkpointing subsystem");
                    true
                },
                r = config_chan.recv() => {
                    log::info!("Config changed, reloading checkpointing subsystem");
                    match r {
                        Ok(_) => { false },
                        Err(_) => {
                            log::error!("Config channel unexpectedly closed, shutting down checkpointing subsystem");
                            true
                        },
                    }
                },
            };

            // Cleanly stop the `manage_subnet` futures.
            stop_subnet_managers.notify_waiters();
            log::debug!("Waiting for subnet managers to finish");
            manage_subnets_task.await?;

            if is_shutdown {
                return anyhow::Ok(());
            }
        }
    }
}

/// This function takes a `HashMap<String, Subnet>` and returns a `Vec` of tuples of the form
/// `(child_subnet, parent_subnet)`, where `child_subnet` is a subnet that we need to actively
/// manage checkpoint for. This means that for each `child_subnet` there exists at least one account
/// for which we need to submit checkpoints on behalf of to `parent_subnet`, which must also be
/// present in the map.
fn subnets_to_manage(subnets_by_id: &HashMap<SubnetID, Subnet>) -> Vec<(Subnet, Subnet)> {
    // We filter for subnets that have at least one account and for which the parent subnet
    // is also in the map, and map into a Vec of (child_subnet, parent_subnet) tuples.
    subnets_by_id
        .values()
        .filter(|s| !s.accounts.is_empty())
        .filter(|s| s.id.parent().is_some() && subnets_by_id.contains_key(&s.id.parent().unwrap()))
        .map(|s| (s.clone(), subnets_by_id[&s.id.parent().unwrap()].clone()))
        .collect()
}

/// Sleeps for some time if stop_notify is not fired. It returns true to flag that we should move to the
/// next iteration of the loop, while false informs that the loop should return.
pub async fn wait_next_iteration(stop_notify: &Arc<Notify>, timeout: Duration) -> Result<bool> {
    select! {
        _ = sleep(timeout) => {Ok(true)}
        _ = stop_notify.notified() => {Ok(false)}
    }
}
