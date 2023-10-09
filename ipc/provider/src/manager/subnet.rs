// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::staking::StakingChangeRequest;
use ipc_sdk::subnet::ConstructParams;
use ipc_sdk::subnet_id::SubnetID;

use crate::lotus::message::ipc::SubnetInfo;

/// Trait to interact with a subnet and handle its lifecycle.
#[async_trait]
pub trait SubnetManager: Send + Sync + TopDownCheckpointQuery {
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
        metadata: Vec<u8>,
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

    /// Propagate a cross-net message forward. For `postbox_msg_key`, we are using bytes because different
    /// runtime have different representations. For FVM, it should be `CID` as bytes. For EVM, it is
    /// `bytes32`.
    async fn propagate(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_key: Vec<u8>,
    ) -> Result<()>;

    async fn send_cross_message(
        &self,
        gateway_addr: Address,
        from: Address,
        cross_msg: CrossMsg,
    ) -> Result<()>;

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()>;

    /// Get the balance of an address
    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount>;

    /// Get chainID for the network.
    /// Returning as a `String` because the maximum value for an EVM
    /// networks is a `U256` that wouldn't fit in an integer type.
    async fn get_chain_id(&self) -> Result<String>;
}

/// Trait to interact with a subnet to query the necessary information for top down checkpoint.
#[async_trait]
pub trait TopDownCheckpointQuery: Send + Sync {
    /// Returns the genesis epoch that the subnet is created in parent network
    async fn genesis_epoch(&self, subnet_id: &SubnetID) -> Result<ChainEpoch>;
    /// Returns the chain head height
    async fn chain_head_height(&self) -> Result<ChainEpoch>;
    /// Returns the list of top down messages
    async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        start_epoch: ChainEpoch,
        end_epoch: ChainEpoch,
    ) -> Result<Vec<CrossMsg>>;
    /// Get the block hash
    async fn get_block_hash(&self, height: ChainEpoch) -> Result<Vec<u8>>;
    /// Get the validator change set from start to end block.
    async fn get_validator_changeset(
        &self,
        subnet_id: &SubnetID,
        start: ChainEpoch,
        end: ChainEpoch,
    ) -> Result<Vec<StakingChangeRequest>>;
}
