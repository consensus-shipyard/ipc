// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

mod error_parsing;
pub mod gas_estimator_middleware;
mod manager;

use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use ipc_api::subnet_id::SubnetID;

use super::subnet::SubnetManager;
pub use manager::EthSubnetManager;

#[async_trait]
pub trait EthManager: SubnetManager {
    /// The current epoch/block number of the blockchain that the manager connects to.
    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch>;

    /// Get the latest applied top down nonce
    async fn get_applied_top_down_nonce(&self, subnet_id: &SubnetID) -> anyhow::Result<u64>;

    /// Get the subnet contract bottom up checkpoint period
    async fn subnet_bottom_up_checkpoint_period(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch>;

    /// The minimal number of validators required for the subnet
    async fn min_validators(&self, subnet_id: &SubnetID) -> anyhow::Result<u64>;
}
