// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
mod app;
pub mod ipc;
mod store;
mod tmconv;
mod upgrades;

pub use app::{App, AppConfig};
pub use store::{AppStore, BitswapBlockstore};

// Different type from `ChainEpoch` just because we might use epoch in a more traditional sense for checkpointing.
pub type BlockHeight = u64;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_VERSION: u64 = 0;
