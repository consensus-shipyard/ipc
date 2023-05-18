// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod bottomup;
pub mod topdown;

use crate::config::Subnet;
use crate::lotus::LotusClient;
use anyhow::Result;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Display;

/// Checkpoint manager that handles a specific parent - child - checkpoint type tuple.
/// For example, we might have `/root` subnet and `/root/t01` as child, one implementation of manager
/// is handling the top-down checkpoint submission for `/root` to `/root/t01`.
#[async_trait]
pub trait CheckpointManager: Display {
    type LotusClient: LotusClient;

    /// The client of the parent
    fn parent_client(&self) -> &Self::LotusClient;

    /// Getter for the parent subnet this checkpoint manager is handling
    fn parent_subnet(&self) -> &Subnet;

    /// Getter for the target subnet this checkpoint manager is handling
    fn child_subnet(&self) -> &Subnet;

    /// The checkpoint period that the current manager is submitting upon
    fn checkpoint_period(&self) -> ChainEpoch;

    /// Obtain the last executed epoch of the checkpoint submission
    async fn last_executed_epoch(&self) -> Result<ChainEpoch>;

    /// The current epoch in the subnet that the checkpoints should be submitted to
    async fn current_epoch(&self) -> Result<ChainEpoch>;

    /// Submit the checkpoint based on the current epoch to submit and the previous epoch that was
    /// already submitted.
    async fn submit_checkpoint(&self, epoch: ChainEpoch, validator: &Address) -> Result<()>;

    /// Checks if the validator has already submitted in the epoch
    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool>;

    /// Performs checks to see if the subnet is ready for checkpoint submission. If `true` means the
    /// subnet is ready for submission, else means the subnet is not ready.
    async fn presubmission_check(&self) -> anyhow::Result<bool>;
}

/// Returns the first cid in the chain head
pub(crate) async fn chain_head_cid(client: &(impl LotusClient + Sync)) -> anyhow::Result<Cid> {
    let child_head = client.chain_head().await?;
    let cid_map = child_head.cids.first().unwrap();
    Cid::try_from(cid_map)
}
