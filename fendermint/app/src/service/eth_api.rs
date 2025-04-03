// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use fendermint_eth_api::HybridClient;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::metrics::register_eth_metrics;
use fendermint_app_settings::eth::EthSettings;

/// Run the Ethereum API facade.
pub async fn run(
    settings: EthSettings,
    client: HybridClient,
    cancel_token: Option<CancellationToken>,
) -> anyhow::Result<()> {
    if settings.metrics.enabled {
        info!("metrics enabled");

        let registry = prometheus::Registry::new();

        register_eth_metrics(&registry).context("failed to register metrics")?;

        info!(
            listen_addr = settings.metrics.listen.to_string(),
            "serving metrics"
        );
        let mut builder = prometheus_exporter::Builder::new(settings.metrics.listen.try_into()?);
        builder.with_registry(registry);
        let _ = builder.start().context("failed to start metrics server")?;
    } else {
        info!("metrics disabled");
    }

    let gas = fendermint_eth_api::GasOpt {
        min_gas_premium: settings.gas.min_gas_premium,
        num_blocks_max_prio_fee: settings.gas.num_blocks_max_prio_fee,
        max_fee_hist_size: settings.gas.max_fee_hist_size,
    };
    let cors = fendermint_eth_api::CorsOpt {
        allowed_origins: settings.cors.allowed_origins,
        allowed_methods: settings.cors.allowed_methods,
        allowed_headers: settings.cors.allowed_headers,
    };
    fendermint_eth_api::listen(
        settings.listen,
        client,
        settings.filter_timeout,
        settings.cache_capacity,
        settings.max_nonce_gap,
        gas,
        cors,
        cancel_token,
    )
    .await
}
