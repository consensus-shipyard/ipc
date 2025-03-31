// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHeight, IPCParentFinality};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use crate::finality::ParentViewPayload;

pub struct TopdownVoter {}

impl TopdownVoter {
    /// Obtain the next topdown parent height to vote
    pub async fn latest_finalized_checkpoint(&self) -> anyhow::Result<IPCParentFinality> {
        todo!()
    }

    pub async fn vote(
        &self,
        parent_view_payload: ParentViewPayload,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
