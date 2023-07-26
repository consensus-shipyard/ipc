// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

// #[deprecated]
// mod conversion;
mod convert;
mod manager;

use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::{CrossMsg, TopDownCheckpoint};
use ipc_sdk::subnet_id::SubnetID;

use super::subnet::SubnetManager;
pub use manager::EthSubnetManager;

use crate::checkpoint::NativeBottomUpCheckpoint;
use crate::manager::evm::manager::subnet_actor_manager_facet;
pub use convert::{eth_to_fil_amount, ethers_address_to_fil_address, fil_to_eth_amount};

#[async_trait]
pub trait EthManager: SubnetManager {
    /// Fetches the last executed epoch for voting in the gateway.
    async fn gateway_last_voting_executed_epoch(&self) -> anyhow::Result<ChainEpoch>;

    /// Fetches the last executed epoch for voting in the subnet actor.
    async fn subnet_last_voting_executed_epoch(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch>;

    /// The current epoch/block number of the blockchain that the manager connects to.
    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch>;

    /// Submit top down checkpoint the gateway.
    async fn submit_top_down_checkpoint(
        &self,
        from: &Address,
        checkpoint: TopDownCheckpoint,
    ) -> anyhow::Result<ChainEpoch>;

    /// Submit bottom up checkpoint to the subnet actor.
    async fn submit_bottom_up_checkpoint(
        &self,
        from: &Address,
        checkpoint: NativeBottomUpCheckpoint,
    ) -> anyhow::Result<ChainEpoch>;

    /// Has the validator voted in subnet contract at epoch
    async fn has_voted_in_subnet(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<bool>;

    /// Has the validator voted in the gateway for an epoch
    async fn has_voted_in_gateway(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<bool>;

    /// Get all the top down messages till a certain epoch
    async fn bottom_up_checkpoint(
        &self,
        epoch: ChainEpoch,
    ) -> anyhow::Result<subnet_actor_manager_facet::BottomUpCheckpoint>;

    /// Get the latest applied top down nonce
    async fn get_applied_top_down_nonce(&self, subnet_id: &SubnetID) -> anyhow::Result<u64>;

    /// Get the bottom up checkpoint a certain epoch
    async fn top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        nonce: u64,
    ) -> anyhow::Result<Vec<CrossMsg>>;

    /// Get the list of validators in a subnet
    async fn validators(&self, subnet_id: &SubnetID) -> anyhow::Result<Vec<Address>>;

    /// Checks if the gateway is initialized
    async fn gateway_initialized(&self) -> anyhow::Result<bool>;

    /// Get the subnet contract bottom up checkpoint period
    async fn subnet_bottom_up_checkpoint_period(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch>;

    /// Get the gateway contract top down checkpoint period
    async fn gateway_top_down_check_period(&self) -> anyhow::Result<ChainEpoch>;

    /// Get the previous checkpoint hash from the gateway
    async fn prev_bottom_up_checkpoint_hash(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> anyhow::Result<[u8; 32]>;

    /// The minimal number of validators required for the subnet
    async fn min_validators(&self, subnet_id: &SubnetID) -> anyhow::Result<u64>;
}
