// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::comet_bft::CometBftService;
use crate::services::eth_api::EthApiService;
use crate::services::fendermint::FendermintService;
use crate::services::Service;
use anyhow::{Context, Result};
use fendermint_app_settings::Settings;
use ipc_observability::traces::create_temporary_subscriber;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, subscriber, warn};

/// Configuration for service restart behavior
#[derive(Debug, Clone)]
struct RestartConfig {
    max_restarts: u32,
    initial_delay: Duration,
    max_delay: Duration,
}

impl Default for RestartConfig {
    fn default() -> Self {
        Self {
            max_restarts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Configuration for NodeManager startup behavior
#[derive(Debug, Clone)]
pub struct NodeManagerConfig {
    /// Maximum time to wait for CometBFT RPC to become ready
    pub max_startup_wait: Duration,
    /// Initial delay between CometBFT readiness checks
    pub initial_poll_interval: Duration,
    /// Maximum delay between CometBFT readiness checks
    pub max_poll_interval: Duration,
    /// Backoff multiplier for retry intervals
    pub backoff_multiplier: f64,
    /// Timeout for individual HTTP requests to CometBFT
    pub http_timeout: Duration,
}

impl Default for NodeManagerConfig {
    fn default() -> Self {
        Self {
            max_startup_wait: Duration::from_secs(60),
            initial_poll_interval: Duration::from_millis(100),
            max_poll_interval: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            http_timeout: Duration::from_secs(2),
        }
    }
}

pub struct NodeManager {
    home: PathBuf,
    settings: Settings,
    config: NodeManagerConfig,
}

impl NodeManager {
    pub fn new(home: PathBuf, settings: Settings) -> Self {
        Self {
            home,
            settings,
            config: NodeManagerConfig::default(),
        }
    }

    pub fn with_config(mut self, config: NodeManagerConfig) -> Self {
        self.config = config;
        self
    }

    /// Start all node services and run them until shutdown
    pub async fn start_all_services(&self) -> Result<()> {
        info!(
            target: "service.node_manager",
            "Starting IPC node services from home directory: {}",
            self.home.display()
        );

        // Initialize temporary tracing for this service startup
        let _tracing_guard = init_temporary_tracing();

        // Create a shared cancellation token for all services
        let cancel_token = CancellationToken::new();

        // Set up graceful shutdown handling
        let shutdown_handle = setup_graceful_shutdown(cancel_token.clone());

        // Start all services in order
        let fendermint_handle = start_fendermint(&self.settings, &cancel_token).await?;
        let cometbft_handle = start_cometbft(self.home.as_path(), &cancel_token).await?;

        // Wait for CometBFT to be ready before starting ETH API
        await_cometbft_ready(&self.settings, &self.config, &cancel_token).await?;

        let eth_api_handle = start_eth_api(&self.settings, &cancel_token).await?;

        // Wait for services to complete or be cancelled
        let service_handles = vec![fendermint_handle, cometbft_handle, eth_api_handle];
        await_services_shutdown(service_handles, shutdown_handle).await?;

        info!(target: "service.node_manager", "All node services stopped cleanly");
        Ok(())
    }
}

/// Initialize temporary tracing for service coordination
///
/// Returns a guard that must be kept alive for tracing to work.
/// This prevents each service from creating redundant temporary subscribers.
fn init_temporary_tracing() -> impl Drop {
    struct TracingGuard {
        _subscriber: subscriber::DefaultGuard,
    }

    impl Drop for TracingGuard {
        fn drop(&mut self) {
            tracing::debug!(target: "service.node_manager", "Temporary tracing guard dropped");
        }
    }

    let temp_subscriber = create_temporary_subscriber();
    let guard = subscriber::set_default(temp_subscriber);

    info!(target: "service.node_manager", "Temporary tracing initialized for service coordination");

    TracingGuard { _subscriber: guard }
}

/// Generic service restart function
///
/// Runs a service with automatic restart on failure, using exponential backoff.
/// Respects cancellation tokens for clean shutdown.
async fn run_service_with_restart<S, F>(
    service_name: &str,
    service_factory: F,
    cancel_token: CancellationToken,
    config: RestartConfig,
) -> Result<()>
where
    S: Service,
    F: Fn() -> S + Send + 'static,
{
    let mut attempts = 0u32;
    let mut delay = config.initial_delay;

    loop {
        if cancel_token.is_cancelled() {
            return Ok(());
        }

        let service = service_factory();

        match service.run(cancel_token.clone()).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                attempts += 1;
                if attempts > config.max_restarts {
                    return Err(e)
                        .context(format!("{} service exceeded max restarts", service_name));
                }

                warn!(
                    target: "service.node_manager",
                    "{} service failed (attempt {}/{}): {} – restarting in {}s",
                    service_name,
                    attempts,
                    config.max_restarts,
                    e,
                    delay.as_secs()
                );

                // Wait, but abort early if we are shutting down.
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {},
                    _ = cancel_token.cancelled() => return Ok(()),
                }

                // Exponential back-off capped at max_delay.
                delay = std::cmp::min(delay * 2, config.max_delay);
            }
        }
    }
}

