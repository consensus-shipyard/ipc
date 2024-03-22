// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub type BlockHeight = u64;
/// Hex encoded block hash.
pub type BlockHashHex<'a> = &'a str;

#[derive(Debug, Default)]
pub struct NewParentView<'a> {
    pub is_null: bool,
    pub block_height: BlockHeight,
    pub block_hash: Option<BlockHashHex<'a>>, // hex encoded, unless null block
    pub num_msgs: usize,
    pub num_validator_changes: usize,
}

#[derive(Debug, Default)]
pub struct ParentFinalityCommitted<'a> {
    pub block_height: BlockHeight,
    pub block_hash: BlockHashHex<'a>,
}

#[derive(Debug, Default)]
pub struct NewBottomUpCheckpoint<'a> {
    pub block_height: BlockHeight,
    pub block_hash: BlockHashHex<'a>,
    pub num_msgs: usize,
    pub next_configuration_number: u64,
}
