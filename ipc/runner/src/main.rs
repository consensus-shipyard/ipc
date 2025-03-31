// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_app::service::node::run as run_node;
use fendermint_app_settings::Settings;
use ipc_observability::config::{
    default_log_level, FileLayerSettings, RotationKind, TracingSettings,
};
use ipc_observability::traces::{set_global_tracing_subscriber, CONSENSUS_TARGET};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio_util::sync::CancellationToken;

use std::env;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use tokio::{select, signal};
use tracing::{debug, error, info, trace, warn};

enum ExitReason {
    AbciEnded,
    CometExited(std::io::Result<std::process::ExitStatus>),
    Signal,
}

async fn wait_for_exit(
    comet_child: &mut Child,
    abci_handle: &mut tokio::task::JoinHandle<()>,
) -> ExitReason {
    select! {
        status = comet_child.wait() => ExitReason::CometExited(status),
        _ = abci_handle => ExitReason::AbciEnded,
        _ = signal::ctrl_c() => ExitReason::Signal,
    }
}

#[tokio::main]
async fn main() {
    let home_dir = expand_tilde("~/.fendermint");
    let config_dir = home_dir.join("config");
    let run_mode = "dev";
    let mut settings = Settings::new(&config_dir, &home_dir, run_mode).unwrap();

    settings.eth.tracing = TracingSettings::default();

    settings.eth.tracing.file = Some(FileLayerSettings {
        enabled: true,
        directory: Some(home_dir.join("logs")),
        level: default_log_level(),
        max_log_files: Some(5),
        rotation: Some(RotationKind::Daily),
        domain_filter: None,
        events_filter: None,
    });

    settings.tracing.file = Some(settings.tracing.file.unwrap_or_default());
    let _trace_file_guard = set_global_tracing_subscriber(&settings.eth.tracing);

    start_services(settings.clone()).await.unwrap();
}

async fn start_services(settings: Settings) -> anyhow::Result<()> {
    let abci_proxy = settings.abci.listen.to_string();

    // Create a cancellation token to gracefully shut down the ABCI server.
    let cancel_token = CancellationToken::new();

    let mut abci_handle = tokio::spawn({
        let settings = settings.clone();
        let token = cancel_token.clone();
        async move {
            if let Err(e) = run_node(settings, Some(token)).await {
                error!("ABCI server stopped with error: {:?}", e);
            }
            info!("ABCI server has ended. Runner will shut down.");
        }
    });

    // TODO: Load max_restarts from config.
    let max_restarts = 3;
    let mut attempts = 0;

    info!("Starting CometBFT with up to {max_restarts} restarts on crash...");

    loop {
        let spawn_result = spawn_cometbft(&abci_proxy).await;
        if let Err(e) = spawn_result {
            attempts += 1;
            error!("Failed to spawn CometBFT: {e}");
            if attempts >= max_restarts {
                error!("Max restarts reached on spawn error. Stopping runner.");
                break;
            }
            sleep(Duration::from_secs(3)).await;
            continue;
        }
        let mut comet_child = spawn_result.unwrap();

        let exit_reason = wait_for_exit(&mut comet_child, &mut abci_handle).await;

        match exit_reason {
            ExitReason::AbciEnded => {
                warn!("ABCI ended. Stopping CometBFT if still running...");
                kill_child(&mut comet_child, "CometBFT").await;
                break;
            }
            ExitReason::Signal => {
                warn!("Received Ctrl+C. Stopping CometBFT...");
                kill_child(&mut comet_child, "CometBFT").await;
                // Signal cancellation to the ABCI server.
                cancel_token.cancel();
                break;
            }
            ExitReason::CometExited(Ok(status)) => {
                let code = status.code().unwrap_or(-1);
                if code == 0 {
                    info!("CometBFT exited cleanly with code 0. Shutting down runner.");
                    break;
                } else {
                    attempts += 1;
                    error!("CometBFT crashed with code {code}. Restart attempt {attempts}/{max_restarts}.");
                    if attempts >= max_restarts {
                        error!("Reached max restarts. Exiting runner.");
                        break;
                    }
                    sleep(Duration::from_secs(3)).await;
                    info!("Restarting CometBFT...");
                }
            }
            ExitReason::CometExited(Err(e)) => {
                attempts += 1;
                error!("CometBFT process error: {e}. Restart attempt {attempts}/{max_restarts}.");
                if attempts >= max_restarts {
                    error!("Reached max restarts. Exiting runner.");
                    break;
                }
                sleep(Duration::from_secs(3)).await;
            }
        }
    }

    // Optionally wait for the ABCI server to shut down gracefully.
    let _ = abci_handle.await;

    info!("Runner is shutting down fully.");
    Ok(())
}

async fn spawn_cometbft(abci_addr: &str) -> anyhow::Result<Child> {
    // todo karel - use correct path
    let mut child = Command::new("cometbft")
        .args(["start", "--proxy_app", &format!("tcp://{}", abci_addr)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            tracing::info!(target: CONSENSUS_TARGET, "{}", line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            tracing::error!(target: CONSENSUS_TARGET, "{}", line);
        }
    });

    Ok(child)
}

/// Helper to kill a running child
async fn kill_child(child: &mut Child, name: &str) {
    if let Some(pid) = child.id() {
        let _ = child.kill().await;
        info!("{name} (pid={pid}) has been killed.");
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~") {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .expect("Could not determine home directory");
        PathBuf::from(home).join(stripped.trim_start_matches(std::path::MAIN_SEPARATOR))
    } else {
        PathBuf::from(path)
    }
}
