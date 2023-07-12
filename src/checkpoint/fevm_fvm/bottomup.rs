// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::proof::create_proof;
use crate::checkpoint::CheckpointManager;
use crate::config::Subnet;
use crate::lotus::LotusClient;
use crate::manager::EthManager;
use anyhow::anyhow;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::{Display, Formatter};

/// Bottom up checkpoint manager. It reads the state of child subnet, FVM, and commits to parent subnet,
/// FEVM.
pub struct BottomUpCheckpointManager<ParentManager, ChildManager> {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    parent_fevm_manager: ParentManager,
    child_fvm_manager: ChildManager,
}

impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> BottomUpCheckpointManager<P, C> {
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
        let checkpoint_period = parent_manager
            .subnet_bottom_up_checkpoint_period(&child_subnet.id)
            .await?;
        Ok(Self::new_with_period(
            parent_subnet,
            child_subnet,
            checkpoint_period,
            parent_manager,
            child_manager,
        ))
    }
}

impl<P, M> Display for BottomUpCheckpointManager<P, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fvm to fevm bottom-up, parent: {:}, child: {:}, period: {:}",
            self.parent.id, self.child.id, self.checkpoint_period
        )
    }
}

#[async_trait]
impl<P: EthManager + Send + Sync, C: LotusClient + Send + Sync> CheckpointManager
    for BottomUpCheckpointManager<P, C>
{
    fn target_subnet(&self) -> &Subnet {
        &self.parent
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
        self.parent_fevm_manager.validators(&self.child.id).await
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_fevm_manager
            .subnet_last_voting_executed_epoch(&self.child.id)
            .await
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_fvm_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Getting fevm to fvm checkpoint bottom-up template for {epoch:} in subnet: {:?}",
            self.child.id
        );

        let template = self.child_fvm_manager
            .ipc_get_checkpoint_template(&self.child.gateway_addr(), epoch)
            .await
            .map_err(|e| {
                anyhow!(
                    "error getting bottom-up checkpoint template for epoch:{epoch:} in subnet: {:?} due to {e:}",
                    self.child.id
                )
            })?;
        log::info!("bottom up template: {template:?}");

        let mut checkpoint =
            crate::manager::evm::subnet_contract::BottomUpCheckpoint::try_from(template)
                .map_err(|e| anyhow!("cannot convert bottom up checkpoint due to: {e:}"))?;

        let proof = create_proof(&self.child_fvm_manager, epoch)
            .await
            .map_err(|e| anyhow!("cannot create proof due to: {e:}"))?;
        let proof_bytes = cbor::serialize(&proof, "fevm-fvm bottom up checkpoint proof")
            .map_err(|e| anyhow!("cannot serialized bottom up checkpoint due to: {e:}"))?
            .to_vec();
        checkpoint.proof = ethers::types::Bytes::from(proof_bytes);

        let prev_epoch = epoch - self.checkpoint_period;
        checkpoint.prev_hash = self
            .parent_fevm_manager
            .prev_bottom_up_checkpoint_hash(&self.child.id, prev_epoch)
            .await
            .map_err(|e| anyhow!("cannot get prev checkpoint hash due to: {e:}"))?;

        log::info!("bottom up checkpoint to submit: {checkpoint:?}");

        self.parent_fevm_manager
            .submit_bottom_up_checkpoint(validator, checkpoint)
            .await
            .map_err(|e| anyhow!("cannot submit bottom up checkpoint due to: {e:}"))?;
        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        let has_voted = self
            .parent_fevm_manager
            .has_voted_in_subnet(&self.child.id, epoch, validator)
            .await?;
        Ok(!has_voted)
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}
