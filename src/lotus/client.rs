// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use num_traits::cast::ToPrimitive;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl, NO_PARAMS};
use crate::lotus::json::ToJson;
use crate::lotus::message::chain::ChainHeadResponse;
use crate::lotus::message::ipc::{
    IPCReadGatewayStateResponse, IPCReadSubnetActorStateResponse, Votes,
};
use crate::lotus::message::mpool::{
    MpoolPushMessage, MpoolPushMessageResponse, MpoolPushMessageResponseInner,
};
use crate::lotus::message::state::{ReadStateResponse, StateWaitMsgResponse};
use crate::lotus::message::wallet::{WalletKeyType, WalletListResponse};
use crate::lotus::message::CIDMap;
use crate::lotus::{LotusClient, NetworkVersion};
use crate::manager::SubnetInfo;

use super::message::ipc::CheckpointResponse;

// RPC methods
mod methods {
    pub const MPOOL_PUSH_MESSAGE: &str = "Filecoin.MpoolPushMessage";
    pub const STATE_WAIT_MSG: &str = "Filecoin.StateWaitMsg";
    pub const STATE_NETWORK_NAME: &str = "Filecoin.StateNetworkName";
    pub const STATE_NETWORK_VERSION: &str = "Filecoin.StateNetworkVersion";
    pub const STATE_ACTOR_CODE_CIDS: &str = "Filecoin.StateActorCodeCIDs";
    pub const WALLET_NEW: &str = "Filecoin.WalletNew";
    pub const WALLET_LIST: &str = "Filecoin.WalletList";
    pub const WALLET_DEFAULT_ADDRESS: &str = "Filecoin.WalletDefaultAddress";
    pub const STATE_READ_STATE: &str = "Filecoin.StateReadState";
    pub const CHAIN_HEAD: &str = "Filecoin.ChainHead";
    pub const IPC_GET_PREV_CHECKPOINT_FOR_CHILD: &str = "Filecoin.IPCGetPrevCheckpointForChild";
    pub const IPC_GET_CHECKPOINT_TEMPLATE: &str = "Filecoin.IPCGetCheckpointTemplate";
    pub const IPC_GET_CHECKPOINT: &str = "Filecoin.IPCGetCheckpoint";
    pub const IPC_READ_GATEWAY_STATE: &str = "Filecoin.IPCReadGatewayState";
    pub const IPC_READ_SUBNET_ACTOR_STATE: &str = "Filecoin.IPCReadSubnetActorState";
    pub const IPC_LIST_CHILD_SUBNETS: &str = "Filecoin.IPCListChildSubnets";
    pub const IPC_GET_VOTES_FOR_CHECKPOINT: &str = "Filecoin.IPCGetVotesForCheckpoint";
    pub const IPC_LIST_CHECKPOINTS: &str = "Filecoin.IPCListCheckpoints";
}

/// The default gateway actor address
const GATEWAY_ACTOR_ADDRESS: &str = "t064";
/// The default state wait confidence value
/// TODO: we can afford 2 epochs confidence (and even one)
/// with Mir, but with Filecoin mainnet this should be increased
/// in case there are reorgs.
const STATE_WAIT_CONFIDENCE: u8 = 2;
/// We dont set a limit on the look back epoch, i.e. check against latest block
const STATE_WAIT_LOOK_BACK_NO_LIMIT: i8 = -1;
/// We are not replacing any previous messages.
/// TODO: when set to false, lotus raises `found message with equal nonce as the one we are looking`
/// TODO: error. Should check this again.
const STATE_WAIT_ALLOW_REPLACE: bool = true;

/// The struct implementation for Lotus Client API. It allows for multiple different trait
/// extension.
/// # Examples
/// ```no_run
/// use ipc_agent::{jsonrpc::JsonRpcClientImpl, lotus::LotusClient, lotus::client::LotusJsonRPCClient};
///
/// #[tokio::main]
/// async fn main() {
///     let h = JsonRpcClientImpl::new("<DEFINE YOUR URL HERE>".parse().unwrap(), None);
///     let n = LotusJsonRPCClient::new(h);
///     println!(
///         "wallets: {:?}",
///         n.wallet_new(ipc_agent::lotus::message::wallet::WalletKeyType::Secp256k1).await
///     );
/// }
/// ```
pub struct LotusJsonRPCClient<T: JsonRpcClient> {
    client: T,
}

