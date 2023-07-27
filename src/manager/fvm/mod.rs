// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

mod conversion;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use crate::checkpoint::{
    create_proof, BottomUpHandler, CheckpointQuery, NativeBottomUpCheckpoint, TopDownHandler,
    VoteQuery,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::types::{InitExecParams, InitExecReturn, INIT_EXEC_METHOD_NUM};
use fil_actors_runtime::{builtin::singletons::INIT_ACTOR_ADDR, cbor};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_SEND;
use fvm_shared::{address::Address, econ::TokenAmount, MethodNum};
use ipc_gateway::{
    BottomUpCheckpoint, CrossMsg, FundParams, PropagateParams, ReleaseParams, TopDownCheckpoint,
    WhitelistPropagatorParams,
};
use ipc_identity::Wallet;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{types::MANIFEST_ID, ConstructParams, JoinParams};

use crate::config::Subnet;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::ipc::{
    IPCReadGatewayStateResponse, IPCReadSubnetActorStateResponse, QueryValidatorSetResponse,
    SubnetInfo,
};
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::message::state::StateWaitMsgResponse;
use crate::lotus::LotusClient;

use super::subnet::SubnetManager;

pub struct LotusSubnetManager<T: JsonRpcClient> {
    lotus_client: LotusJsonRPCClient<T>,
    gateway_addr: Address,
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> SubnetManager for LotusSubnetManager<T> {
    async fn create_subnet(&self, from: Address, params: ConstructParams) -> Result<Address> {
        if !self.is_network_match(&params.parent).await? {
            return Err(anyhow!("subnet actor being deployed in the wrong parent network, parent network names do not match"));
        }

        let exec_params = InitExecParams {
            code_cid: self.get_subnet_actor_code_cid().await?,
            constructor_params: cbor::serialize(&params, "create subnet actor")?,
        };
        log::debug!("create subnet for init actor with params: {exec_params:?}");
        let init_params = cbor::serialize(&exec_params, "init subnet actor params")?;
        let message = MpoolPushMessage::new(
            INIT_ACTOR_ADDR,
            from,
            INIT_EXEC_METHOD_NUM,
            init_params.to_vec(),
        );

        let state_wait_response = self.mpool_push_and_wait(message).await?;
        let result = state_wait_response
            .receipt
            .parse_result_into::<InitExecReturn>()?;
        let addr = result.robust_address;
        let id = result.id_address;
        log::info!("created subnet result - robust address: {addr:}, robust address: {id:}");

        Ok(addr)
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        validator_net_addr: String,
        worker_addr: Address,
    ) -> Result<()> {
        if from != worker_addr {
            return Err(anyhow!("worker address should equal sender"));
        }
        let parent = subnet.parent().ok_or_else(|| anyhow!("cannot join root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!("subnet actor being deployed in the wrong parent network, parent network names do not match"));
        }

        let to = subnet.subnet_actor();
        let mut message = MpoolPushMessage::new(
            to,
            from,
            ipc_subnet_actor::Method::Join as MethodNum,
            cbor::serialize(&JoinParams { validator_net_addr }, "join subnet params")?.to_vec(),
        );
        message.value = collateral;

        self.mpool_push_and_wait(message).await?;
        log::info!("joined subnet: {subnet:}");

        Ok(())
    }

    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        let parent = subnet
            .parent()
            .ok_or_else(|| anyhow!("cannot leave root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!("subnet actor being deployed in the wrong parent network, parent network names do not match"));
        }

        self.mpool_push_and_wait(MpoolPushMessage::new(
            subnet.subnet_actor(),
            from,
            ipc_subnet_actor::Method::Leave as MethodNum,
            vec![],
        ))
        .await?;
        log::info!("left subnet: {subnet:}");

        Ok(())
    }

    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        let parent = subnet.parent().ok_or_else(|| anyhow!("cannot kill root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!("subnet actor being deployed in the wrong parent network, parent network names do not match"));
        }

        self.mpool_push_and_wait(MpoolPushMessage::new(
            subnet.subnet_actor(),
            from,
            ipc_subnet_actor::Method::Kill as MethodNum,
            vec![],
        ))
        .await?;
        log::info!("left subnet: {subnet:}");

        Ok(())
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>> {
        let subnets = self
            .lotus_client
            .ipc_list_child_subnets(gateway_addr)
            .await?;

        log::debug!("received subnets: {subnets:?}");

        let mut map = HashMap::new();
        for s in subnets {
            map.insert(s.id.clone(), s);
        }

        log::debug!("converted to subnets: {map:?}");

        Ok(map)
    }

    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        // When we perform the fund, we should send to the gateway of the subnet's parent
        let parent = subnet.parent().ok_or_else(|| anyhow!("cannot fund root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!(
                "subnet actor being funded not matching current network"
            ));
        }

        let fund_params = cbor::serialize(&FundParams { subnet, to }, "fund subnet actor params")?;
        let mut message = MpoolPushMessage::new(
            gateway_addr,
            from,
            ipc_gateway::Method::Fund as MethodNum,
            fund_params.to_vec(),
        );
        message.value = amount;
        let r = self.mpool_push_and_wait(message).await?;
        Ok(r.height as ChainEpoch)
    }

    async fn release(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        // When we perform the release, we should send to the gateway of the subnet
        if !self.is_network_match(&subnet).await? {
            return Err(anyhow!(
                "subnet actor being released not matching current network"
            ));
        }

        let release_params = cbor::serialize(&ReleaseParams { to }, "fund subnet actor params")?;
        let mut message = MpoolPushMessage::new(
            gateway_addr,
            from,
            ipc_gateway::Method::Release as MethodNum,
            release_params.to_vec(),
        );
        message.value = amount;

        let r = self.mpool_push_and_wait(message).await?;
        Ok(r.height as ChainEpoch)
    }

    async fn propagate(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_cid: Cid,
    ) -> Result<()> {
        if !self.is_network_match(&subnet).await? {
            return Err(anyhow!("propagation not targeting the correct network"));
        }

        let params = cbor::serialize(
            &PropagateParams {
                postbox_cid: postbox_msg_cid,
            },
            "propagate params",
        )?;

        let message = MpoolPushMessage::new(
            gateway_addr,
            from,
            ipc_gateway::Method::Propagate as MethodNum,
            params.to_vec(),
        );

        self.mpool_push_and_wait(message).await?;
        Ok(())
    }

    async fn set_validator_net_addr(
        &self,
        subnet: SubnetID,
        from: Address,
        validator_net_addr: String,
    ) -> Result<()> {
        // When we set the validator net addr, we should send to the subnet's parent
        let parent = subnet.parent().ok_or_else(|| anyhow!("cannot fund root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!(
                "set validator net addr not targeting the correct parent network"
            ));
        }

        let params = cbor::serialize(
            &JoinParams { validator_net_addr },
            "set validator net addr params",
        )?;

        let message = MpoolPushMessage::new(
            subnet.subnet_actor(),
            from,
            ipc_subnet_actor::Method::SetValidatorNetAddr as MethodNum,
            params.to_vec(),
        );

        self.mpool_push_and_wait(message).await?;
        Ok(())
    }

