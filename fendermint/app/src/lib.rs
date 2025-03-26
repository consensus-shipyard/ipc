extern crate core;

// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
mod app;
pub mod ipc;
pub mod keys;
pub mod metrics;
pub mod observe;
pub mod service;
mod store;
mod tmconv;
mod validators;
pub use fs_err as fs;

pub use app::{App, AppConfig};
pub use store::{AppStore, BitswapBlockstore};

// Different type from `ChainEpoch` just because we might use epoch in a more traditional sense for checkpointing.
pub type BlockHeight = u64;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum AppExitCode {
    /// Fendermint exited normally
    Ok = 0,
    /// Fendermint exited with an unknown error
    UnknownError = 1,
    /// Fendermint exited since it reached a block height equal to halt_height
    Halt = 2,
}
