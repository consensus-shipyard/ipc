// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::{create_proof, BottomUpHandler, NativeBottomUpCheckpoint, VoteQuery};
use crate::jsonrpc::JsonRpcClientImpl;
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::evm::subnet_contract;
use crate::manager::{EthManager, EthSubnetManager};
use anyhow::anyhow;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

pub struct FevmSubnetManager {
    evm_subnet_manager: EthSubnetManager,
    lotus_client: LotusJsonRPCClient<JsonRpcClientImpl>,
}

impl FevmSubnetManager {
    pub fn new(
        evm_subnet_manager: EthSubnetManager,
        lotus_client: LotusJsonRPCClient<JsonRpcClientImpl>,
    ) -> Self {
        Self {
            evm_subnet_manager,
            lotus_client,
        }
    }
}

#[async_trait]
impl VoteQuery<NativeBottomUpCheckpoint> for FevmSubnetManager {
    async fn last_executed_epoch(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .subnet_last_voting_executed_epoch(subnet_id)
            .await
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager.current_epoch().await
    }

    async fn has_voted(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<bool> {
        self.evm_subnet_manager
            .has_voted_in_subnet(subnet_id, epoch, validator)
            .await
    }
}

#[async_trait]
impl BottomUpHandler for FevmSubnetManager {
    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .subnet_bottom_up_checkpoint_period(subnet_id)
            .await
    }

    async fn validators(&self, subnet_id: &SubnetID) -> anyhow::Result<Vec<Address>> {
        self.evm_subnet_manager.validators(subnet_id).await
    }

    async fn checkpoint_template(
        &self,
        epoch: ChainEpoch,
    ) -> anyhow::Result<NativeBottomUpCheckpoint> {
        let checkpoint = self.evm_subnet_manager.bottom_up_checkpoint(epoch).await?;
        NativeBottomUpCheckpoint::try_from(checkpoint)
    }

    async fn populate_prev_hash(
        &self,
        template: &mut NativeBottomUpCheckpoint,
        subnet: &SubnetID,
        previous_epoch: ChainEpoch,
    ) -> anyhow::Result<()> {
        template.prev_check = Some(
            self.evm_subnet_manager
                .prev_bottom_up_checkpoint_hash(subnet, previous_epoch)
                .await
                .map_err(|e| anyhow!("cannot get prev checkpoint hash due to: {e:}"))?
                .to_vec(),
        );
        Ok(())
    }

    async fn populate_proof(&self, template: &mut NativeBottomUpCheckpoint) -> anyhow::Result<()> {
        let proof = create_proof(&self.lotus_client, template.epoch).await?;
        let proof_bytes = cbor::serialize(&proof, "fevm bottom up checkpoint proof")?.to_vec();
        template.proof = Some(proof_bytes);
        Ok(())
    }

    async fn submit(
        &self,
        validator: &Address,
        checkpoint: NativeBottomUpCheckpoint,
    ) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .submit_bottom_up_checkpoint(
                validator,
                subnet_contract::BottomUpCheckpoint::try_from(checkpoint)?,
            )
            .await
    }
}
