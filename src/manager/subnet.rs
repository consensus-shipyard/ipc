// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
///! IPC node-specific traits.
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};

use crate::lotus::message::{ipc::SubnetInfo, wallet::WalletKeyType};

/// Trait to interact with a subnet and handle its lifecycle.
#[async_trait]
pub trait SubnetManager {
    /// Deploys a new subnet actor on the `parent` subnet and with the
    /// configuration passed in `ConstructParams`.
    /// The result of the function is the ID address for the subnet actor from which the final
    /// subet ID can be inferred.
    async fn create_subnet(&self, from: Address, params: ConstructParams) -> Result<Address>;

    /// Performs the call to join a subnet from a wallet address and staking an amount
    /// of collateral. This function, as well as all of the ones on this trait, can infer
    /// the specific subnet and actors on which to perform the relevant calls from the
    /// SubnetID given as an argument.
    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        params: JoinParams,
    ) -> Result<()>;

    /// Sends a request to leave a subnet from a wallet address.
    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Sends a signal to kill a subnet
    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Lists all the registered children in a gateway.
    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>>;

    /// Fund injects new funds from an account of the parent chain to a subnet
    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        amount: TokenAmount,
    ) -> Result<()>;

    /// Release creates a new check message to release funds in parent chain
    async fn release(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        amount: TokenAmount,
    ) -> Result<()>;

    /// Propagate a cross-net message forward
    async fn propagate(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_cid: Cid,
    ) -> Result<()>;

    /// Sets a new net address to an existing validator
    async fn set_validator_net_addr(
        &self,
        subnet: SubnetID,
        from: Address,
        validator_net_addr: String,
    ) -> Result<()>;

    /// Whitelist a series of addresses as propagator of a cross net message
    async fn whitelist_propagator(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        postbox_msg_cid: Cid,
        from: Address,
        to_add: Vec<Address>,
    ) -> Result<()>;

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()>;

    ///  Create new wallet in a subnet
    async fn wallet_new(&self, key_type: WalletKeyType) -> Result<Address>;

    /// Returns the epoch of the latest top-down checkpoint executed
    async fn last_topdown_executed(&self) -> Result<ChainEpoch>;

    /// Returns the list of checkpoints from a subnet actor for the given epoch range.
    async fn list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> Result<Vec<BottomUpCheckpoint>>;
}
