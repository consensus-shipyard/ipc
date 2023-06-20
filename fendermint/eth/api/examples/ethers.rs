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

// See https://coinsbench.com/ethereum-with-rust-tutorial-part-1-create-simple-transactions-with-rust-26d365a7ea93
// and https://coinsbench.com/ethereum-with-rust-tutorial-part-2-compile-and-deploy-solidity-contract-with-rust-c3cd16fce8ee

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context};
use clap::Parser;
use ethers::{
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider, ProviderError},
    signers::{Signer, Wallet},
};
use ethers_core::{
    k256::ecdsa::SigningKey,
    types::{
        Address, BlockId, BlockNumber, Eip1559TransactionRequest, TransactionReceipt, H160, H256,
        U256, U64,
    },
};
use fendermint_rpc::message::MessageFactory;
use fendermint_vm_actor_interface::eam::EthAddress;
use libsecp256k1::SecretKey;
use tracing::Level;

type TestMiddleware = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

#[derive(Parser, Debug)]
pub struct Options {
    /// The host of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "127.0.0.1", env = "FM_ETH__HTTP__HOST")]
    pub http_host: String,

    /// The port of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "8545", env = "FM_ETH__HTTP__PORT")]
    pub http_port: u32,

    /// Secret key used to send funds, expected to be in Base64 format.
    ///
    /// Assumed to exist with a non-zero balance.
    #[arg(long, short)]
    pub secret_key_from: PathBuf,

    /// Secret key used to receive funds, expected to be in Base64 format.
    #[arg(long, short)]
    pub secret_key_to: PathBuf,

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

    let mut provider = Provider::<Http>::try_from(opts.http_endpoint())?;

    // Tendermint block interval is lower.
    provider.set_interval(Duration::from_secs(2));

    tracing::debug!("running the tests...");
    run(provider, opts).await?;

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
    tracing::debug!("checking request {method}...");
    match res {
        Ok(value) => match check(&value).check_result() {
            Ok(()) => Ok(value),
            Err(e) => Err(anyhow!("failed to check {method}: {e}:\n{value:?}")),
        },
        Err(e) => Err(anyhow!("failed to call {method}: {e}")),
    }
}

struct TestAccount {
    secret_key: SecretKey,
    eth_addr: H160,
}