    async fn whitelist_propagator(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        postbox_msg_cid: Cid,
        from: Address,
        to_add: Vec<Address>,
    ) -> Result<()> {
        if !self.is_network_match(&subnet).await? {
            return Err(anyhow!("whitelist not targeting the correct network"));
        }

        let params = cbor::serialize(
            &WhitelistPropagatorParams {
                postbox_cid: postbox_msg_cid,
                to_add,
            },
            "whitelist propagate params",
        )?;

        let message = MpoolPushMessage::new(
            gateway_addr,
            from,
            ipc_gateway::Method::WhiteListPropagator as MethodNum,
            params.to_vec(),
        );

        self.mpool_push_and_wait(message).await?;
        Ok(())
    }

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()> {
        let mut message = MpoolPushMessage::new(to, from, METHOD_SEND, Vec::new());
        message.value = amount;
        self.mpool_push_and_wait(message).await?;
        log::info!("sending FIL from {from:} to {to:}");

        Ok(())
    }

    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount> {
        log::info!("get the balance of an address");
        self.lotus_client.wallet_balance(address).await
    }

    async fn last_topdown_executed(&self, gateway_addr: &Address) -> Result<ChainEpoch> {
        let head = self.lotus_client.chain_head().await?;
        let cid_map = head.cids.first().unwrap().clone();
        let tip_set = Cid::try_from(cid_map)?;
        let gw_state = self
            .lotus_client
            .ipc_read_gateway_state(gateway_addr, tip_set)
            .await?;

        Ok(gw_state.top_down_checkpoint_voting.last_voting_executed)
    }

