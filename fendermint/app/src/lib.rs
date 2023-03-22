// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
pub mod app;
pub mod store;
mod tmconv;

// Different type from `ChainEpoch` just because we might use epoch in a more traditional sense for checkpointing.
pub type BlockHeight = u64;