impl<T: JsonRpcClient> LotusJsonRPCClient<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> LotusClient for LotusJsonRPCClient<T> {
    async fn mpool_push_message(
        &self,
        msg: MpoolPushMessage,
    ) -> Result<MpoolPushMessageResponseInner> {
        let nonce = msg
            .nonce
            .map(|n| serde_json::Value::Number(n.into()))
            .unwrap_or(serde_json::Value::Null);

        let to_value = |t: Option<TokenAmount>| {
            t.map(|n| serde_json::Value::Number(n.atto().to_u64().unwrap().into()))
                .unwrap_or(serde_json::Value::Null)
        };
        let gas_limit = to_value(msg.gas_limit);
        let gas_premium = to_value(msg.gas_premium);
        let gas_fee_cap = to_value(msg.gas_fee_cap);
        let max_fee = to_value(msg.max_fee);

        // refer to: https://lotus.filecoin.io/reference/lotus/mpool/#mpoolpushmessage
        let params = json!([
            {
                "to": msg.to.to_string(),
                "from": msg.from.to_string(),
                "value": msg.value.atto().to_string(),
                "method": msg.method,
                "params": msg.params,

                // THESE ALL WILL AUTO POPULATE if null
                "nonce": nonce,
                "gas_limit": gas_limit,
                "gas_fee_cap": gas_fee_cap,
                "gas_premium": gas_premium,
                "cid": CIDMap::from(msg.cid),
                "version": serde_json::Value::Null,
            },
            {
                "max_fee": max_fee
            }
        ]);

        let r = self
            .client
            .request::<MpoolPushMessageResponse>(methods::MPOOL_PUSH_MESSAGE, params)
            .await?;
        log::debug!("received mpool_push_message response: {r:?}");

        Ok(r.message)
    }

    async fn state_wait_msg(&self, cid: Cid) -> Result<StateWaitMsgResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statewaitmsg
        let params = json!([
            CIDMap::from(cid),
            STATE_WAIT_CONFIDENCE,
            STATE_WAIT_LOOK_BACK_NO_LIMIT,
            STATE_WAIT_ALLOW_REPLACE,
        ]);

