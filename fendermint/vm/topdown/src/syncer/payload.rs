// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHash, BlockHeight};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::StakingChangeRequest;

#[derive(Clone, Debug)]
pub struct ParentBlockViewPayload {
    pub parent_hash: BlockHash,
    /// Encodes cross-net messages.
    pub xnet_msgs: Vec<IpcEnvelope>,
    /// Encodes validator membership change commands.
    pub validator_changes: Vec<StakingChangeRequest>,
}

#[derive(Clone, Debug)]
pub struct ParentBlockView {
    pub parent_height: BlockHeight,
    /// If the payload is None, this means the parent height is a null block
    pub payload: Option<ParentBlockViewPayload>,
}

impl ParentBlockView {
    pub fn null_block(h: BlockHeight) -> Self {
        Self {
            parent_height: h,
            payload: None,
        }
    }

    pub fn nonnull_block(
        h: BlockHeight,
        parent_hash: BlockHash,
        xnet_msgs: Vec<IpcEnvelope>,
        validator_changes: Vec<StakingChangeRequest>,
    ) -> Self {
        Self {
            parent_height: h,
            payload: Some(ParentBlockViewPayload {
                parent_hash,
                xnet_msgs,
                validator_changes,
            }),
        }
    }
}
