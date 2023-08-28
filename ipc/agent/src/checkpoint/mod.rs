// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::config::{ReloadableConfig, Subnet};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_util::future::join_all;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_identity::Wallet;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::select;
use tokio::time::sleep;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

pub use bottomup::*;
use ipc_identity::PersistentKeyStore;
use ipc_sdk::subnet_id::SubnetID;
pub use proof::create_proof;
use std::fmt::Display;
pub use topdown::*;

mod bottomup;
mod proof;
mod setup;
mod topdown;

const TASKS_PROCESS_THRESHOLD_SEC: u64 = 15;
const SUBMISSION_LOOK_AHEAD_ROUNDS: i64 = 3;

/// Checkpoint manager that handles a specific parent - child - checkpoint type tuple.
/// For example, we might have `/r123` subnet and `/r123/t01` as child, one implementation of manager
/// is handling the top-down checkpoint submission for `/r123` to `/r123/t01`.
#[async_trait]
pub trait VoteQuery<T> {
    async fn last_executed_epoch(&self, subnet_id: &SubnetID) -> Result<ChainEpoch>;
    async fn current_epoch(&self) -> Result<ChainEpoch>;
    async fn has_voted(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> Result<bool>;
}

/// Checkpoint submission utility query trait
#[async_trait]
pub trait CheckpointQuery<T>: VoteQuery<T> {
    /// Get the checkpoint period
    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> Result<ChainEpoch>;
    /// Get the list of validators in the subnet id
    async fn validators(&self, subnet_id: &SubnetID) -> Result<Vec<Address>>;
}

#[async_trait]
pub trait CheckpointManager: Display + Send + Sync {
    /// Get the subnet config that this manager is submitting checkpoints to. For example, if it is
    /// top down checkpoints, target subnet return the child subnet config. If it is bottom up, target
    /// subnet returns parent subnet.
    fn target_subnet(&self) -> &Subnet;

    /// Getter for the parent subnet this checkpoint manager is handling
    fn parent_subnet(&self) -> &Subnet;

    /// Getter for the target subnet this checkpoint manager is handling
    fn child_subnet(&self) -> &Subnet;

    /// The checkpoint period that the current manager is submitting upon
    fn checkpoint_period(&self) -> ChainEpoch;

    /// Get the list of validators that should submit checkpoints
    async fn validators(&self) -> Result<Vec<Address>>;

    /// Obtain the last executed epoch of the checkpoint submission
    async fn last_executed_epoch(&self) -> Result<ChainEpoch>;

    /// The current epoch in the subnet that the checkpoints should be submitted to
    async fn current_epoch(&self) -> Result<ChainEpoch>;

    /// Submit the checkpoint based on the current epoch to submit and the previous epoch that was
    /// already submitted.
    async fn submit_checkpoint(&self, epoch: ChainEpoch, validator: &Address) -> Result<()>;

    /// Checks if the validator has already submitted in the epoch
    async fn should_submit_in_epoch(&self, validator: &Address, epoch: ChainEpoch) -> Result<bool>;

    /// Performs checks to see if the subnet is ready for checkpoint submission. If `true` means the
    /// subnet is ready for submission, else means the subnet is not ready.
    async fn presubmission_check(&self) -> Result<bool>;
}

pub struct CheckpointSubsystem {
    /// The subsystem uses a `ReloadableConfig` to ensure that, at all, times, the subnets under
    /// management are those in the latest version of the config.
    config: Arc<ReloadableConfig>,
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
}

impl CheckpointSubsystem {
    /// Creates a new `CheckpointSubsystem` with a configuration `config`.
    pub fn new(
        config: Arc<ReloadableConfig>,
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            config,
            fvm_wallet,
            evm_keystore,
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
            let managers = match setup::setup_managers_from_config(
                &config.subnets,
                self.fvm_wallet.clone(),
                self.evm_keystore.clone(),
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
                    _ = process_managers(managers.as_slice()) => {},
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

fn handle_err_response(manager: &dyn CheckpointManager, response: anyhow::Result<()>) {
    if response.is_err() {
        log::warn!("manager {manager:} had error: {:}", response.unwrap_err());
    }
}

async fn process_managers(managers: &[Box<dyn CheckpointManager>]) -> anyhow::Result<()> {
    // Tracks the start time of the processing, will use this to determine should sleep
    let start_time = Instant::now();

    let futures = managers
        .iter()
        .map(|manager| async {
            let response = submit_till_current_epoch(manager.borrow()).await;
            handle_err_response(manager.borrow(), response);
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
async fn submit_till_current_epoch(manager: &dyn CheckpointManager) -> Result<()> {
    if !manager.presubmission_check().await? {
        log::info!("subnet in manager: {manager:} not ready to submit checkpoint");
        return Ok(());
    }

    // we might have to obtain the list of validators as some validators might leave the subnet
    // we can improve the performance by caching if this slows down the process significantly.
    let mut validators = manager
        .validators()
        .await
        .map_err(|e| anyhow!("cannot get child validators for {manager:} due to {e:}"))?;
    log::debug!("list of validators from on chain: {validators:?} for manager: {manager:}");
    remove_not_managed(&mut validators, &manager.target_subnet().accounts());
    log::debug!("list of validators: {validators:?} for manager: {manager:}");

    if validators.is_empty() {
        log::info!(
            "no validators: {validators:?} for manager: {manager:}, do not submit checkpoints"
        );
        return Ok(());
    }

    let period = manager.checkpoint_period();
    log::debug!("checkpoint period: {period} for manager: {manager}");

    let last_executed_epoch = manager
        .last_executed_epoch()
        .await
        .map_err(|e| anyhow!("cannot get last executed epoch for {manager:} due to {e:}"))?;
    log::debug!("obtained last executed epoch: {last_executed_epoch:} for manager: {manager:}");
    let current_epoch = manager
        .current_epoch()
        .await
        .map_err(|e| anyhow!("cannot get the current eopch for {manager:} due to {e:}"))?;
    log::debug!("obtained current epoch: {last_executed_epoch:} for manager: {manager:}");

    log::info!(
        "latest epoch {:?}, last executed epoch: {:?} for checkpointing: {:}",
        current_epoch,
        last_executed_epoch,
        manager,
    );

    let mut next_epoch = last_executed_epoch + period;
    let cut_off_epoch = std::cmp::min(
        current_epoch,
        SUBMISSION_LOOK_AHEAD_ROUNDS * period + last_executed_epoch,
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

/// Removes the not managed accounts from the list of validators
fn remove_not_managed(validators: &mut Vec<Address>, managed_accounts: &[Address]) {
    let set: HashSet<_> = managed_accounts.iter().collect();
    validators.drain_filter(|v| !set.contains(v));
}

/// Tracks the metadata required for both top down and bottom up checkpoint submissions, such as
/// parent/child subnet and checkpoint period.
pub struct CheckpointMetadata {
    pub(crate) parent: Subnet,
    pub(crate) child: Subnet,
    pub(crate) period: ChainEpoch,
}
