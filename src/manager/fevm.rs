// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::checkpoint::{
    create_proof, BottomUpHandler, CheckpointQuery, NativeBottomUpCheckpoint, TopDownHandler,
    VoteQuery,
};
use crate::config::Subnet;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::chain::ChainHeadResponse;
use crate::lotus::message::ipc::QueryValidatorSetResponse;
use crate::lotus::LotusClient;
use crate::manager::subnet::TopDownCheckpointQuery;
use crate::manager::{EthManager, EthSubnetManager, SubnetInfo, SubnetManager};
use anyhow::anyhow;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_agent_sdk::jsonrpc::JsonRpcClientImpl;
use ipc_gateway::{CrossMsg, TopDownCheckpoint};
use ipc_identity::{PersistentKeyStore, Wallet};
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::ConstructParams;

pub struct FevmSubnetManager {
    evm_subnet_manager: EthSubnetManager,
    lotus_client: LotusJsonRPCClient<JsonRpcClientImpl>,
}

impl FevmSubnetManager {
    pub fn from_subnet_with_wallet_store(
        subnet: &Subnet,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
        fvm_wallet: Arc<RwLock<Wallet>>,
    ) -> anyhow::Result<Self> {
        let eth = EthSubnetManager::from_subnet_with_wallet_store(subnet, evm_keystore)?;
        let client = LotusJsonRPCClient::from_subnet_with_wallet_store(subnet, fvm_wallet);
        Ok(Self {
            evm_subnet_manager: eth,
            lotus_client: client,
        })
    }

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
impl TopDownCheckpointQuery for FevmSubnetManager {
    async fn chain_head(&self) -> anyhow::Result<ChainHeadResponse> {
        self.lotus_client.chain_head().await
    }

    async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        start_epoch: ChainEpoch,
        end_epoch: ChainEpoch,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        self.top_down_msgs(subnet_id, start_epoch, end_epoch).await
    }

    async fn get_block_hash(&self, height: ChainEpoch) -> anyhow::Result<Vec<u8>> {
        self.evm_subnet_manager.get_block_hash(height).await
    }
}

#[async_trait]
impl SubnetManager for FevmSubnetManager {
    async fn create_subnet(
        &self,
        from: Address,
        params: ConstructParams,
    ) -> anyhow::Result<Address> {
        self.evm_subnet_manager.create_subnet(from, params).await
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        validator_net_addr: String,
        worker_addr: Address,
    ) -> anyhow::Result<()> {
        self.evm_subnet_manager
            .join_subnet(subnet, from, collateral, validator_net_addr, worker_addr)
            .await
    }

    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> anyhow::Result<()> {
        self.evm_subnet_manager.leave_subnet(subnet, from).await
    }

    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> anyhow::Result<()> {
        self.evm_subnet_manager.kill_subnet(subnet, from).await
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> anyhow::Result<HashMap<SubnetID, SubnetInfo>> {
        self.evm_subnet_manager
            .list_child_subnets(gateway_addr)
            .await
    }

    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .fund(subnet, gateway_addr, from, to, amount)
            .await
    }

    async fn release(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .release(subnet, gateway_addr, from, to, amount)
            .await
    }

    async fn propagate(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_key: Vec<u8>,
    ) -> anyhow::Result<()> {
        self.evm_subnet_manager
            .propagate(subnet, gateway_addr, from, postbox_msg_key)
            .await
    }

    async fn send_cross_message(
        &self,
        gateway_addr: Address,
        from: Address,
        cross_msg: CrossMsg,
    ) -> anyhow::Result<()> {
        self.evm_subnet_manager
            .send_cross_message(gateway_addr, from, cross_msg)
            .await
    }

    async fn set_validator_net_addr(
        &self,
        subnet: SubnetID,
        from: Address,
        validator_net_addr: String,
    ) -> anyhow::Result<()> {
        self.evm_subnet_manager
            .set_validator_net_addr(subnet, from, validator_net_addr)
            .await
    }

    async fn send_value(
        &self,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<()> {
        self.evm_subnet_manager.send_value(from, to, amount).await
    }

    async fn wallet_balance(&self, address: &Address) -> anyhow::Result<TokenAmount> {
        self.evm_subnet_manager.wallet_balance(address).await
    }

    async fn last_topdown_executed(&self, gateway_addr: &Address) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .last_topdown_executed(gateway_addr)
            .await
    }

    async fn list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> anyhow::Result<Vec<NativeBottomUpCheckpoint>> {
        self.evm_subnet_manager
            .list_checkpoints(subnet_id, from_epoch, to_epoch)
            .await
    }

    async fn get_validator_set(
        &self,
        subnet_id: &SubnetID,
        gateway: Option<Address>,
        epoch: Option<ChainEpoch>,
    ) -> anyhow::Result<QueryValidatorSetResponse> {
        self.evm_subnet_manager
            .get_validator_set(subnet_id, gateway, epoch)
            .await
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
            .get_validator_set(subnet_id, None, None)
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
        start_epoch: ChainEpoch,
        end_epoch: ChainEpoch,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        self.evm_subnet_manager
            .top_down_msgs(subnet_id, start_epoch, end_epoch)
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