/// Set up graceful shutdown signal handling
///
/// Listens for SIGINT/SIGTERM and triggers cancellation when received.
/// Returns a handle to the shutdown task.
fn setup_graceful_shutdown(cancel_token: CancellationToken) -> JoinHandle<()> {
    let token = cancel_token.clone();
    tokio::spawn(async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C signal handler");
        };

        #[cfg(unix)]
        let sigterm = async {
            use tokio::signal::unix::{signal, SignalKind};
            signal(SignalKind::terminate())
                .expect("failed to install SIGTERM signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let sigterm = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!(target: "service.node_manager", "Received Ctrl+C, initiating graceful shutdown");
            }
            _ = sigterm => {
                info!(target: "service.node_manager", "Received SIGTERM, initiating graceful shutdown");
            }
        }

        token.cancel();
        info!(target: "service.node_manager", "Shutdown signal sent to all services");
    })
}

/// Start the Fendermint service
///
/// Fendermint provides the ABCI server that CometBFT connects to.
async fn start_fendermint(
    settings: &Settings,
    cancel_token: &CancellationToken,
) -> Result<JoinHandle<Result<()>>> {
    info!(target: "service.node_manager", "Starting Fendermint ABCI service");

    let settings = settings.clone();
    let token = cancel_token.clone();
    let config = RestartConfig::default();

    let handle = tokio::spawn(async move {
        run_service_with_restart(
            "Fendermint",
            move || FendermintService::new(settings.clone()),
            token,
            config,
        )
        .await
    });

    Ok(handle)
}

/// Start the CometBFT consensus service
///
/// CometBFT connects to Fendermint's ABCI server for consensus.
async fn start_cometbft(
    home: &Path,
    cancel_token: &CancellationToken,
) -> Result<JoinHandle<Result<()>>> {
    info!(target: "service.node_manager", "Starting CometBFT consensus service");

    let cometbft_service = CometBftService::new(home.join("cometbft"));
    let token = cancel_token.clone();

    let handle = tokio::spawn(async move {
        cometbft_service
            .run(token)
            .await
            .context("CometBFT service encountered an error")
    });

    info!(target: "service.node_manager", "CometBFT service started");
    Ok(handle)
}

/// Wait for CometBFT RPC to become ready
///
/// Uses exponential backoff to avoid overwhelming the RPC endpoint during startup.
/// Respects cancellation to enable clean shutdown during startup.
async fn await_cometbft_ready(
    settings: &Settings,
    config: &NodeManagerConfig,
    cancel_token: &CancellationToken,
) -> Result<()> {
    info!(target: "service.node_manager", "Waiting for CometBFT RPC to be ready");

    let start_time = Instant::now();
    let mut poll_interval = config.initial_poll_interval;
    let rpc_url = settings
        .tendermint_rpc_url()
        .context("failed to get CometBFT RPC URL from settings")?;

    loop {
        // Check for cancellation
        if cancel_token.is_cancelled() {
            info!(target: "service.node_manager", "CometBFT readiness check cancelled");
            return Ok(());
        }

        // Check if we've exceeded maximum wait time
        if start_time.elapsed() > config.max_startup_wait {
            return Err(anyhow::anyhow!(
                "CometBFT RPC did not become ready within {} seconds at {}",
                config.max_startup_wait.as_secs(),
                rpc_url
            ));
        }

        // Try to connect to CometBFT RPC
        match check_comet_bft_status(&rpc_url.to_string(), config, cancel_token).await {
            Ok(()) => {
                info!(
                    target: "service.node_manager",
                    "CometBFT RPC is ready at {} (took {:.2}s)",
                    rpc_url,
                    start_time.elapsed().as_secs_f64()
                );
                return Ok(());
            }
            Err(e) => {
                info!(
                    target: "service.node_manager",
                    "CometBFT RPC not ready yet at {}: {} (retrying in {:.1}s)",
                    rpc_url,
                    e,
                    poll_interval.as_secs_f64()
                );
            }
        }

        // Wait with cancellation support
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {}
            _ = cancel_token.cancelled() => {
                info!(target: "service.node_manager", "CometBFT readiness check cancelled during sleep");
                return Ok(());
            }
        }

        // Apply exponential backoff
        poll_interval = std::cmp::min(
            Duration::from_secs_f64(poll_interval.as_secs_f64() * config.backoff_multiplier),
            config.max_poll_interval,
        );
    }
}

