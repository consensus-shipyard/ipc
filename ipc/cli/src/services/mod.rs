// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::Result;
use async_trait::async_trait;
use futures_util::future::{join_all, select_all};
use tokio::signal;
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

/// Run multiple services concurrently with graceful shutdown
pub async fn run_services(services: Vec<Box<dyn Service>>) -> Result<()> {
    let shutdown_token = CancellationToken::new();

    // Spawn a task to listen for SIGINT (Ctrl+C).
    let shutdown_token_clone = shutdown_token.clone();
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            tracing::warn!("SIGINT received, shutting down all services.");
            shutdown_token_clone.cancel();
        }
    });

    let tasks: Vec<_> = services
        .into_iter()
        .map(|service| {
            let token = shutdown_token.child_token();
            let name = service.name().to_string();
            tokio::spawn(async move {
                match service.run(token).await {
                    Ok(()) => {
                        tracing::info!("Service '{}' finished gracefully", name);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Service '{}' encountered an error: {:?}", name, e);
                        Err(e)
                    }
                }
            })
        })
        .collect();

    let (result, _index, remaining) = select_all(tasks).await;

    if let Err(e) = result {
        tracing::warn!("A service failed with error: {:?}. Triggering shutdown.", e);
    }
    shutdown_token.cancel();
    let _ = join_all(remaining).await;
    Ok(())
}
