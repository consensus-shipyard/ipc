// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod binary;

use anyhow::{Context, Result};
use binary::init_comet_binary;
use std::ffi::OsStr;
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// Simple CometBFT daemon with restart logic
pub struct CometDaemon {
    args: Vec<String>,
    max_restarts: u32,
}

impl CometDaemon {
    /// Create a new daemon with the given arguments
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            max_restarts: 3,
        }
    }

    /// Set maximum restart attempts
    pub fn with_max_restarts(mut self, max_restarts: u32) -> Self {
        self.max_restarts = max_restarts;
        self
    }

    /// Run the daemon with restart logic
    pub async fn run_with_restart(&self, shutdown: CancellationToken) -> Result<()> {
        let binary_path = init_comet_binary().context("failed to initialize CometBFT binary")?;

        let mut attempts = 0;

        while attempts < self.max_restarts {
            if shutdown.is_cancelled() {
                info!("Shutdown requested");
                break;
            }

            let mut child = Command::new(&binary_path)
                .args(&self.args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .context("failed to spawn CometBFT process")?;

            info!("Started CometBFT daemon");

            let exit_status = tokio::select! {
                status = child.wait() => status.context("waiting for CometBFT process")?,
                _ = shutdown.cancelled() => {
                    info!("Shutdown signal received");
                    child.start_kill().ok();
                    let _ = child.wait().await;
                    break;
                }
            };

            if exit_status.success() {
                info!("CometBFT exited cleanly");
                break;
            } else {
                attempts += 1;
                error!(
                    "CometBFT crashed, attempt {}/{}",
                    attempts, self.max_restarts
                );

                if attempts < self.max_restarts {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }

        if attempts >= self.max_restarts {
            return Err(anyhow::anyhow!(
                "CometBFT failed after {} attempts",
                attempts
            ));
        }

        Ok(())
    }
}

/// Run a CometBFT command once
pub async fn run_comet<I, S>(args: I) -> Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary_path = init_comet_binary().context("failed to initialize CometBFT binary")?;

    let output = Command::new(&binary_path)
        .args(args)
        .output()
        .await
        .context("failed to run CometBFT command")?;

    if !output.stdout.is_empty() {
        info!("CometBFT: {}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() && !output.status.success() {
        error!(
            "CometBFT error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.status)
}
