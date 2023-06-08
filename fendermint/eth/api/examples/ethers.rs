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
//!      -d '{"jsonrpc":"2.0","id":0,"method":"eth_getBlockTransactionCountByNumber","params":["0x1"]}' \
//!      http://localhost:8545
//! ```

use std::fmt::Debug;

use anyhow::anyhow;
use clap::Parser;
use ethers::providers::{Http, Middleware, Provider, ProviderError};
use ethers_core::types::{BlockId, BlockNumber, H256, U256, U64};
use fendermint_vm_actor_interface::eam::EthAddress;
use tracing::Level;

#[derive(Parser, Debug)]
pub struct Options {
    /// The host of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "127.0.0.1", env = "FM_ETH__HTTP__HOST")]
    pub http_host: String,

    /// The port of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "8545", env = "FM_ETH__HTTP__PORT")]
    pub http_port: u32,

    /// ID of the actor who is the subject of our enquiries.
    ///
    /// Assumed to exist with a non-zero balance.
    #[arg(long, short)]
    pub actor_id: u64,

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

    run(provider, opts.actor_id).await?;

    Ok(())
}

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

// The following methods are called by the [`Provider`].
// This is not an exhaustive list of JSON-RPC methods that the API implements, just what the client library calls.
//
// DONE:
// - eth_accounts
// - eth_blockNumber
// - eth_chainId
// - eth_getBalance
// - eth_getUncleCountByBlockHash
// - eth_getUncleCountByBlockNumber
// - eth_getUncleByBlockHashAndIndex
// - eth_getUncleByBlockNumberAndIndex
// - eth_getTransactionCount
// - eth_gasPrice
// - eth_getBlockByHash
// - eth_getBlockByNumber
// - eth_getTransactionByHash
// - eth_getTransactionReceipt
// - eth_feeHistory
//
// DOING:
//
// TODO:
// - eth_sendRawTransaction
// - eth_newBlockFilter
// - eth_newPendingTransactionFilter
// - eth_getBlockReceipts
// - eth_syncing
// - eth_createAccessList
// - eth_sendTransaction
// - eth_sign
// - eth_getLogs
// - eth_newBlockFilter
// - eth_newPendingTransactionFilter
// - eth_newFilter
// - eth_uninstallFilter
// - eth_getFilterChanges
// - eth_getProof
// - eth_mining
// - eth_subscribe
// - eth_unsubscribe
//
// BLOCKED:
// - eth_call
// - eth_estimateGas
// - eth_getStorageAt
// - eth_getCode

/// Exercise the above methods, so we know at least the parameters are lined up correctly.
async fn run(provider: Provider<Http>, actor_id: u64) -> anyhow::Result<()> {
    let addr = EthAddress::from_id(actor_id);
    let addr = ethers::core::types::Address::from_slice(&addr.0);

    request("eth_accounts", provider.get_accounts().await, |acnts| {
        acnts.is_empty()
    })?;

    let bn = request("eth_blockNumber", provider.get_block_number().await, |bn| {
        bn.as_u64() > 0
    })?;

    request("eth_chainId", provider.get_chainid().await, |id| {
        !id.is_zero()
    })?;

    request(
        "eth_getBalance",
        provider.get_balance(addr, None).await,
        |b| !b.is_zero(),
    )?;

    request(
        "eth_getUncleCountByBlockHash",
        provider
            .get_uncle_count(BlockId::Hash(H256([0u8; 32])))
            .await,
        |uc| uc.is_zero(),
    )?;

    request(
        "eth_getUncleCountByBlockNumber",
        provider
            .get_uncle_count(BlockId::Number(BlockNumber::Number(bn)))
            .await,
        |uc| uc.is_zero(),
    )?;

    request(
        "eth_getUncleByBlockHashAndIndex",
        provider
            .get_uncle(BlockId::Hash(H256([0u8; 32])), U64::from(0))
            .await,
        |u| u.is_none(),
    )?;

    request(
        "eth_getUncleByBlockNumberAndIndex",
        provider
            .get_uncle(BlockId::Number(BlockNumber::Number(bn)), U64::from(0))
            .await,
        |u| u.is_none(),
    )?;

    // Querying at genesis, so the transaction count should be zero.
    request(
        "eth_getTransactionCount",
        provider
            .get_transaction_count(addr, Some(BlockId::Number(BlockNumber::Earliest)))
            .await,
        |u| u.is_zero(),
    )?;

    let b = request(
        "eth_getBlockByNumber",
        provider
            .get_block(BlockId::Number(BlockNumber::Number(bn)))
            .await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    let bh = b.unwrap().hash.expect("hash should be set");

    request(
        "eth_getBlockByHash",
        provider.get_block(BlockId::Hash(bh)).await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    // TODO: Use a block which we know has transactions.
    request(
        "eth_getBlockByNumber",
        provider
            .get_block_with_txs(BlockId::Number(BlockNumber::Number(bn)))
            .await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    // TODO: Use a block which we know has transactions.
    request(
        "eth_getBlockByHash",
        provider.get_block_with_txs(BlockId::Hash(bh)).await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    let base_fee = request("eth_gasPrice", provider.get_gas_price().await, |id| {
        !id.is_zero()
    })?;

    // TODO: Get the fee history after transactions have been added.
    request(
        "eth_feeHistory",
        provider
            .fee_history(
                U256::from(10),
                BlockNumber::Latest,
                &[0.25, 0.5, 0.75, 0.95],
            )
            .await,
        |hist| {
            hist.base_fee_per_gas.len() > 0 && *hist.base_fee_per_gas.last().unwrap() == base_fee
        },
    )?;

    // TODO: Get an existing transaction
    request(
        "eth_getTransactionByHash",
        provider.get_transaction(H256::default()).await,
        |tx| tx.is_none(),
    )?;

    // TODO: Get an existing transaction
    request(
        "eth_getTransactionReceipt",
        provider.get_transaction_receipt(H256::default()).await,
        |tx| tx.is_none(),
    )?;

    Ok(())
}
