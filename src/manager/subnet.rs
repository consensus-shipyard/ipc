// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
///! IPC node-specific traits.
use std::collections::HashMap;

use crate::checkpoint::NativeBottomUpCheckpoint;
use anyhow::Result;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::ConstructParams;

use crate::lotus::message::ipc::QueryValidatorSetResponse;
use crate::lotus::message::ipc::SubnetInfo;

/// Trait to interact with a subnet and handle its lifecycle.
#[async_trait]
pub trait SubnetManager: Send + Sync {
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
        validator_net_addr: String,
        worker_addr: Address,
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

    /// Fund injects new funds from an account of the parent chain to a subnet.
    /// Returns the epoch that the fund is executed in the parent.
    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch>;

    /// Release creates a new check message to release funds in parent chain
    /// Returns the epoch that the released is executed in the child.
    async fn release(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch>;

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

    /// Get the balance of an address
    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount>;

    /// Returns the epoch of the latest top-down checkpoint executed
    async fn last_topdown_executed(&self, gateway_addr: &Address) -> Result<ChainEpoch>;

    /// Returns the list of checkpoints from a subnet actor for the given epoch range.
    async fn list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> Result<Vec<NativeBottomUpCheckpoint>>;

    /// Returns the validator set
    async fn get_validator_set(
        &self,
        subnet_id: &SubnetID,
        gateway: Option<Address>,
    ) -> Result<QueryValidatorSetResponse>;
}
