// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::types::{InitExecParams, InitExecReturn, INIT_EXEC_METHOD_NUM};
use fil_actors_runtime::{builtin::singletons::INIT_ACTOR_ADDR, cbor};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_SEND;
use fvm_shared::{address::Address, econ::TokenAmount, MethodNum};
use ipc_gateway::{BottomUpCheckpoint, PropagateParams, WhitelistPropagatorParams};
use ipc_identity::Wallet;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{types::MANIFEST_ID, ConstructParams, JoinParams};

use crate::config::Subnet;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::ipc::SubnetInfo;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::message::state::StateWaitMsgResponse;
use crate::lotus::message::wallet::WalletKeyType;
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
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        // When we perform the fund, we should send to the gateway of the subnet's parent
        let parent = subnet.parent().ok_or_else(|| anyhow!("cannot fund root"))?;
        if !self.is_network_match(&parent).await? {
            return Err(anyhow!(
                "subnet actor being funded not matching current network"
            ));
        }

        let fund_params = cbor::serialize(&subnet, "fund subnet actor params")?;
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
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        // When we perform the release, we should send to the gateway of the subnet
        if !self.is_network_match(&subnet).await? {
            return Err(anyhow!(
                "subnet actor being released not matching current network"
            ));
        }

        let mut message = MpoolPushMessage::new(
            gateway_addr,
            from,
            ipc_gateway::Method::Release as MethodNum,
            vec![],
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

    async fn wallet_new(&self, key_type: WalletKeyType) -> Result<Address> {
        log::info!("creating new wallet");
        let addr_str = self.lotus_client.wallet_new(key_type).await?;
        Address::from_str(&addr_str).map_err(|_| anyhow!("cannot get address from string output"))
    }

    async fn wallet_list(&self) -> Result<Vec<Address>> {
        log::info!("list wallet in subnet");
        self.lotus_client
            .wallet_list()
            .await?
            .iter()
            .map(|raw| Address::from_str(raw).map_err(|_| anyhow!("invalid addr: {raw:}")))
            .collect::<Result<_>>()
    }

    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount> {
        log::info!("get the balance of an address");
        self.lotus_client.wallet_balance(address).await
    }

    async fn last_topdown_executed(&self) -> Result<ChainEpoch> {
        let head = self.lotus_client.chain_head().await?;
        let cid_map = head.cids.first().unwrap().clone();
        let tip_set = Cid::try_from(cid_map)?;
        let gw_state = self.lotus_client.ipc_read_gateway_state(tip_set).await?;

        Ok(gw_state.top_down_checkpoint_voting.last_voting_executed)
    }

    async fn list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> Result<Vec<BottomUpCheckpoint>> {
        let checkpoints = self
            .lotus_client
            .ipc_list_checkpoints(subnet_id, from_epoch, to_epoch)
            .await?;
        Ok(checkpoints)
    }
}

impl<T: JsonRpcClient + Send + Sync> LotusSubnetManager<T> {
    pub fn new(lotus_client: LotusJsonRPCClient<T>) -> Self {
        Self { lotus_client }
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
        LotusSubnetManager::new(client)
    }

    pub fn from_subnet_with_wallet_store(subnet: &Subnet, wallet: Arc<RwLock<Wallet>>) -> Self {
        let client = LotusJsonRPCClient::from_subnet_with_wallet_store(subnet, wallet);
        LotusSubnetManager::new(client)
    }
}