/// Check if CometBFT RPC is responding
///
/// Supports cancellation to enable clean shutdown during health checks.
async fn check_comet_bft_status(
    rpc_url: &str,
    config: &NodeManagerConfig,
    cancel_token: &CancellationToken,
) -> Result<()> {
    use reqwest::Client;

    let client = Client::new();
    let status_url = format!("{}/status", rpc_url);

    // Create the request with timeout
    let request_future = client.get(&status_url).timeout(config.http_timeout).send();

    // Race the request against cancellation
    let response = tokio::select! {
        result = request_future => {
            result.with_context(|| format!("failed to connect to CometBFT RPC at {}", status_url))?
        }
        _ = cancel_token.cancelled() => {
            return Err(anyhow::anyhow!("CometBFT status check cancelled"));
        }
    };

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "CometBFT RPC returned error status: {} at {}",
            response.status(),
            status_url
        ))
    }
}

/// Start the ETH API service
///
/// ETH API provides Ethereum JSON-RPC compatibility over CometBFT.
async fn start_eth_api(
    settings: &Settings,
    cancel_token: &CancellationToken,
) -> Result<JoinHandle<Result<()>>> {
    info!(target: "service.node_manager", "Starting ETH API service");

    let settings = settings.clone();
    let token = cancel_token.clone();
    let config = RestartConfig::default();

    let handle = tokio::spawn(async move {
        run_service_with_restart(
            "ETH API",
            move || EthApiService::new(settings.clone(), Duration::from_secs(5)),
            token,
            config,
        )
        .await
    });

    Ok(handle)
}

/// Wait for all services to complete or shutdown
///
/// Handles JoinErrors explicitly, distinguishing between panics and cancellation.
/// Ensures clean shutdown coordination across all services.
async fn await_services_shutdown(
    service_handles: Vec<JoinHandle<Result<()>>>,
    shutdown_handle: JoinHandle<()>,
) -> Result<()> {
    info!(target: "service.node_manager", "Waiting for all services to complete");

    // Wait for either all services to complete or shutdown signal
    let services_future = async {
        let mut results = Vec::new();
        for handle in service_handles {
            results.push(handle.await);
        }
        results
    };

    let service_results = tokio::select! {
        results = services_future => results,
        _ = shutdown_handle => {
            info!(target: "service.node_manager", "Shutdown completed via signal handler");
            return Ok(());
        }
    };

    // Handle service results with explicit JoinError inspection
    let mut had_errors = false;
    for (index, result) in service_results.into_iter().enumerate() {
        let service_name = match index {
            0 => "Fendermint",
            1 => "CometBFT",
            2 => "ETH API",
            _ => "Unknown",
        };

        match result {
            Ok(Ok(())) => {
                info!(target: "service.node_manager", "{} service completed successfully", service_name);
            }
            Ok(Err(service_error)) => {
                error!(
                    target: "service.node_manager",
                    "{} service failed with error: {}",
                    service_name,
                    service_error
                );
                had_errors = true;
            }
            Err(join_error) => {
                if join_error.is_panic() {
                    error!(
                        target: "service.node_manager",
                        "{} service panicked: {}",
                        service_name,
                        join_error
                    );
                    had_errors = true;
                } else if join_error.is_cancelled() {
                    info!(
                        target: "service.node_manager",
                        "{} service was cancelled during shutdown",
                        service_name
                    );
                } else {
                    warn!(
                        target: "service.node_manager",
                        "{} service join failed: {}",
                        service_name,
                        join_error
                    );
                    had_errors = true;
                }
            }
        }
    }

    if had_errors {
        Err(anyhow::anyhow!(
            "One or more services encountered errors during execution"
        ))
    } else {
        Ok(())
    }
}
