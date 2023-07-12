// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::proof::create_proof;
use crate::checkpoint::CheckpointManager;
use crate::config::Subnet;
use crate::lotus::LotusClient;
use crate::manager::EthManager;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::{Display, Formatter};

pub struct BottomUpCheckpointManager<T, M> {
    parent_subnet: Subnet,
    parent_manager: T,
    child_subnet: Subnet,
    child_manager: T,
    checkpoint_period: ChainEpoch,
    /// The lotus client. This is mainly for proof generation
    child_lotus_client: M,
}

impl<T: EthManager + Send + Sync, M: LotusClient + Send + Sync> BottomUpCheckpointManager<T, M> {
    pub fn new_with_period(
        parent: Subnet,
        parent_manager: T,
        child_subnet: Subnet,
        child_manager: T,
        child_lotus_client: M,
        checkpoint_period: ChainEpoch,
    ) -> Self {
        Self {
            parent_subnet: parent,
            parent_manager,
            child_subnet,
            child_manager,
            checkpoint_period,
            child_lotus_client,
        }
    }

    pub async fn new(
        parent: Subnet,
        parent_manager: T,
        child_subnet: Subnet,
        child_manager: T,
        child_lotus_client: M,
    ) -> anyhow::Result<Self> {
        let checkpoint_period = parent_manager
            .subnet_bottom_up_checkpoint_period(&child_subnet.id)
            .await?;
        Ok(Self::new_with_period(
            parent,
            parent_manager,
            child_subnet,
            child_manager,
            child_lotus_client,
            checkpoint_period,
        ))
    }
}

impl<T, M> Display for BottomUpCheckpointManager<T, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fevm bottom-up, parent: {:}, child: {:}",
            self.parent_subnet.id, self.child_subnet.id
        )
    }
}

#[async_trait]
impl<T: EthManager + Send + Sync, M: LotusClient + Send + Sync> CheckpointManager
    for BottomUpCheckpointManager<T, M>
{
    fn target_subnet(&self) -> &Subnet {
        &self.parent_subnet
    }

    fn parent_subnet(&self) -> &Subnet {
        &self.parent_subnet
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child_subnet
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn validators(&self) -> anyhow::Result<Vec<Address>> {
        self.parent_manager.validators(&self.child_subnet.id).await
    }

    /// The last executed voting epoch for bottom up checkpoint, the value should be fetch from
    /// parent gateway.
    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_manager
            .gateway_last_voting_executed_epoch()
            .await
    }

    /// Bottom up checkpoint submission, we should be focusing on the child subnet's current block
    /// number/chain epoch
    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_manager.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        let mut checkpoint = self.child_manager.bottom_up_checkpoint(epoch).await?;

        let proof = create_proof(&self.child_lotus_client, epoch).await?;
        let proof_bytes = cbor::serialize(&proof, "fevm bottom up checkpoint proof")?.to_vec();
        checkpoint.proof = ethers::types::Bytes::from(proof_bytes);

        self.parent_manager
            .submit_bottom_up_checkpoint(validator, checkpoint)
            .await?;
        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        self.parent_manager
            .has_voted_in_subnet(&self.child_subnet.id, epoch, validator)
            .await
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}
