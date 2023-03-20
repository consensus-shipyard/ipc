// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::cbor;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::MethodNum;
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use primitives::TCid;
use tokio::select;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

use crate::config::{ReloadableConfig, Subnet};
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;

/// The frequency at which to check a new chain head.
const CHAIN_HEAD_REQUEST_PERIOD: Duration = Duration::from_secs(10);

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

            // Create a `manage_subnet` future for each (child, parent) subnet pair under management
            // and collect them in a `FuturesUnordered` set.
            let mut manage_subnet_futures = FuturesUnordered::new();
            let stop_subnet_managers = Arc::new(Notify::new());
            for (child, parent) in subnets_to_manage(&config.subnets) {
                manage_subnet_futures
                    .push(manage_subnet((child, parent), stop_subnet_managers.clone()));
            }

            log::debug!("We have {} subnets to manage", manage_subnet_futures.len());

            // Spawn a task to drive the `manage_subnet` futures.
            let manage_subnets_task = tokio::spawn(async move {
                loop {
                    match manage_subnet_futures.next().await {
                        Some(Ok(())) => {}
                        Some(Err(e)) => {
                            log::error!("Error in manage_subnet: {}", e);
                        }
                        None => {
                            log::debug!("All manage_subnet_futures have finished");
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
fn subnets_to_manage(subnets: &HashMap<String, Subnet>) -> Vec<(Subnet, Subnet)> {
    // First, we remap subnets by SubnetID.
    let subnets_by_id: HashMap<SubnetID, Subnet> = subnets
        .values()
        .map(|s| (s.id.clone(), s.clone()))
        .collect();

    // Then, we filter for subnets that have at least one account and for which the parent subnet
    // is also in the map, and map into a Vec of (child_subnet, parent_subnet) tuples.
    subnets_by_id
        .values()
        .filter(|s| !s.accounts.is_empty())
        .filter(|s| s.id.parent().is_some() && subnets_by_id.contains_key(&s.id.parent().unwrap()))
        .map(|s| (s.clone(), subnets_by_id[&s.id.parent().unwrap()].clone()))
        .collect()
}

/// Monitors a subnet `child` for checkpoint blocks. It emits an event for every new checkpoint block.
async fn manage_subnet((child, parent): (Subnet, Subnet), stop_notify: Arc<Notify>) -> Result<()> {
    log::info!(
        "Starting checkpoint manager for (child, parent) subnet pair ({}, {})",
        child.id,
        parent.id
    );

    let child_client = LotusJsonRPCClient::from_subnet(&child);
    let parent_client = LotusJsonRPCClient::from_subnet(&parent);

    // Read the parent's chain head and obtain the tip set CID.
    let parent_head = parent_client.chain_head().await?;
    let cid_map = parent_head.cids.first().unwrap().clone();
    let parent_tip_set = Cid::try_from(cid_map)?;

    // Extract the checkpoint period from the state of the subnet actor in the parent.
    let state = parent_client
        .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
        .await?;
    let period = state.check_period;

    // We should have a way of knowing whether the validator has voted in the current open
    // checkpoint epoch.
    // TODO: Hook this up to the new IPC methods.

    // We can now start looping. In each loop we read the child subnet's chain head and check if
    // it's a checkpoint epoch. If it is, we construct and submit a checkpoint.
    loop {
        let child_head = child_client.chain_head().await?;
        let epoch: ChainEpoch = ChainEpoch::try_from(child_head.height)?;
        if epoch % period == 0 {
            // It's a checkpointing epoch and we may have checkpoints to submit.
            log::info!(
                "Checkpointing epoch {} for subnet pair ({}, {})",
                epoch,
                child.id,
                parent.id
            );

            // First, we check which accounts are in the validator set. This is done by reading
            // the parent's chain head and requesting the state at that tip set.
            let parent_head = parent_client.chain_head().await?;
            // A key assumption we make now is that each block has exactly one tip set. We panic
            // if this is not the case as it violates our assumption.
            // TODO: update this logic once the assumption changes (i.e., mainnet)
            assert_eq!(parent_head.cids.len(), 1);
            let cid_map = parent_head.cids.first().unwrap().clone();
            let parent_tip_set = Cid::try_from(cid_map)?;

            let subnet_actor_state = parent_client
                .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
                .await?;

            let mut validator_set: HashSet<Address, RandomState> = HashSet::new();
            match subnet_actor_state.validator_set.validators {
                None => {}
                Some(validators) => {
                    for v in validators {
                        validator_set.insert(Address::from_str(v.addr.deref())?);
                    }
                }
            };

            // Now, for each account defined in the `child` subnet that is in the validator set, we
            // submit a checkpoint on its behalf.
            assert_eq!(child_head.cids.len(), 1); // Again, check key assumption
            let child_tip_set = Cid::try_from(child_head.cids.first().unwrap().clone())?;
            for account in child.accounts.iter() {
                if validator_set.contains(account) {
                    log::info!(
                        "Submitting checkpoint for account {}  from child {} to parent {}",
                        account,
                        child.id,
                        parent.id
                    );
                    submit_checkpoint(
                        child_tip_set,
                        epoch,
                        account,
                        &child,
                        &child_client,
                        &parent_client,
                    )
                    .await?;
                }
            }
        }

        // Sleep for an appropriate amount of time before checking the chain head again or return
        // if a stop notification is received.
        select! {
            _ = sleep(CHAIN_HEAD_REQUEST_PERIOD) => {}
            _ = stop_notify.notified() => { return Ok(()); }
        }
    }
}

/// Submits a checkpoint for `epoch` on behalf of `account` to the subnet actor of `child_subnet`
/// deployed on the parent subnet.
async fn submit_checkpoint<T: JsonRpcClient + Send + Sync>(
    child_tip_set: Cid,
    epoch: ChainEpoch,
    account: &Address,
    child_subnet: &Subnet,
    child_client: &LotusJsonRPCClient<T>,
    parent_client: &LotusJsonRPCClient<T>,
) -> Result<()> {
    let mut checkpoint = Checkpoint::new(child_subnet.id.clone(), epoch);

    // Get the children checkpoints from the template on the gateway actor of the child subnet.
    let template = child_client.ipc_get_checkpoint_template(epoch).await?;
    checkpoint.data.children = template.data.children;

    // Get the CID of previous checkpoint of the child subnet from the gateway actor of the parent
    // subnet.
    let response = parent_client
        .ipc_get_prev_checkpoint_for_child(child_subnet.id.clone())
        .await?;
    let cid = Cid::try_from(response.cid)?;
    checkpoint.data.prev_check = TCid::from(cid);
    checkpoint.data.proof = child_tip_set.to_bytes();

    // The checkpoint is constructed. Now we call the `submit_checkpoint` method on the subnet actor
    // of the child subnet that is deployed on the parent subnet.
    let to = child_subnet.id.subnet_actor();
    let from = *account;
    let message = MpoolPushMessage::new(
        to,
        from,
        ipc_subnet_actor::Method::SubmitCheckpoint as MethodNum,
        cbor::serialize(&checkpoint, "checkpoint")?.to_vec(),
    );
    parent_client.mpool_push_message(message).await?;

    Ok(())
}
