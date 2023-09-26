// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_rpc::client::ws_client;
use tendermint_rpc::WebSocketClient;

use crate::{
    cmd,
    options::eth::{EthArgs, EthCommands},
    settings::eth::EthSettings,
};

cmd! {
  EthArgs(self, settings: EthSettings) {
    match self.command.clone() {
      EthCommands::Run { url, proxy_url: _ } => {
        let (client, driver) = ws_client(url).await?;
        let driver_handle = tokio::spawn(async move { driver.run().await });

        let result = run(settings, client).await;

        // Await the driver's termination to ensure proper connection closure.
        let _ = driver_handle.await;
        result
      }
    }
  }
}

/// Run the Ethereum
async fn run(settings: EthSettings, client: WebSocketClient) -> anyhow::Result<()> {
    let gas = fendermint_eth_api::GasOpt {
        min_gas_premium: settings.gas.min_gas_premium,
        num_blocks_max_prio_fee: settings.gas.num_blocks_max_prio_fee,
        max_fee_hist_size: settings.gas.max_fee_hist_size,
    };
    fendermint_eth_api::listen(
        settings.listen,
        client,
        settings.filter_timeout,
        settings.cache_capacity,
        gas,
    )
    .await
}