    async fn list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> Result<Vec<NativeBottomUpCheckpoint>> {
        let checkpoints = self
            .lotus_client
            .ipc_list_checkpoints(subnet_id, from_epoch, to_epoch)
            .await?
            .into_iter()
            .map(NativeBottomUpCheckpoint::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(checkpoints)
    }

    async fn get_validator_set(
        &self,
        subnet_id: &SubnetID,
        gateway: Option<Address>,
    ) -> Result<QueryValidatorSetResponse> {
        let gateway = gateway.ok_or_else(|| anyhow!("gateway address needed"))?;

        let chain_head = self.lotus_client.chain_head().await?;
        let cid_map = chain_head.cids.first().unwrap().clone();
        let tip_set = Cid::try_from(cid_map)?;

        let response = self
            .lotus_client
            .ipc_read_subnet_actor_state(subnet_id, tip_set)
            .await?;

        let genesis_epoch = self
            .lotus_client
            .ipc_get_genesis_epoch_for_subnet(subnet_id, gateway)
            .await?;

        let mut validator_set = response.validator_set;
        if let Some(validators) = validator_set.validators.as_mut() {
            validators
                .iter_mut()
                .for_each(|v| v.worker_addr = Some(v.addr.clone()));
        }
        Ok(QueryValidatorSetResponse {
            validator_set,
            min_validators: response.min_validators,
            genesis_epoch,
        })
    }
}

impl<T: JsonRpcClient + Send + Sync> LotusSubnetManager<T> {
    pub fn new(lotus_client: LotusJsonRPCClient<T>, gateway_addr: Address) -> Self {
        Self {
            lotus_client,
            gateway_addr,
        }
    }

    async fn get_subnet_state(
        &self,
        subnet_id: &SubnetID,
    ) -> Result<IPCReadSubnetActorStateResponse> {
        let head = self.lotus_client.chain_head().await?;

        // A key assumption we make now is that each block has exactly one tip set. We panic
        // if this is not the case as it violates our assumption.
        // TODO: update this logic once the assumption changes (i.e., mainnet)
        assert_eq!(head.cids.len(), 1);

        let cid_map = head.cids.first().unwrap().clone();
        let tip_set = Cid::try_from(cid_map)?;

        self.lotus_client
            .ipc_read_subnet_actor_state(subnet_id, tip_set)
            .await
    }

    /// Publish the message to memory pool and wait for the response
    async fn mpool_push_and_wait(&self, message: MpoolPushMessage) -> Result<StateWaitMsgResponse> {
        let message_cid = self.lotus_client.mpool_push(message).await?;
        log::debug!("message published with cid: {message_cid:?}");

        self.lotus_client.state_wait_msg(message_cid).await
    }

    /// Checks the `network` is the one we are currently talking to.
    async fn is_network_match(&self, network: &SubnetID) -> Result<bool> {
        let network_name = self.lotus_client.state_network_name().await?;
        log::debug!(
            "current network name: {network_name:?}, to check network: {:?}",
            network.to_string()
        );

        Ok(network.to_string() == network_name)
    }

    /// Obtain the actor code cid of `ipc_subnet_actor` only, since this is the
    /// code cid we are interested in.
    async fn get_subnet_actor_code_cid(&self) -> Result<Cid> {
        let network_version = self.lotus_client.state_network_version(vec![]).await?;
        log::debug!("received network version: {network_version:?}");

        let mut cid_map = self
            .lotus_client
            .state_actor_code_cids(network_version)
            .await?;

        cid_map
            .remove(MANIFEST_ID)
            .ok_or_else(|| anyhow!("actor cid not found"))
    }
}

impl LotusSubnetManager<JsonRpcClientImpl> {
    pub fn from_subnet(subnet: &Subnet) -> Self {
        let client = LotusJsonRPCClient::from_subnet(subnet);
        LotusSubnetManager::new(client, subnet.gateway_addr())
    }

    pub fn from_subnet_with_wallet_store(subnet: &Subnet, wallet: Arc<RwLock<Wallet>>) -> Self {
        let client = LotusJsonRPCClient::from_subnet_with_wallet_store(subnet, wallet);
        LotusSubnetManager::new(client, subnet.gateway_addr())
    }
}

impl<T: JsonRpcClient + Send + Sync> LotusSubnetManager<T> {
    async fn parent_head(&self) -> Result<Cid> {
        chain_head_cid(&self.lotus_client).await
    }

