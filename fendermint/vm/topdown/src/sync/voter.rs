// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use crate::{BlockHeight, IPCParentFinality};

pub struct TopdownVoter {}

impl TopdownVoter {
    pub fn livesness_period(&self) -> anyhow::Result<BlockHeight> {
        todo!()
    }

    /// Obtain the next topdown parent height to vote
    pub async fn latest_finalized_checkpoint(&self) -> anyhow::Result<IPCParentFinality> {
        todo!()
    }

    pub async fn vote(
        &self,
        _parent_block_height: BlockHeight,
        _xnet_msgs: Vec<IpcEnvelope>,
        _power_changes: Vec<PowerChangeRequest>
    ) -> anyhow::Result<()> {
        todo!()
    }
}