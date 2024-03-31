// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub type BlockHeight = u64;

#[derive(Debug, Default)]
pub struct GetLatestAcceptedCheckpoint {
    pub block_height: BlockHeight,
}

#[derive(Debug, Default)]
pub struct SubmitBottomUpCheckpoint {
    pub block_height: BlockHeight,
    pub checkpoint_count: u64,
}

#[derive(Debug, Default)]
pub struct SubmitBottomUpCheckpointFail {
    pub block_height: BlockHeight,
    pub checkpoint_count: u64,
}
