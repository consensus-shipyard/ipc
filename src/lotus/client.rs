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

use crate::config::Subnet;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl, NO_PARAMS};
use crate::lotus::message::chain::ChainHeadResponse;
use crate::lotus::message::ipc::{
    IPCGetPrevCheckpointForChildResponse, IPCReadGatewayStateResponse,
    IPCReadSubnetActorStateResponse,
};
use crate::lotus::message::mpool::{
    MpoolPushMessage, MpoolPushMessageResponse, MpoolPushMessageResponseInner,
};
use crate::lotus::message::state::{ReadStateResponse, StateWaitMsgResponse};
use crate::lotus::message::wallet::{WalletKeyType, WalletListResponse};
use crate::lotus::message::CIDMap;
use crate::lotus::{LotusClient, NetworkVersion};

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
    pub const IPC_READ_GATEWAY_STATE: &str = "Filecoin.IPCReadGatewayState";
    pub const IPC_READ_SUBNET_ACTOR_STATE: &str = "Filecoin.IPCReadSubnetActorState";
}

/// The default gateway actor address
const GATEWAY_ACTOR_ADDRESS: &str = "f064";

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

    async fn state_wait_msg(&self, cid: Cid, nonce: u64) -> Result<StateWaitMsgResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statewaitmsg
        let params = json!([CIDMap::from(cid), nonce]);

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
    ) -> Result<IPCGetPrevCheckpointForChildResponse> {
        let parent = match child_subnet_id.parent() {
            None => return Err(anyhow!("The child_subnet_id must be a valid child subnet")),
            Some(parent) => parent,
        };
        let subnet_actor = child_subnet_id.subnet_actor().to_string();
        let params =
            json!([GATEWAY_ACTOR_ADDRESS, {"Parent": parent.to_string(), "Actor": subnet_actor}]);

        let r = self
            .client
            .request::<IPCGetPrevCheckpointForChildResponse>(
                methods::IPC_GET_PREV_CHECKPOINT_FOR_CHILD,
                params,
            )
            .await?;
        Ok(r)
    }

    async fn ipc_get_checkpoint_template(&self, epoch: ChainEpoch) -> Result<Checkpoint> {
        let r = self
            .client
            .request::<Checkpoint>(
                methods::IPC_GET_CHECKPOINT_TEMPLATE,
                json!([GATEWAY_ACTOR_ADDRESS, epoch]),
            )
            .await?;
        Ok(r)
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
        tip_set: Cid,
    ) -> Result<IPCReadSubnetActorStateResponse> {
        let params = json!([GATEWAY_ACTOR_ADDRESS, [CIDMap::from(tip_set)]]);
        let r = self
            .client
            .request::<IPCReadSubnetActorStateResponse>(
                methods::IPC_READ_SUBNET_ACTOR_STATE,
                params,
            )
            .await?;
        Ok(r)
    }
}

impl LotusJsonRPCClient<JsonRpcClientImpl> {
    /// A constructor that returns a `LotusJsonRPCClient` from a `Subnet`. The returned
    /// `LotusJsonRPCClient` makes requests to the URL defined in the `Subnet`.
    #[allow(dead_code)]
    pub(crate) fn from_subnet(subnet: &Subnet) -> Self {
        let url = subnet.jsonrpc_api_http.clone();
        let auth_token = subnet.auth_token.as_deref();
        let jsonrpc_client = JsonRpcClientImpl::new(url, auth_token);
        LotusJsonRPCClient::new(jsonrpc_client)
    }
}