        let r = self
            .client
            .request::<StateWaitMsgResponse>(methods::STATE_WAIT_MSG, params)
            .await?;
        log::debug!("received state_wait_msg response: {r:?}");
        Ok(r)
    }

    async fn state_network_name(&self) -> Result<String> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statenetworkname
        let r = self
            .client
            .request::<String>(methods::STATE_NETWORK_NAME, serde_json::Value::Null)
            .await?;
        log::debug!("received state_network_name response: {r:?}");
        Ok(r)
    }

    async fn state_network_version(&self, tip_sets: Vec<Cid>) -> Result<NetworkVersion> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statenetworkversion
        let params = json!([tip_sets.into_iter().map(CIDMap::from).collect::<Vec<_>>()]);

        let r = self
            .client
            .request::<NetworkVersion>(methods::STATE_NETWORK_VERSION, params)
            .await?;

        log::debug!("received state_network_version response: {r:?}");
        Ok(r)
    }

    async fn state_actor_code_cids(
        &self,
        network_version: NetworkVersion,
    ) -> Result<HashMap<String, Cid>> {
        // refer to: https://github.com/filecoin-project/lotus/blob/master/documentation/en/api-v1-unstable-methods.md#stateactormanifestcid
        let params = json!([network_version]);

        let r = self
            .client
            .request::<HashMap<String, CIDMap>>(methods::STATE_ACTOR_CODE_CIDS, params)
            .await?;

        let mut cids = HashMap::new();
        for (key, cid_map) in r.into_iter() {
            cids.insert(key, Cid::try_from(cid_map)?);
        }

        log::debug!("received state_actor_manifest_cid response: {cids:?}");
        Ok(cids)
    }

    async fn wallet_default(&self) -> Result<Address> {
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletdefaultaddress
        let r = self
            .client
            .request::<String>(methods::WALLET_DEFAULT_ADDRESS, json!({}))
            .await?;
        log::debug!("received wallet_default response: {r:?}");

        let addr = Address::from_str(&r)?;
        Ok(addr)
    }

    async fn wallet_list(&self) -> Result<WalletListResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletlist
        let r = self
            .client
            .request::<WalletListResponse>(methods::WALLET_LIST, json!({}))
            .await?;
        log::debug!("received wallet_list response: {r:?}");
        Ok(r)
    }

    async fn wallet_new(&self, key_type: WalletKeyType) -> Result<String> {
        let key_type_str = key_type.as_ref();
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletnew
        let r = self
            .client
            .request::<String>(methods::WALLET_NEW, json!([key_type_str]))
            .await?;
        log::debug!("received wallet_new response: {r:?}");
        Ok(r)
    }

    async fn read_state<State: DeserializeOwned + Debug>(
        &self,
        address: Address,
        tipset: Cid,
    ) -> Result<ReadStateResponse<State>> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statereadstate
        let r = self
            .client
            .request::<ReadStateResponse<State>>(
                methods::STATE_READ_STATE,
                json!([address.to_string(), [CIDMap::from(tipset)]]),
            )
            .await?;
        log::debug!("received read_state response: {r:?}");
        Ok(r)
    }

    async fn chain_head(&self) -> Result<ChainHeadResponse> {
        let r = self
            .client
            .request::<ChainHeadResponse>(methods::CHAIN_HEAD, NO_PARAMS)
            .await?;
        log::debug!("received chain_head response: {r:?}");
        Ok(r)
    }

    async fn ipc_get_prev_checkpoint_for_child(
        &self,
        child_subnet_id: SubnetID,
    ) -> Result<Option<CIDMap>> {
        if child_subnet_id.parent().is_none() {
            return Err(anyhow!("The child_subnet_id must be a valid child subnet"));
        }
        let params = json!([GATEWAY_ACTOR_ADDRESS, child_subnet_id.to_json()]);

        let r = self
            .client
            .request::<Option<CIDMap>>(methods::IPC_GET_PREV_CHECKPOINT_FOR_CHILD, params)
            .await?;
        Ok(r)
    }

    async fn ipc_get_checkpoint_template(&self, epoch: ChainEpoch) -> Result<Checkpoint> {
        let r = self
            .client
            .request::<CheckpointResponse>(
                methods::IPC_GET_CHECKPOINT_TEMPLATE,
                json!([GATEWAY_ACTOR_ADDRESS, epoch]),
            )
            .await?;

        Ok(Checkpoint::try_from(r)?)
    }

    async fn ipc_get_checkpoint(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> Result<Checkpoint> {
        let params = json!([subnet_id.to_json(), epoch]);
        let r = self
            .client
            .request::<CheckpointResponse>(methods::IPC_GET_CHECKPOINT, params)
            .await
            .map_err(|e| {
                log::debug!(
                    "error getting checkpoint for epoch {epoch:} in subnet {:?}: {}",
                    subnet_id,
                    e.to_string()
                );
                e
            })?;

        Ok(Checkpoint::try_from(r)?)
    }

    async fn ipc_read_gateway_state(&self, tip_set: Cid) -> Result<IPCReadGatewayStateResponse> {
        let params = json!([GATEWAY_ACTOR_ADDRESS, [CIDMap::from(tip_set)]]);
        let r = self
            .client
            .request::<IPCReadGatewayStateResponse>(methods::IPC_READ_GATEWAY_STATE, params)
            .await?;
        Ok(r)
    }

    async fn ipc_read_subnet_actor_state(
        &self,
        subnet_id: &SubnetID,
        tip_set: Cid,
    ) -> Result<IPCReadSubnetActorStateResponse> {
        let params = json!([subnet_id.to_json(), [CIDMap::from(tip_set)]]);
        log::debug!("sending {params:?}");

        let r = self
            .client
            .request::<IPCReadSubnetActorStateResponse>(
                methods::IPC_READ_SUBNET_ACTOR_STATE,
                params,
            )
            .await?;
        Ok(r)
    }

    async fn ipc_list_child_subnets(&self, gateway_addr: Address) -> Result<Vec<SubnetInfo>> {
        let params = json!([gateway_addr.to_string()]);
        let r = self
            .client
            .request::<Option<Vec<SubnetInfo>>>(methods::IPC_LIST_CHILD_SUBNETS, params)
            .await?;
        Ok(r.unwrap_or_default())
    }

    async fn ipc_get_votes_for_checkpoint(
        &self,
        subnet_id: SubnetID,
        checkpoint_cid: Cid,
    ) -> Result<Votes> {
        let params = json!([subnet_id.to_json(), CIDMap::from(checkpoint_cid)]);
        let r = self
            .client
            .request::<Votes>(methods::IPC_GET_VOTES_FOR_CHECKPOINT, params)
            .await?;
        Ok(r)
    }

    async fn ipc_list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> Result<Vec<CheckpointResponse>> {
        let parent = subnet_id
            .parent()
            .ok_or_else(|| anyhow!("no parent found"))?
            .to_string();
        let actor = subnet_id.subnet_actor().to_string();
        let params = json!([
            {
                "Parent": parent,
                "Actor": actor
            },
            from_epoch,
            to_epoch
        ]);
        let r = self
            .client
            .request::<Vec<CheckpointResponse>>(methods::IPC_LIST_CHECKPOINTS, params)
            .await?;
        Ok(r)
    }
}

impl LotusJsonRPCClient<JsonRpcClientImpl> {
    /// A constructor that returns a `LotusJsonRPCClient` from a `Subnet`. The returned
    /// `LotusJsonRPCClient` makes requests to the URL defined in the `Subnet`.
    pub fn from_subnet(subnet: &crate::config::Subnet) -> Self {
        let url = subnet.jsonrpc_api_http.clone();
        let auth_token = subnet.auth_token.as_deref();
        let jsonrpc_client = JsonRpcClientImpl::new(url, auth_token);
        LotusJsonRPCClient::new(jsonrpc_client)
    }
}
