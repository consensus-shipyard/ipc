// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::{
    env, fs, io,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

static COMET_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/comet"));

/// Writes the embedded `COMET_BIN` binary to a temp file, ensures it's executable,
/// and returns its path. Subsequent calls do nothing if already initialized.
fn init_comet() -> io::Result<PathBuf> {
    let file_name = if cfg!(windows) {
        "cometbft.exe"
    } else {
        "cometbft"
    };

    let mut tmp = env::temp_dir();
    tmp.push(file_name);

    if tmp.exists() {
        return Ok(tmp);
    }

    fs::write(&tmp, COMET_BIN)?;

    // On Unix, set the exec bit; no-op on Windows
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp, perms)?;
    }

    Ok(tmp)
}

/// Spawns the embedded Comet BFT binary with the provided arguments.
/// Automatically handles writing and permission setup.
pub fn run_comet<I, S>(args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let path = init_comet()?;
    Command::new(path).args(args).status()
}

/// CometBFT daemon for long-running processes with restart logic and logging
pub struct CometDaemon {
    args: Vec<String>,
    max_restarts: u32,
    restart_delay: Duration,
}

impl CometDaemon {
    /// Create a new CometBFT daemon with the given arguments
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            max_restarts: 5,
            restart_delay: Duration::from_secs(3),
        }
    }

    /// Set the maximum number of restart attempts
    pub fn with_max_restarts(mut self, max_restarts: u32) -> Self {
        self.max_restarts = max_restarts;
        self
    }

    /// Set the delay between restart attempts
    pub fn with_restart_delay(mut self, delay: Duration) -> Self {
        self.restart_delay = delay;
        self
    }

    /// Run the CometBFT daemon with restart logic and graceful shutdown
    pub async fn run_with_restart(&self, shutdown: CancellationToken) -> Result<()> {
        let mut attempts = 0;

        loop {
            if shutdown.is_cancelled() {
                info!(target: "cometbft", "Shutdown signal received");
                break;
            }

            match self.spawn_daemon().await {
                Ok(mut child) => {
                    info!(target: "cometbft", "CometBFT daemon started");

                    // Wait for either process exit or shutdown signal
                    let exit_status = tokio::select! {
                        status = child.wait() => status,
                        _ = shutdown.cancelled() => {
                            self.kill_process(&mut child).await;
                            break;
                        }
                    };

                    match exit_status {
                        Ok(status) if status.success() => {
                            info!(target: "cometbft", "CometBFT exited cleanly");
                            break;
                        }
                        Ok(status) => {
                            attempts += 1;
                            error!(
                                target: "cometbft",
                                "CometBFT crashed with code {}. Restart attempt {}/{}",
                                status.code().unwrap_or(-1),
                                attempts,
                                self.max_restarts
                            );
                        }
                        Err(e) => {
                            attempts += 1;
                            error!(
                                target: "cometbft",
                                "Error waiting for CometBFT: {}. Restart attempt {}/{}",
                                e,
                                attempts,
                                self.max_restarts
                            );
                        }
                    }
                }
                Err(e) => {
                    attempts += 1;
                    error!(
                        target: "cometbft",
                        "Failed to spawn CometBFT: {}. Restart attempt {}/{}",
                        e,
                        attempts,
                        self.max_restarts
                    );
                }
            }

            if attempts >= self.max_restarts {
                error!(target: "cometbft", "Max restarts reached, giving up");
                return Err(anyhow::anyhow!(
                    "CometBFT failed after {} restart attempts",
                    attempts
                ));
            }

            info!(
                target: "cometbft",
                "Waiting {} seconds before restart attempt {}",
                self.restart_delay.as_secs(),
                attempts + 1
            );
            tokio::time::sleep(self.restart_delay).await;
        }

        Ok(())
    }

    /// Spawn the CometBFT daemon process with logging setup
    async fn spawn_daemon(&self) -> Result<Child> {
        let path = init_comet().context("failed to initialize CometBFT binary")?;
        let mut child = tokio::process::Command::new(path)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn CometBFT daemon")?;

        // Setup logging with service prefix
        self.setup_logging(&mut child)?;

        Ok(child)
    }

    /// Setup stdout and stderr logging with service prefix
    fn setup_logging(&self, child: &mut Child) -> Result<()> {
        // Setup stdout logging
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    info!(target: "cometbft", "stdout: {}", line);
                }
            });
        }

        // Setup stderr logging
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    error!(target: "cometbft", "stderr: {}", line);
                }
            });
        }

        Ok(())
    }

    /// Gracefully kill the CometBFT process
    async fn kill_process(&self, child: &mut Child) {
        if let Some(pid) = child.id() {
            let _ = child.kill().await;
            info!(target: "cometbft", "CometBFT process (pid={}) killed", pid);
        }
    }
}