    async fn submission_tipset(&self, epoch: ChainEpoch) -> anyhow::Result<Cid> {
        let submission_tip_set = self
            .lotus_client
            .get_tipset_by_height(epoch, self.parent_head().await?)
            .await?;
        let cid_map = submission_tip_set.cids.first().unwrap().clone();
        Cid::try_from(cid_map)
    }

    async fn child_head(&self) -> Result<Cid> {
        chain_head_cid(&self.lotus_client).await
    }

    pub async fn gateway_state(&self) -> Result<IPCReadGatewayStateResponse> {
        gateway_state(&self.lotus_client, &self.gateway_addr).await
    }

    async fn get_validators(&self, subnet_id: &SubnetID) -> Result<Vec<Address>> {
        let subnet_actor_state = self.get_subnet_state(subnet_id).await?;
        subnet_actor_state
            .validator_set
            .validators
            .unwrap_or_default()
            .iter()
            .map(|f| Address::from_str(&f.addr).map_err(|e| anyhow!("cannot create address: {e:}")))
            .collect::<Result<_>>()
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> VoteQuery<NativeBottomUpCheckpoint> for LotusSubnetManager<T> {
    async fn last_executed_epoch(&self, subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let subnet_actor_state = self.get_subnet_state(subnet_id).await?;

        Ok(subnet_actor_state
            .bottom_up_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> Result<ChainEpoch> {
        self.lotus_client.current_epoch().await
    }

    async fn has_voted(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> Result<bool> {
        log::debug!(
            "attempt to obtain the next submission epoch in bottom up checkpoint for subnet: {:?}",
            subnet_id
        );

        let has_voted = self
            .lotus_client
            .ipc_validator_has_voted_bottomup(subnet_id, epoch, validator)
            .await?;

        // we should vote only when the validator has not voted
        Ok(!has_voted)
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> CheckpointQuery<NativeBottomUpCheckpoint>
    for LotusSubnetManager<T>
{
    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let tip_set = chain_head_cid(&self.lotus_client).await?;
        let state = self
            .lotus_client
            .ipc_read_subnet_actor_state(subnet_id, tip_set)
            .await
            .map_err(|e| {
                log::error!("error getting subnet actor state for {:?}", subnet_id);
                e
            })?;

        Ok(state.bottom_up_check_period)
    }

    async fn validators(&self, subnet_id: &SubnetID) -> Result<Vec<Address>> {
        self.get_validators(subnet_id).await
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> BottomUpHandler for LotusSubnetManager<T> {
    async fn checkpoint_template(&self, epoch: ChainEpoch) -> Result<NativeBottomUpCheckpoint> {
        let template = self
            .lotus_client
            .ipc_get_checkpoint_template(&self.gateway_addr, epoch)
            .await
            .map_err(|e| {
                anyhow!(
                    "error getting bottom-up checkpoint template for epoch:{epoch:} due to {e:}"
                )
            })?;

        let mut checkpoint = BottomUpCheckpoint::new(template.source().clone(), epoch);
        checkpoint.data.children = template.data.children;
        checkpoint.data.cross_msgs = template.data.cross_msgs;
        log::debug!("raw bottom up templated: {checkpoint:?}");

        NativeBottomUpCheckpoint::try_from(checkpoint)
    }

    async fn populate_prev_hash(
        &self,
        template: &mut NativeBottomUpCheckpoint,
        subnet: &SubnetID,
        previous_epoch: ChainEpoch,
    ) -> Result<()> {
        let response = self
            .lotus_client
            .ipc_get_prev_checkpoint_for_child(&self.gateway_addr, subnet)
            .await
            .map_err(|e| {
                anyhow!(
                    "error getting previous bottom-up checkpoint for epoch:{:} in subnet: {:?} due to {e:}",
                    previous_epoch,
                    subnet
                )
            })?;

        // if previous checkpoint is set
        if let Some(cid_map) = response {
            let cid = Cid::try_from(cid_map)?;
            template.prev_check = Some(cid.to_bytes());
        } else {
            template.prev_check = None;
        }

        Ok(())
    }

    async fn populate_proof(&self, template: &mut NativeBottomUpCheckpoint) -> Result<()> {
        let proof = create_proof(&self.lotus_client, template.epoch).await?;
        let bytes = cbor::serialize(&proof, "fvm bottom up checkpoint proof")?.to_vec();
        template.proof = Some(bytes);
        Ok(())
    }

    async fn submit(
        &self,
        validator: &Address,
        checkpoint: NativeBottomUpCheckpoint,
    ) -> Result<ChainEpoch> {
        let to = checkpoint.source.subnet_actor();
        let message = MpoolPushMessage::new(
            to,
            *validator,
            ipc_subnet_actor::Method::SubmitCheckpoint as MethodNum,
            cbor::serialize(&BottomUpCheckpoint::try_from(&checkpoint)?, "checkpoint")?.to_vec(),
        );
        let message_cid = self.lotus_client.mpool_push(message).await.map_err(|e| {
            anyhow!(
                "error submitting checkpoint for epoch {:} in subnet: {:?} with reason {e:}",
                checkpoint.epoch,
                checkpoint.source
            )
        })?;
        log::debug!("checkpoint message published with cid: {message_cid:?}");

        Ok(self.lotus_client.state_wait_msg(message_cid).await?.height as ChainEpoch)
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> VoteQuery<TopDownCheckpoint> for LotusSubnetManager<T> {
    async fn last_executed_epoch(&self, _subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let child_gw_state = self.gateway_state().await?;
        Ok(child_gw_state
            .top_down_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> Result<ChainEpoch> {
        self.lotus_client.current_epoch().await
    }

    async fn has_voted(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> Result<bool> {
        let has_voted = self
            .lotus_client
            .ipc_validator_has_voted_topdown(&self.gateway_addr, epoch, validator)
            .await
            .map_err(|e| {
                anyhow!(
                    "error checking if validator has voted for subnet: {subnet_id:} due to {e:}"
                )
            })?;
        Ok(has_voted)
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> CheckpointQuery<TopDownCheckpoint> for LotusSubnetManager<T> {
    async fn checkpoint_period(&self, _subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let tip_set = chain_head_cid(&self.lotus_client).await?;
        let state = self
            .lotus_client
            .ipc_read_gateway_state(&self.gateway_addr, tip_set)
            .await?;
        Ok(state.top_down_check_period)
    }

    async fn validators(&self, subnet_id: &SubnetID) -> Result<Vec<Address>> {
        self.get_validators(subnet_id).await
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> TopDownHandler for LotusSubnetManager<T> {
    async fn gateway_initialized(&self) -> Result<bool> {
        let state = self.gateway_state().await?;
        Ok(state.initialized)
    }

    async fn applied_topdown_nonce(&self, _subnet_id: &SubnetID) -> Result<u64> {
        Ok(self
            .lotus_client
            .ipc_read_gateway_state(&self.gateway_addr, self.child_head().await?)
            .await?
            .applied_topdown_nonce)
    }

    async fn top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        nonce: u64,
        epoch: ChainEpoch,
    ) -> Result<Vec<CrossMsg>> {
        let submission_tip_set = self.submission_tipset(epoch).await?;
        let top_down_msgs = self
            .lotus_client
            .ipc_get_topdown_msgs(subnet_id, &self.gateway_addr, submission_tip_set, nonce)
            .await?;

        log::debug!(
            "nonce: {:} for submission tip set: {:} at epoch {:} of subnet: {:}, size of top down messages: {:}",
            nonce, submission_tip_set, epoch, subnet_id, top_down_msgs.len()
        );

        Ok(top_down_msgs)
    }

    async fn submit(
        &self,
        validator: &Address,
        checkpoint: TopDownCheckpoint,
    ) -> Result<ChainEpoch> {
        let submitted_epoch = self
            .lotus_client
            .ipc_submit_top_down_checkpoint(self.gateway_addr, validator, checkpoint)
            .await?;

        Ok(submitted_epoch)
    }
}

pub async fn gateway_state(
    client: &(impl LotusClient + Sync),
    gateway_addr: &Address,
) -> Result<IPCReadGatewayStateResponse> {
    let head = client.chain_head().await?;
    let cid_map = head.cids.first().unwrap().clone();
    let tip_set = Cid::try_from(cid_map)?;

    client.ipc_read_gateway_state(gateway_addr, tip_set).await
}

/// Returns the first cid in the chain head
pub async fn chain_head_cid(client: &(impl LotusClient + Sync)) -> anyhow::Result<Cid> {
    let child_head = client.chain_head().await?;
    let cid_map = child_head.cids.first().unwrap();
    Cid::try_from(cid_map)
}
