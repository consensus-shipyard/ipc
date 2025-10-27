// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Bottom up checkpoint manager

use crate::config::Subnet;
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
    parent: Subnet,
    child: Subnet,
    period: ChainEpoch,
}

pub struct BottomUpCheckpointManager<T> {
    metadata: CheckpointConfig,
    parent_handler: Arc<T>,
    child_handler: T,
    /// The number of blocks away from the chain head that is considered final
    finalization_blocks: ChainEpoch,
}

impl<T: SignedHeaderRelayer> BottomUpCheckpointManager<T> {
    pub async fn new(
        parent: Subnet,
        child: Subnet,
        parent_handler: T,
        child_handler: T,
    ) -> Result<Self> {
        let period = parent_handler
            .submission_period(&child.id)
            .await
            .map_err(|e| anyhow!("cannot get bottom up checkpoint period: {e}"))?;
        Ok(Self {
            metadata: CheckpointConfig {
                parent,
                child,
                period,
            },
            parent_handler: Arc::new(parent_handler),
            child_handler,
            finalization_blocks: 0,
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
    ) -> Result<Self> {
        let parent_handler =
            EthSubnetManager::from_subnet_with_wallet_store(&parent, Some(keystore.clone()))?;
        let child_handler =
            EthSubnetManager::from_subnet_with_wallet_store(&child, Some(keystore))?;
        Self::new(parent, child, parent_handler, child_handler).await
    }
}

impl<T: SignedHeaderRelayer> Display for BottomUpCheckpointManager<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "light client relayer, target: {:}, source: {:}",
            self.metadata.parent.id, self.metadata.child.id
        )
    }
}

impl<T: SignedHeaderRelayer + Send + Sync + 'static> BottomUpCheckpointManager<T> {
    /// Getter for the parent subnet this checkpoint manager is handling
    pub fn parent_subnet(&self) -> &Subnet {
        &self.metadata.parent
    }

    /// Getter for the target subnet this checkpoint manager is handling
    pub fn child_subnet(&self) -> &Subnet {
        &self.metadata.child
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
                    tokio::time::sleep(submission_interval).await;
                    continue;
                }
            };

            if let Err(e) = self
                .submit_missing_app_hash_breakdowns(submitter, height)
                .await
            {
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

    /// The bottom up checkpoint submits only the app hash. The breakdown of the app hash, i.e.
    /// configuration number or bottom up message batch merkle root have to be submitted separately.
    /// This method does not handle rollup activity as it's not required in ro
    async fn submit_missing_app_hash_breakdowns(
        &self,
        submitter: Address,
        end_height: ChainEpoch,
    ) -> Result<()> {
        let mut next_height = self
            .parent_handler
            .get_last_app_commitment_height(&self.metadata.child.id)
            .await?;

        next_height += self.metadata.period as u64;

        while next_height < end_height as u64 {
            let Some(mut commitment) = self
                .child_handler
                .query_app_hash_breakdown(next_height as ChainEpoch)
                .await?
            else {
                continue;
            };

            let state_root = self
                .child_handler
                // the state root from fendermint client is actually in the next block height
                .get_state_root((next_height + 1) as ChainEpoch)
                .await?;

            tracing::info!(
                height = next_height,
                state_root = hex::encode(state_root.as_slice()),
                "obtains state root at height"
            );

            commitment.state_root = ethers::types::Bytes::from(state_root);

            self.parent_handler
                .record_app_hash_breakdown(
                    next_height as ChainEpoch,
                    &submitter,
                    &self.metadata.child.id,
                    commitment,
                )
                .await?;

            next_height += self.metadata.period as u64;
        }

        Ok(())
    }

    async fn submit_next_signed_header(&self, submitter: Address) -> Result<Option<ChainEpoch>> {
        let last_checkpoint_epoch = self
            .parent_handler
            .get_last_bottom_up_checkpoint_height(&self.metadata.child.id)
            .await
            .map_err(|e| {
                anyhow!("cannot obtain the last bottom up checkpoint height due to: {e:}")
            })?;

        let next_checkpoint_epoch = last_checkpoint_epoch as ChainEpoch + self.metadata.period;

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
            .list_active_validators(&self.metadata.child.id)
            .await?;
        tracing::info!(
            length = active_validators.len(),
            "obtained list of active validators"
        );

        let pubkeys = active_validators
            .iter()
            .map(|(_, info)| info.staking.metadata.as_slice());
        tracing::info!("obtained list of active validators public keys");

        let mut header = self
            .child_handler
            // we need to query signed header of the next block for the app hash in the checkpoint epoch
            .get_signed_header(next_checkpoint_epoch as u64 + 1)
            .await?;
        tracing::info!("obtained signed header: {header:?}");

        // order validators against the public keys ordered on chain. This is required as contract
        // requires the exact public keys ordering onchain.
        let cert = header.generate_validator_cert(pubkeys)?;
        tracing::info!(cert = ?cert, "obtained certificate");

        let height = header.header.height;

        self.parent_handler
            .submit_signed_header(&submitter, &self.child_subnet().id, header, cert)
            .await?;

        Ok(Some(height))
    }

    /// Checks if there are any pending bottom up batch commitments, if so execute them.
    async fn execute_pending_batch_commitments(&self, submitter: Address) -> Result<()> {
        let pending_commitments = self
            .parent_handler
            .list_pending_bottom_up_batch_commitments(&self.metadata.child.id)
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
            self.parent_handler
                .execute_bottom_up_batch(&submitter, &self.metadata.child.id, height, inclusions)
                .await
                .inspect_err(|err| {
                    tracing::error!("Fail to execute bottom up batch at height {height}: {err}");
                })?;
        }

        tracing::debug!("Waiting for all execution tasks to finish");

        Ok(())
    }
}
