// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_rpc::client::ws_client;
use tendermint_rpc::WebSocketClient;

use crate::{
    cmd,
    options::eth::{EthArgs, EthCommands},
    settings::EthSettings,
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
    fendermint_eth_api::listen(
        settings.listen.addr(),
        client,
        settings.filter_timeout,
        settings.cache_capacity,
        settings.gas,
    )
    .await
}
