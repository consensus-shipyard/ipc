use crate::services::Service;
use anyhow::{Context, Ok, Result};
use async_trait::async_trait;
use fendermint_app::service::eth_api::run as run_eth_api;
use fendermint_app_settings::Settings;
use fendermint_eth_api::HybridClient;
use std::time::Duration;
use tokio::select;
use tokio_util::sync::CancellationToken;

pub struct EthApiService {
    settings: Settings,
    connect_retry_delay: Duration,
}

impl EthApiService {
    pub fn new(settings: Settings, connect_retry_delay: Duration) -> Self {
        Self {
            settings,
            connect_retry_delay,
        }
    }
}

#[async_trait]
impl Service for EthApiService {
    fn name(&self) -> &'static str {
        "ETH API Service"
    }

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        let http_url = self
            .settings
            .tendermint_rpc_url()
            .context("invalid Tendermint URL")?;

        let ws_url = self
            .settings
            .tendermint_websocket_url()
            .context("invalid Tendermint WS URL")?;

        let (client, driver) = HybridClient::new(http_url, ws_url, self.connect_retry_delay)
            .context("failed to create HybridClient")?;

        let driver_shutdown = shutdown.child_token();

        let driver_future = async {
            select! {
                _ = driver.run() => Ok(()),
                _ = driver_shutdown.cancelled() => Ok(()),
            }
        };

        let api_future = run_eth_api(self.settings.eth.clone(), client, Some(shutdown.clone()));

        let result = select! {
            api_result = api_future => {
                shutdown.cancel();
                api_result
            },
            _ = driver_future => {
                shutdown.cancel();
                Err(anyhow::anyhow!("Driver terminated unexpectedly"))
            },
        };

        result
    }
}
