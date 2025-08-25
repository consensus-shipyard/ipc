// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod binary;

use anyhow::{Context, Result};
use binary::init_comet_binary;
use ipc_observability::traces::CONSENSUS_TARGET;
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

            // Capture stdout and stderr streams
            let stdout = child.stdout.take().expect("stdout should be available");
            let stderr = child.stderr.take().expect("stderr should be available");

            // Spawn tasks to handle stdout and stderr streams
            let stdout_handle = tokio::spawn(process_stdout_stream(stdout));
            let stderr_handle = tokio::spawn(process_stderr_stream(stderr));

            let exit_status = tokio::select! {
                status = child.wait() => status.context("waiting for CometBFT process")?,
                _ = shutdown.cancelled() => {
                    info!("Shutdown signal received");
                    child.start_kill().ok();
                    let _ = child.wait().await;
                    break;
                }
            };

            // Wait for stream processing to complete
            let _ = tokio::join!(stdout_handle, stderr_handle);

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

/// Process CometBFT stdout stream
async fn process_stdout_stream(stream: tokio::process::ChildStdout) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if !line.trim().is_empty() {
            tracing::event!(
                target: CONSENSUS_TARGET,
                tracing::Level::INFO,
                "CometBFT: {}",
                line
            );
        }
    }
}

/// Process CometBFT stderr stream
async fn process_stderr_stream(stream: tokio::process::ChildStderr) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if !line.trim().is_empty() {
            // Determine log level based on content and emit appropriate event
            if line.contains("ERROR") || line.contains("error") {
                tracing::event!(
                    target: CONSENSUS_TARGET,
                    tracing::Level::ERROR,
                    "CometBFT: {}",
                    line
                );
            } else if line.contains("WARN") || line.contains("warn") {
                tracing::event!(
                    target: CONSENSUS_TARGET,
                    tracing::Level::WARN,
                    "CometBFT: {}",
                    line
                );
            } else if line.contains("DEBUG") || line.contains("debug") {
                tracing::event!(
                    target: CONSENSUS_TARGET,
                    tracing::Level::DEBUG,
                    "CometBFT: {}",
                    line
                );
            } else {
                tracing::event!(
                    target: CONSENSUS_TARGET,
                    tracing::Level::INFO,
                    "CometBFT: {}",
                    line
                );
            }
        }
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
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        for line in stdout_str.lines() {
            if !line.trim().is_empty() {
                tracing::event!(
                    target: CONSENSUS_TARGET,
                    tracing::Level::INFO,
                    "CometBFT: {}",
                    line
                );
            }
        }
    }
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        for line in stderr_str.lines() {
            if !line.trim().is_empty() {
                // Determine log level based on content and emit appropriate event
                if line.contains("ERROR") || line.contains("error") {
                    tracing::event!(
                        target: CONSENSUS_TARGET,
                        tracing::Level::ERROR,
                        "CometBFT: {}",
                        line
                    );
                } else if line.contains("WARN") || line.contains("warn") {
                    tracing::event!(
                        target: CONSENSUS_TARGET,
                        tracing::Level::WARN,
                        "CometBFT: {}",
                        line
                    );
                } else if line.contains("DEBUG") || line.contains("debug") {
                    tracing::event!(
                        target: CONSENSUS_TARGET,
                        tracing::Level::DEBUG,
                        "CometBFT: {}",
                        line
                    );
                } else {
                    tracing::event!(
                        target: CONSENSUS_TARGET,
                        tracing::Level::INFO,
                        "CometBFT: {}",
                        line
                    );
                }
            }
        }
    }

    Ok(output.status)
}
