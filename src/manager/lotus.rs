// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;

use crate::config::Subnet;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::types::{InitExecParams, InitExecReturn, INIT_EXEC_METHOD_NUM};
use fil_actors_runtime::{builtin::singletons::INIT_ACTOR_ADDR, cbor};
use fvm_shared::{address::Address, econ::TokenAmount, MethodNum};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{types::MANIFEST_ID, ConstructParams, JoinParams};

use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::ipc::SubnetInfo;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::message::state::StateWaitMsgResponse;
use crate::lotus::LotusClient;

use super::subnet::SubnetManager;

pub struct LotusSubnetManager<T: JsonRpcClient> {
    lotus_client: LotusJsonRPCClient<T>,
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
        let addr = result.id_address;
        log::info!("created subnet result: {addr:}");

        Ok(addr)
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        params: JoinParams,
    ) -> Result<()> {
        let parent = subnet.parent().ok_or_else(|| anyhow!("cannot join root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!("subnet actor being deployed in the wrong parent network, parent network names do not match"));
        }

        let to = subnet.subnet_actor();
        let mut message = MpoolPushMessage::new(
            to,
            from,
            ipc_subnet_actor::Method::Join as MethodNum,
            cbor::serialize(&params, "join subnet params")?.to_vec(),
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

    async fn submit_checkpoint(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _ch: Checkpoint,
    ) -> Result<()> {
        panic!("not implemented")
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>> {
        let subnets = self
            .lotus_client
            .ipc_list_child_subnets(gateway_addr)
            .await?;

        let mut map = HashMap::new();
        for s in subnets {
            map.insert(s.id.clone(), s);
        }
        Ok(map)
    }

    async fn fund(&self, _subnet: SubnetID, _from: Address, _amount: TokenAmount) -> Result<()> {
        todo!()
    }

    async fn release(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        todo!()
    }

    async fn propagate(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _postbox_msg_cid: Cid,
    ) -> Result<()> {
        todo!()
    }

    async fn whitelist_propagator(
        &self,
        _subnet: SubnetID,
        _postbox_msg_cid: Cid,
        _from: Address,
        _to_add: Vec<Address>,
    ) -> Result<()> {
        todo!()
    }
}

impl<T: JsonRpcClient + Send + Sync> LotusSubnetManager<T> {
    pub fn new(lotus_client: LotusJsonRPCClient<T>) -> Self {
        Self { lotus_client }
    }

    /// Publish the message to memory pool and wait for the response
    async fn mpool_push_and_wait(&self, message: MpoolPushMessage) -> Result<StateWaitMsgResponse> {
        let mem_push_response = self.lotus_client.mpool_push_message(message).await?;

        let message_cid = mem_push_response.cid()?;
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
        LotusSubnetManager::new(client)
    }
}
