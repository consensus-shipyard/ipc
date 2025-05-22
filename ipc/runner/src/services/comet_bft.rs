use super::Service;
use crate::process_manager::{kill_process, ProcessManager};
use anyhow::Result;
use async_trait::async_trait;
use fendermint_app_settings::SocketAddress;
use tokio::select;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

pub struct CometBftService {
    /// The proxy address to pass to CometBFT.
    pub abci_proxy: SocketAddress,
    /// Maximum restart attempts before giving up.
    pub max_restarts: u32,
}

impl CometBftService {
    pub fn new(abci_proxy: SocketAddress, max_restarts: u32) -> Self {
        Self {
            abci_proxy,
            max_restarts,
        }
    }
}

#[async_trait]
impl Service for CometBftService {
    fn name(&self) -> &'static str {
        "CometBFT Service"
    }

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        let mut attempts = 0;

        let proxy_app = format!("tcp://{}", self.abci_proxy);
        let process_manager =
            ProcessManager::new("cometbft".into(), vec!["start", "--proxy_app", &proxy_app]);

        loop {
            if shutdown.is_cancelled() {
                tracing::info!("Shutdown signal received for CometBFT service.");
                break;
            }

            let spawn_result = process_manager.spawn().await;
            let mut child = match spawn_result {
                Ok(child) => child,
                Err(e) => {
                    attempts += 1;
                    tracing::error!(
                        "Failed to spawn CometBFT: {}. Restart attempt {}/{}",
                        e,
                        attempts,
                        self.max_restarts
                    );
                    if attempts >= self.max_restarts {
                        tracing::error!("Max restarts reached for CometBFT. Exiting service.");
                        return Err(e.into());
                    }
                    sleep(Duration::from_secs(3)).await;
                    continue;
                }
            };

            // Wait for either the process to exit or a shutdown signal.
            let exit_status = select! {
                status = child.wait() => status,
                _ = shutdown.cancelled() => {
                    kill_process(&mut child, "CometBFT").await;
                    break;
                }
            };

            match exit_status {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    if code == 0 {
                        tracing::info!("CometBFT exited cleanly with code 0.");
                        break;
                    } else {
                        attempts += 1;
                        tracing::error!(
                            "CometBFT crashed with code {}. Restart attempt {}/{}.",
                            code,
                            attempts,
                            self.max_restarts
                        );
                        if attempts >= self.max_restarts {
                            tracing::error!("Max restarts reached for CometBFT. Exiting service.");
                            break;
                        }
                        sleep(Duration::from_secs(3)).await;
                    }
                }
                Err(e) => {
                    attempts += 1;
                    tracing::error!(
                        "Error waiting for CometBFT process: {}. Restart attempt {}/{}.",
                        e,
                        attempts,
                        self.max_restarts
                    );
                    if attempts >= self.max_restarts {
                        tracing::error!("Max restarts reached for CometBFT. Exiting service.");
                        break;
                    }
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }

        Ok(())
    }
}
