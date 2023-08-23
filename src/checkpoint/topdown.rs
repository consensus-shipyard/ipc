// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::{CheckpointManager, CheckpointMetadata, CheckpointQuery};
use crate::config::Subnet;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::TopDownCheckpoint;
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::subnet_id::SubnetID;
use std::fmt::{Display, Formatter};

/// The trait that handles the bottom up checkpoint submission data preparation and actual submission.
#[async_trait]
pub trait TopDownHandler: Send + Sync + CheckpointQuery<TopDownCheckpoint> {
    /// Checks if the gateway is initialized
    async fn gateway_initialized(&self) -> Result<bool>;
    /// Get the latest applied top down nonce
    async fn applied_topdown_nonce(&self, subnet_id: &SubnetID) -> Result<u64>;
    /// Fetch the checkpoint top down messages at the specified epoch
    async fn top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        nonce: u64,
        epoch: ChainEpoch,
    ) -> Result<Vec<CrossMsg>>;
    /// Submit the checkpoint for validator
    async fn submit(
        &self,
        validator: &Address,
        checkpoint: TopDownCheckpoint,
    ) -> Result<ChainEpoch>;
}

pub struct TopDownManager<P: TopDownHandler, C: TopDownHandler> {
    metadata: CheckpointMetadata,
    parent_handler: P,
    child_handler: C,
}

impl<P: TopDownHandler, C: TopDownHandler> TopDownManager<P, C> {
    pub async fn new(
        parent: Subnet,
        child: Subnet,
        parent_handler: P,
        child_handler: C,
    ) -> Result<Self> {
        let period = child_handler
            .checkpoint_period(&child.id)
            .await
            .map_err(|e| anyhow!("cannot get bottom up checkpoint period: {e}"))?;
        Ok(Self {
            metadata: CheckpointMetadata {
                parent,
                child,
                period,
            },
            parent_handler,
            child_handler,
        })
    }
}

impl<P: TopDownHandler, C: TopDownHandler> Display for TopDownManager<P, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "top-down, parent: {:}, child: {:}",
            self.metadata.parent.id, self.metadata.child.id
        )
    }
}

#[async_trait]
impl<P: TopDownHandler, C: TopDownHandler> CheckpointManager for TopDownManager<P, C> {
    fn target_subnet(&self) -> &Subnet {
        &self.metadata.child
    }

    /// Getter for the parent subnet this checkpoint manager is handling
    fn parent_subnet(&self) -> &Subnet {
        &self.metadata.parent
    }

    /// Getter for the target subnet this checkpoint manager is handling
    fn child_subnet(&self) -> &Subnet {
        &self.metadata.child
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.metadata.period
    }

    async fn validators(&self) -> Result<Vec<Address>> {
        self.parent_handler
            .validators(&self.metadata.child.id)
            .await
    }

    async fn last_executed_epoch(&self) -> Result<ChainEpoch> {
        self.child_handler
            .last_executed_epoch(&self.metadata.child.id)
            .await
    }

    async fn current_epoch(&self) -> Result<ChainEpoch> {
        self.parent_handler.current_epoch().await
    }

    async fn submit_checkpoint(&self, epoch: ChainEpoch, validator: &Address) -> Result<()> {
        let nonce = self
            .child_handler
            .applied_topdown_nonce(&self.metadata.child.id)
            .await?;
        log::info!("latest applied top down nonce for {self:}: {nonce}");

        let top_down_msgs = self
            .parent_handler
            .top_down_msgs(&self.metadata.child.id, nonce, epoch)
            .await?;
        log::info!(
            "top down messages to execute for {self:}: {:}",
            top_down_msgs.len()
        );

        // we submit the topdown messages to the CHILD subnet.
        let topdown_checkpoint = TopDownCheckpoint {
            epoch,
            top_down_msgs,
        };

        log::info!("top down checkpoint to submit: {topdown_checkpoint:?}");

        let submitted_epoch = self
            .child_handler
            .submit(validator, topdown_checkpoint)
            .await?;

        log::info!(
            "checkpoint at epoch {:} for manager: {:} published with at epoch: {:?}, executed",
            epoch,
            self,
            submitted_epoch,
        );

        Ok(())
    }

    async fn should_submit_in_epoch(&self, validator: &Address, epoch: ChainEpoch) -> Result<bool> {
        let has_voted = self
            .child_handler
            .has_voted(&self.metadata.child.id, epoch, validator)
            .await?;
        Ok(!has_voted)
    }

    async fn presubmission_check(&self) -> Result<bool> {
        if self.metadata.parent.id.is_root() {
            Ok(true)
        } else {
            self.parent_handler.gateway_initialized().await
        }
    }
}
