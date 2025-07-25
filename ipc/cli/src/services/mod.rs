// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::Result;
use async_trait::async_trait;

use tokio_util::sync::CancellationToken;

pub mod comet_bft;
pub mod eth_api;
pub mod fendermint;
pub mod node_manager;

#[async_trait]
pub trait Service: Send + Sync {
    /// Returns a static name for the service.
    fn name(&self) -> &'static str;

    /// Runs the service until the provided shutdown token is triggered.
    async fn run(&self, shutdown: CancellationToken) -> Result<()>;
}
