use std::str::FromStr;
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::{
    create_proof, BottomUpHandler, CheckpointQuery, NativeBottomUpCheckpoint, TopDownHandler,
    VoteQuery,
};
use crate::jsonrpc::JsonRpcClientImpl;
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::{EthManager, EthSubnetManager, SubnetManager};
use anyhow::anyhow;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::{CrossMsg, TopDownCheckpoint};
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
impl CheckpointQuery<NativeBottomUpCheckpoint> for FevmSubnetManager {
    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .subnet_bottom_up_checkpoint_period(subnet_id)
            .await
    }

    async fn validators(&self, subnet_id: &SubnetID) -> anyhow::Result<Vec<Address>> {
        self.evm_subnet_manager.validators(subnet_id).await
    }
}

#[async_trait]
impl BottomUpHandler for FevmSubnetManager {
    async fn checkpoint_template(
        &self,
        epoch: ChainEpoch,
    ) -> anyhow::Result<NativeBottomUpCheckpoint> {
        let checkpoint = self.evm_subnet_manager.bottom_up_checkpoint(epoch).await?;
        log::debug!("raw bottom up templated: {checkpoint:?}");

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
            .submit_bottom_up_checkpoint(validator, checkpoint)
            .await
    }
}

#[async_trait]
impl VoteQuery<TopDownCheckpoint> for FevmSubnetManager {
    async fn last_executed_epoch(&self, _subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .gateway_last_voting_executed_epoch()
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
        let has_voted = self.evm_subnet_manager.has_voted_in_gateway(epoch, validator)
            .await
            .map_err(|e| {
                anyhow!("error checking if validator has voted topdown in epoch: {epoch:} for subnet: {subnet_id:} due to {e:}")
            })?;

        Ok(has_voted)
    }
}

#[async_trait]
impl CheckpointQuery<TopDownCheckpoint> for FevmSubnetManager {
    async fn checkpoint_period(&self, _subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .gateway_top_down_check_period()
            .await
    }

    async fn validators(&self, subnet_id: &SubnetID) -> anyhow::Result<Vec<Address>> {
        let r = self
            .evm_subnet_manager
            .get_validator_set(subnet_id, None)
            .await?;
        if let Some(validators) = r.validator_set.validators {
            let v = validators
                .into_iter()
                .map(|v| Address::from_str(&v.worker_addr.unwrap()))
                .collect::<Result<Vec<_>, _>>()?;
            log::debug!("top down validators: {v:?}");
            Ok(v)
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait]
impl TopDownHandler for FevmSubnetManager {
    async fn gateway_initialized(&self) -> anyhow::Result<bool> {
        self.evm_subnet_manager.gateway_initialized().await
    }

    async fn applied_topdown_nonce(&self, subnet_id: &SubnetID) -> anyhow::Result<u64> {
        self.evm_subnet_manager
            .get_applied_top_down_nonce(subnet_id)
            .await
    }

    async fn top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        nonce: u64,
        epoch: ChainEpoch,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        self.evm_subnet_manager
            .top_down_msgs(subnet_id, epoch, nonce)
            .await
    }

    async fn submit(
        &self,
        _validator: &Address,
        _checkpoint: TopDownCheckpoint,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }
}
