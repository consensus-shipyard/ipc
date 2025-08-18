// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Bottom up checkpoint manager

use crate::config::Subnet;
use crate::manager::cometbft::CometbftClient;
use crate::manager::{EthSubnetManager, SignedHeaderRelayer};
use anyhow::{anyhow, Result};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_wallet::{EthKeyAddress, PersistentKeyStore};
use std::cmp::max;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Tracks the config required for bottom up checkpoint submissions
/// parent/child subnet and checkpoint period.
pub struct CheckpointConfig {
    target: Subnet,
    source: Subnet,
    period: ChainEpoch,
}

pub struct BottomUpCheckpointManager<T> {
    metadata: CheckpointConfig,
    parent_handler: Arc<T>,
    cometbft_client: CometbftClient,
    child_handler: T,
    /// The number of blocks away from the chain head that is considered final
    finalization_blocks: ChainEpoch,
}

impl<T: SignedHeaderRelayer> BottomUpCheckpointManager<T> {
    pub async fn new(
        parent: Subnet,
        child: Subnet,
        cometbft_client: CometbftClient,
        parent_handler: T,
        child_handler: T,
    ) -> Result<Self> {
        let period = parent_handler
            .submission_period(&child.id)
            .await
            .map_err(|e| anyhow!("cannot get bottom up checkpoint period: {e}"))?;
        Ok(Self {
            metadata: CheckpointConfig {
                target: parent,
                source: child,
                period,
            },
            parent_handler: Arc::new(parent_handler),
            child_handler,
            finalization_blocks: 0,
            cometbft_client,
        })
    }

    pub fn with_finalization_blocks(mut self, finalization_blocks: ChainEpoch) -> Self {
        self.finalization_blocks = finalization_blocks;
        self
    }
}

impl BottomUpCheckpointManager<EthSubnetManager> {
    pub async fn new_evm_manager(
        parent: Subnet,
        child: Subnet,
        keystore: Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>,
        cometbft_client: CometbftClient,
    ) -> Result<Self> {
        let parent_handler =
            EthSubnetManager::from_subnet_with_wallet_store(&parent, Some(keystore.clone()))?;
        let child_handler =
            EthSubnetManager::from_subnet_with_wallet_store(&child, Some(keystore))?;
        Self::new(
            parent,
            child,
            cometbft_client,
            parent_handler,
            child_handler,
        )
        .await
    }
}

impl<T: SignedHeaderRelayer> Display for BottomUpCheckpointManager<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "light client relayer, target: {:}, source: {:}",
            self.metadata.target.id, self.metadata.source.id
        )
    }
}

impl<T: SignedHeaderRelayer + Send + Sync + 'static> BottomUpCheckpointManager<T> {
    /// Getter for the parent subnet this checkpoint manager is handling
    pub fn target_subnet(&self) -> &Subnet {
        &self.metadata.target
    }

    /// Getter for the target subnet this checkpoint manager is handling
    pub fn source_subnet(&self) -> &Subnet {
        &self.metadata.source
    }

    /// The submission period that the current manager is submitting upon
    pub fn submission_period(&self) -> ChainEpoch {
        self.metadata.period
    }

    /// Run the bottom up checkpoint submission daemon in the foreground
    pub async fn run(self, submitter: Address, submission_interval: Duration) {
        tracing::info!("launching {self} for {submitter}");

        loop {
            let height = match self.submit_next_signed_header(submitter).await {
                Ok(Some(h)) => h,
                Ok(None) => {
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                        "cannot submit checkpoint for submitter: {submitter} due to {e}"
                    );
                    continue;
                }
            };

            if let Err(e) = self.commit_next_validator_changes(submitter, height).await {
                tracing::error!(
                    "cannot commit next validator changes for submitter: {submitter} due to {e}"
                );
            }

            if let Err(e) = self.execute_pending_batch_commitments(submitter).await {
                tracing::error!("cannot execute pending batch commitments for submitter: {submitter} due to {e}");
            }
            tokio::time::sleep(submission_interval).await;
        }
    }

    async fn commit_next_validator_changes(
        &self,
        submitter: Address,
        end_height: ChainEpoch,
    ) -> Result<()> {
        let heights = self
            .child_handler
            .get_last_commitment_heights(&self.metadata.target.id)
            .await?;

        let mut next_height = heights.config_number;
        while next_height <= end_height as u64 {
            next_height += self.metadata.period as u64;
            let Some(commitment) = self
                .child_handler
                .query_commitment(next_height as ChainEpoch)
                .await?
            else {
                continue;
            };

            self.parent_handler
                .confirm_validator_change(
                    next_height as ChainEpoch,
                    &submitter,
                    &self.metadata.target.id,
                    commitment,
                )
                .await?;
        }

        Ok(())
    }

    async fn submit_next_signed_header(&self, submitter: Address) -> Result<Option<ChainEpoch>> {
        let last_checkpoint_epoch = self
            .parent_handler
            .last_submission_height(&self.metadata.source.id)
            .await
            .map_err(|e| {
                anyhow!("cannot obtain the last bottom up checkpoint height due to: {e:}")
            })?;

        let next_checkpoint_epoch = last_checkpoint_epoch + self.metadata.period;

        tracing::info!(
            last_checkpoint_epoch,
            next_checkpoint_epoch,
            "last and next checkpoint submission heights"
        );

        let current_height = self.child_handler.current_epoch().await?;
        let finalized_height = max(1, current_height - self.finalization_blocks);

        tracing::debug!("last submission height: {last_checkpoint_epoch}, current height: {current_height}, finalized_height: {finalized_height}");

        if finalized_height <= next_checkpoint_epoch {
            return Ok(None);
        }

        let active_validators = self
            .parent_handler
            .list_active_validators(&self.metadata.source.id)
            .await?;
        let pubkeys = active_validators
            .iter()
            .map(|(_, info)| info.staking.metadata.as_slice());

        let mut header = self
            .cometbft_client
            .fetch_signed_header(next_checkpoint_epoch)
            .await?;
        header.order_commit_against(pubkeys)?;
        let height = header.header.height;

        self.parent_handler
            .submit_signed_header(&submitter, &self.source_subnet().id, header)
            .await?;

        Ok(Some(height))
    }

    /// Checks if there are any pending bottom up batch commitments, if so execute them.
    async fn execute_pending_batch_commitments(&self, submitter: Address) -> Result<()> {
        let pending_commitments = self
            .parent_handler
            .list_pending_bottom_up_batch_commitments(&self.metadata.target.id)
            .await
            .map_err(|e| {
                anyhow!(
                    "cannot obtain the list of pending bottom up batch commitments due to: {e:}"
                )
            })?;
        tracing::info!("total pending commitments: {}", pending_commitments.len());

        for commitment in pending_commitments {
            let inclusions = self
                .child_handler
                .make_next_bottom_up_batch_inclusions(&commitment)
                .await?;

            let height = commitment.height as i64;
            self.parent_handler.execute_bottom_up_batch(
                &submitter,
                &self.metadata.target.id,
                height,
                inclusions,
            )
                .await
                .inspect_err(|err| {
                    tracing::error!("Fail to execute bottom up batch at height {height}: {err}");
                })?;

        }

        tracing::debug!("Waiting for all execution tasks to finish");

        Ok(())
    }
}