impl TestAccount {
    pub fn new(sk: &Path) -> anyhow::Result<Self> {
        let sk = MessageFactory::read_secret_key(sk)?;
        let pk = libsecp256k1::PublicKey::from_secret_key(&sk);
        let ea = EthAddress::new_secp256k1(&pk.serialize())?;
        let h = Address::from_slice(&ea.0);

        Ok(Self {
            secret_key: sk,
            eth_addr: h,
        })
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
// - eth_sendRawTransaction
//
// TODO:
// - eth_newBlockFilter
// - eth_newPendingTransactionFilter
// - eth_getBlockReceipts
// - eth_syncing
// - eth_createAccessList
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
// - eth_call
// - eth_estimateGas
// - eth_getStorageAt
// - eth_getCode
//
// WON'T DO:
// - eth_sign
// - eth_sendTransaction
//

/// Exercise the above methods, so we know at least the parameters are lined up correctly.
async fn run(provider: Provider<Http>, opts: Options) -> anyhow::Result<()> {
    let from = TestAccount::new(&opts.secret_key_from)?;
    let to = TestAccount::new(&opts.secret_key_to)?;

    tracing::info!(from = ?from.eth_addr, to = ?to.eth_addr, "ethereum address");

    request("eth_accounts", provider.get_accounts().await, |acnts| {
        acnts.is_empty()
    })?;

    let bn = request("eth_blockNumber", provider.get_block_number().await, |bn| {
        bn.as_u64() > 0
    })?;

    let chain_id = request("eth_chainId", provider.get_chainid().await, |id| {
        !id.is_zero()
    })?;

    let mw = make_middleware(provider.clone(), chain_id.as_u64(), &from)
        .context("failed to create middleware")?;

    request(
        "eth_getBalance",
        provider.get_balance(from.eth_addr, None).await,
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
            .get_transaction_count(from.eth_addr, Some(BlockId::Number(BlockNumber::Earliest)))
            .await,
        |u| u.is_zero(),
    )?;

    // Get a block without transactions
    let b = request(
        "eth_getBlockByNumber w/o txns",
        provider
            .get_block(BlockId::Number(BlockNumber::Number(bn)))
            .await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    let bh = b.unwrap().hash.expect("hash should be set");

    // Get the same block without transactions by hash.
    request(
        "eth_getBlockByHash w/o txns",
        provider.get_block(BlockId::Hash(bh)).await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    let base_fee = request("eth_gasPrice", provider.get_gas_price().await, |id| {
        !id.is_zero()
    })?;

    // Send the transaction and wait for receipt
    let receipt = example_transfer(mw, to).await.context("transfer failed")?;
    let tx_hash = receipt.transaction_hash;
    let bn = receipt.block_number.unwrap();
    let bh = receipt.block_hash.unwrap();

    tracing::info!(height = ?bn, ?tx_hash, "example transfer");

    // Get a block with transactions by number.
    request(
        "eth_getBlockByNumber /w txns",
        provider
            .get_block_with_txs(BlockId::Number(BlockNumber::Number(bn)))
            .await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    // Get the block with transactions by hash.
    request(
        "eth_getBlockByHash /w txns",
        provider.get_block_with_txs(BlockId::Hash(bh)).await,
        |b| b.is_some() && b.as_ref().map(|b| b.number).flatten() == Some(bn),
    )?;

    // By now there should be a transaction in a block.
    request(
        "eth_feeHistory",
        provider
            .fee_history(
                U256::from(100),
                BlockNumber::Latest,
                &[0.25, 0.5, 0.75, 0.95],
            )
            .await,
        |hist| {
            hist.base_fee_per_gas.len() > 0
                && *hist.base_fee_per_gas.last().unwrap() == base_fee
                && hist.gas_used_ratio.iter().any(|r| *r > 0.0)
        },
    )?;

    request(
        "eth_getTransactionByHash",
        provider.get_transaction(tx_hash).await,
        |tx| tx.is_some(),
    )?;

    request(
        "eth_getTransactionReceipt",
        provider.get_transaction_receipt(tx_hash).await,
        |tx| tx.is_some(),
    )?;

    Ok(())
}

/// Make an example transfer.
async fn example_transfer(
    mw: TestMiddleware,
    to: TestAccount,
) -> anyhow::Result<TransactionReceipt> {
    // Create a transaction to transfer 1000 atto.
    let tx = Eip1559TransactionRequest::new().to(to.eth_addr).value(1000);

    // Set the gas based on the testkit so it doesn't trigger estimation (which isn't implemented yet).
    let tx = tx
        .gas(10_000_000_000u64)
        .max_fee_per_gas(0)
        .max_priority_fee_per_gas(0);

    // `send_transaction` will fill in the missing fields like `from` and `nonce` (which involves querying the API).
    let receipt = mw
        .send_transaction(tx, None)
        .await
        .context("failed to send transaction")?
        .log_msg("Pending transfer")
        .retries(5)
        .await?
        .context("Missing receipt")?;

    Ok(receipt)
}

/// Create a middleware that will assign nonces and sign the message.
fn make_middleware(
    provider: Provider<Http>,
    chain_id: u64,
    sender: &TestAccount,
) -> anyhow::Result<TestMiddleware> {
    // We have to use Ethereum's signing scheme, beause the `from` is not part of the RLP representation,
    // it is inferred from the public key recovered from the signature. We could potentially hash the
    // transaction in a different way, but we can't for example use the actor ID in the hash, because
    // we have no way of sending it along with the message.
    let wallet: Wallet<SigningKey> =
        Wallet::from_bytes(&sender.secret_key.serialize())?.with_chain_id(chain_id);

    Ok(SignerMiddleware::new(provider, wallet))
}
