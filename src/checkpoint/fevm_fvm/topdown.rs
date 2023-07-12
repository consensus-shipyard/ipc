// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::{chain_head_cid, gateway_state, CheckpointManager};
use crate::config::Subnet;
use crate::lotus::LotusClient;
use crate::manager::EthManager;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::{CrossMsg, TopDownCheckpoint};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Top down checkpoint manager. It reads the state of parent subnet, FEVM, and commits to child subnet,
/// FVM.
#[warn(dead_code)]
pub struct TopDownCheckpointManager<ParentManager, ChildManager> {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    parent_fevm_manager: ParentManager,
    child_fvm_manager: ChildManager,
}

impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> TopDownCheckpointManager<P, C> {
    pub fn new_with_period(
        parent: Subnet,
        child: Subnet,
        checkpoint_period: ChainEpoch,
        parent_fevm_manager: P,
        child_fvm_manager: C,
    ) -> Self {
        Self {
            parent,
            child,
            checkpoint_period,
            parent_fevm_manager,
            child_fvm_manager,
        }
    }

    pub async fn new(
        parent_subnet: Subnet,
        parent_manager: P,
        child_subnet: Subnet,
        child_manager: C,
    ) -> anyhow::Result<Self> {
        let child_tip_set = chain_head_cid(&child_manager).await?;
        let state = child_manager
            .ipc_read_gateway_state(&child_subnet.gateway_addr(), child_tip_set)
            .await?;
        let checkpoint_period = state.top_down_check_period;
        Ok(Self::new_with_period(
            parent_subnet,
            child_subnet,
            checkpoint_period,
            parent_manager,
            child_manager,
        ))
    }
}

impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> Display
    for TopDownCheckpointManager<P, C>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fevm to fvm top-down, parent: {:}, child: {:}",
            self.parent.id, self.child.id
        )
    }
}

#[async_trait]
impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> CheckpointManager
    for TopDownCheckpointManager<P, C>
{
    fn target_subnet(&self) -> &Subnet {
        &self.child
    }

    fn parent_subnet(&self) -> &Subnet {
        &self.parent
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn validators(&self) -> anyhow::Result<Vec<Address>> {
        let r = self
            .parent_fevm_manager
            .get_validator_set(&self.child.id, self.parent.gateway_addr())
            .await?;
        if let Some(validators) = r.validator_set.validators {
            let v = validators
                .into_iter()
                .map(|v| Address::from_str(&v.worker_addr.unwrap()))
                .collect::<Result<Vec<_>, _>>()?;
            log::info!("top down validators: {v:?}");
            Ok(v)
        } else {
            Ok(vec![])
        }
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let child_gw_state = gateway_state(&self.child_fvm_manager, &self.child).await?;
        Ok(child_gw_state
            .top_down_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_fevm_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        let nonce = gateway_state(&self.child_fvm_manager, &self.child)
            .await?
            .applied_topdown_nonce;
        log::info!(
            "child subnet: {:?} applied top down nonce: {nonce:}",
            self.child.id
        );

        let msgs = self
            .parent_fevm_manager
            .top_down_msgs(&self.child.id, epoch, nonce)
            .await?;

        log::info!("top down messages: {msgs:?}");

        // we submit the topdown messages to the CHILD subnet.
        let topdown_checkpoint = TopDownCheckpoint {
            epoch,
            top_down_msgs: msgs
                .into_iter()
                .map(CrossMsg::try_from)
                .collect::<anyhow::Result<_>>()?,
        };

        log::info!("top down checkpoint to submit: {topdown_checkpoint:?}");

        let submitted_epoch = self
            .child_fvm_manager
            .ipc_submit_top_down_checkpoint(
                self.child.gateway_addr(),
                validator,
                topdown_checkpoint,
            )
            .await?;

        log::debug!(
            "checkpoint at epoch {:} for manager: {:} published with at epoch: {:?}, executed",
            epoch,
            self,
            submitted_epoch,
        );

        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        let has_voted = self
            .child_fvm_manager
            .ipc_validator_has_voted_topdown(&self.child.gateway_addr(), epoch, validator)
            .await
            .map_err(|e| {
                anyhow!("error checking if validator has voted for manager: {self:} due to {e:}")
            })?;

        // we should vote only when the validator has not voted
        Ok(!has_voted)
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        if self.parent.id.is_root() {
            Ok(true)
        } else {
            self.parent_fevm_manager.gateway_initialized().await
        }
    }
}
