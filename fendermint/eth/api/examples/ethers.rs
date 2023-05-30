// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Example of using the Ethereum JSON-RPC facade with the Ethers provider.
//!
//! The example assumes that the following has been started and running in the background:
//! 1. Fendermint ABCI application
//! 2. Tendermint Core / Comet BFT
//! 3. Fendermint Ethereum API facade
//!
//! # Usage
//! ```text
//! cargo run -p fendermint_eth_api --release --example ethers --
//! ```
//!
//! A method can also be called directly with `curl`:
//!
//! ```text
//! curl -X POST -i \
//!      -H 'Content-Type: application/json' \
//!      -d '{"jsonrpc":"2.0","id":"id","method":"eth_blockNumber","params":[]}' \
//!      http://localhost:8545
//! ```

use std::fmt::Debug;

use anyhow::anyhow;
use clap::Parser;
use ethers::providers::{Http, Middleware, Provider, ProviderError};
use tracing::Level;

#[derive(Parser, Debug)]
pub struct Options {
    /// The host of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "127.0.0.1", env = "FM_ETH__HTTP__HOST")]
    pub http_host: String,

    /// The port of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "8545", env = "FM_ETH__HTTP__PORT")]
    pub http_port: u32,

    /// Enable DEBUG logs.
    #[arg(long, short)]
    pub verbose: bool,
}

impl Options {
    pub fn log_level(&self) -> Level {
        if self.verbose {
            Level::DEBUG
        } else {
            Level::INFO
        }
    }

    pub fn http_endpoint(&self) -> String {
        format!("http://{}:{}", self.http_host, self.http_port)
    }
}

/// See the module docs for how to run.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Options = Options::parse();

    tracing_subscriber::fmt()
        .with_max_level(opts.log_level())
        .init();

    let provider = Provider::<Http>::try_from(opts.http_endpoint())?;

    run(provider).await?;

    Ok(())
}

// The following methods are called by the Provider.
//
// OK:
// - eth_accounts
// - eth_blockNumber
//
// TODO:
// - eth_newBlockFilter
// - eth_newBlockFilter
// - eth_newPendingTransactionFilter
// - eth_getBlockByHash
// - eth_getBlockByNumber
// - eth_call
// - eth_getUncleCountByBlockHash
// - eth_getUncleCountByBlockNumber
// - eth_getUncleByBlockHashAndIndex
// - eth_getUncleByBlockNumberAndIndex
// - eth_getTransactionByHash
// - eth_getTransactionReceipt
// - eth_getBlockReceipts
// - eth_gasPrice
// - eth_getTransactionCount
// - eth_getBalance
// - eth_chainId
// - eth_syncing
// - eth_call
// - eth_estimateGas
// - eth_createAccessList
// - eth_sendTransaction
// - eth_sendRawTransaction
// - eth_sign
// - eth_sign
// - eth_getLogs
// - eth_newBlockFilter
// - eth_newPendingTransactionFilter
// - eth_newFilter
// - eth_uninstallFilter
// - eth_getFilterChanges
// - eth_getstorageat
// - eth_getStorageAt
// - eth_getCode
// - eth_getProof
// - eth_mining
// - eth_subscribe
// - eth_unsubscribe
// - eth_feeHistory
// - eth_feeHistory
// - eth_blockNumber
// - eth_getChainId
// - eth_estimateGas
// - geth_admin_nodeinfo
// - spawn_geth_and_create_provider
// - spawn_geth_and_create_provider
// - spawn_geth_instances
// - spawn_geth_and_create_provider
// - add_second_geth_peer
// - spawn_geth_instances

trait CheckResult {
    fn check_result(&self) -> anyhow::Result<()>;
}

impl CheckResult for bool {
    fn check_result(&self) -> anyhow::Result<()> {
        if *self {
            Ok(())
        } else {
            Err(anyhow!("expected true; got false"))
        }
    }
}

fn request<T, F, C>(method: &str, res: Result<T, ProviderError>, check: F) -> anyhow::Result<T>
where
    T: Debug,
    F: FnOnce(&T) -> C,
    C: CheckResult,
{
    match res {
        Ok(value) => match check(&value).check_result() {
            Ok(()) => Ok(value),
            Err(e) => Err(anyhow!("failed to check {method}: {e}:\n{value:?}")),
        },
        Err(e) => Err(anyhow!("failed to call {method}: {e}")),
    }
}

async fn run(provider: Provider<Http>) -> anyhow::Result<()> {
    request("eth_blockNumber", provider.get_block_number().await, |bn| {
        bn.as_u64() > 0
    })?;

    request("eth_accounts", provider.get_accounts().await, |acnts| {
        acnts.is_empty()
    })?;

    Ok(())
}
