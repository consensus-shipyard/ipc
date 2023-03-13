// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
///! The lotus api to interact with lotus node
use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use serde::de::DeserializeOwned;

use message::chain::ChainHeadResponse;
use message::mpool::{MpoolPushMessage, MpoolPushMessageResponseInner};
use message::state::{ReadStateResponse, StateWaitMsgResponse};
use message::wallet::{WalletKeyType, WalletListResponse};

use crate::lotus::message::ipc::{
    IPCGetPrevCheckpointForChildResponse, IPCReadGatewayStateResponse,
    IPCReadSubnetActorStateResponse,
};

pub mod client;
pub mod message;
#[cfg(test)]
mod tests;

/// The network version of lotus network.
/// see https://github.com/filecoin-project/go-state-types/blob/f6fd668a32b4b4a0bc39fd69d8a5f8fb11f49461/network/version.go#L7
pub type NetworkVersion = u32;

/// The Lotus client api to interact with the Lotus node.
#[async_trait]
pub trait LotusClient {
    /// Push the message to memory pool, see: https://lotus.filecoin.io/reference/lotus/mpool/#mpoolpushmessage
    async fn mpool_push_message(
        &self,
        msg: MpoolPushMessage,
    ) -> Result<MpoolPushMessageResponseInner>;

    /// Wait for the message cid of a particular nonce, see: https://lotus.filecoin.io/reference/lotus/state/#statewaitmsg
    async fn state_wait_msg(&self, cid: Cid) -> Result<StateWaitMsgResponse>;

    /// Returns the name of the network the node is synced to, see https://lotus.filecoin.io/reference/lotus/state/#statenetworkname
    async fn state_network_name(&self) -> Result<String>;

    /// Returns the network version at the given tipset, see https://lotus.filecoin.io/reference/lotus/state/#statenetworkversion
    async fn state_network_version(&self, tip_sets: Vec<Cid>) -> Result<NetworkVersion>;

    /// Returns the CID of the builtin actors manifest for the given network version, see https://github.com/filecoin-project/lotus/blob/master/documentation/en/api-v1-unstable-methods.md#stateactormanifestcid
    async fn state_actor_code_cids(
        &self,
        network_version: NetworkVersion,
    ) -> Result<HashMap<String, Cid>>;

    /// Get the default wallet of the node, see: https://lotus.filecoin.io/reference/lotus/wallet/#walletdefaultaddress
    async fn wallet_default(&self) -> Result<Address>;

    /// List the wallets in the node, see: https://lotus.filecoin.io/reference/lotus/wallet/#walletlist
    async fn wallet_list(&self) -> Result<WalletListResponse>;

    /// Create a new wallet, see: https://lotus.filecoin.io/reference/lotus/wallet/#walletnew
    async fn wallet_new(&self, key_type: WalletKeyType) -> Result<String>;

    /// Read the state of the address at tipset, see: https://lotus.filecoin.io/reference/lotus/state/#statereadstate
    async fn read_state<State: DeserializeOwned + Debug>(
        &self,
        address: Address,
        tipset: Cid,
    ) -> Result<ReadStateResponse<State>>;

    /// Returns the current head of the chain.
    /// See: https://lotus.filecoin.io/reference/lotus/chain/#chainhead
    async fn chain_head(&self) -> Result<ChainHeadResponse>;

    async fn ipc_get_prev_checkpoint_for_child(
        &self,
        child_subnet_id: SubnetID,
    ) -> Result<IPCGetPrevCheckpointForChildResponse>;

    /// Returns the checkpoint template at `epoch`.
    async fn ipc_get_checkpoint_template(&self, epoch: ChainEpoch) -> Result<Checkpoint>;

    /// Returns the state of the gateway actor at `tip_set`.
    async fn ipc_read_gateway_state(&self, tip_set: Cid) -> Result<IPCReadGatewayStateResponse>;

    /// Returns the state of the subnet actor at `tip_set`.
    async fn ipc_read_subnet_actor_state(
        &self,
        subnet_id: &SubnetID,
        tip_set: Cid,
    ) -> Result<IPCReadSubnetActorStateResponse>;
}
