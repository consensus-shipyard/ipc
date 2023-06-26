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
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use ethers::{
    prelude::{abigen, ContractCall, ContractFactory, SignerMiddleware},
    providers::{Http, Middleware, Provider, ProviderError},
    signers::{Signer, Wallet},
};
use ethers_core::{
    k256::ecdsa::SigningKey,
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, BlockNumber, Bytes,
        Eip1559TransactionRequest, TransactionReceipt, H160, H256, U256, U64,
    },
};
use fendermint_rpc::message::MessageFactory;
use fendermint_vm_actor_interface::eam::EthAddress;
use libsecp256k1::SecretKey;
use tracing::Level;

type TestMiddleware = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;
type TestContractCall<T> = ContractCall<TestMiddleware, T>;

// This assumes that https://github.com/filecoin-project/builtin-actors is checked out next to this project,
// which the Makefile in the root takes care of with `make actor-bundle`, a dependency of creating docker images.
const SIMPLECOIN_HEX: &'static str =
    include_str!("../../../../../builtin-actors/actors/evm/tests/contracts/SimpleCoin.bin");

const SIMPLECOIN_ABI: &'static str =
    include_str!("../../../../../builtin-actors/actors/evm/tests/contracts/SimpleCoin.abi");

/// Gas limit to set for transactions.
const ENOUGH_GAS: u64 = 10_000_000_000u64;

// Generate a statically typed interface for the contract.
// An example of what it looks like is at https://github.com/filecoin-project/ref-fvm/blob/evm-integration-tests/testing/integration/tests/evm/src/simple_coin/simple_coin.rs
abigen!(
    SimpleCoin,
    "../../../../builtin-actors/actors/evm/tests/contracts/SimpleCoin.abi"
);

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
// - eth_sendRawTransaction
// - eth_call
// - eth_estimateGas
//
// DOING:
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

    // Go back one block, so we can be sure there are results.
    let bn = bn - 1;

    let chain_id = request("eth_chainId", provider.get_chainid().await, |id| {
        !id.is_zero()
    })?;

    let mw = make_middleware(provider.clone(), chain_id.as_u64(), &from)
        .context("failed to create middleware")?;
    let mw = Arc::new(mw);

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

    tracing::info!("sending example transfer");

    let transfer = make_transfer(&mw, to)
        .await
        .context("failed to make a transfer")?;

    let receipt = send_transaction(&mw, transfer.clone())
        .await
        .context("failed to send transfer")?;

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

    // Calling with 0 nonce so the node figures out the latest value.
    let mut probe_tx = transfer.clone();
    probe_tx.set_nonce(0);
    let probe_height = BlockId::Number(BlockNumber::Number(bn));

    request(
        "eth_call",
        provider.call(&probe_tx, Some(probe_height)).await,
        |_| true,
    )?;

    request(
        "eth_estimateGas",
        provider.estimate_gas(&probe_tx, Some(probe_height)).await,
        |gas: &U256| !gas.is_zero(),
    )?;

    tracing::info!("deploying SimpleCoin");

    let bytecode =
        Bytes::from(hex::decode(SIMPLECOIN_HEX).context("failed to decode contract hex")?);
    let abi = serde_json::from_str::<ethers::core::abi::Abi>(SIMPLECOIN_ABI)?;

    let factory = ContractFactory::new(abi, bytecode, mw.clone());
    let mut deployer = factory.deploy(())?;

    // Fill the fields so we can debug any difference between this and the node.
    // Using `Some` block ID because with `None` the eth_estimateGas call would receive invalid parameters.
    mw.fill_transaction(&mut deployer.tx, Some(BlockId::Number(BlockNumber::Latest)))
        .await?;
    tracing::info!(sighash = ?deployer.tx.sighash(), "deployment tx");

    // NOTE: This will call eth_estimateGas to figure out how much gas to use, because we don't set it,
    // unlike in the case of the example transfer. What the [Provider::fill_transaction] will _also_ do
    // is estimate the fees using eth_feeHistory, here:
    // https://github.com/gakonst/ethers-rs/blob/df165b84229cdc1c65e8522e0c1aeead3746d9a8/ethers-providers/src/rpc/provider.rs#LL300C30-L300C51
    // These were set to zero in the earlier example transfer, ie. it was basically paid for by the miner (which is not at the moment charged),
    // so the test passed. Here, however, there will be a non-zero cost to pay by the deployer, and therefore those balances
    // have to be much higher than the defaults used earlier, e.g. the deployment cost 30 FIL, and we used to give 1 FIL.
    let (contract, receipt) = deployer
        .send_with_receipt()
        .await
        .context("failed to send deployment")?;

    tracing::info!(addr = ?contract.address(), "SimpleCoin deployed");

    let contract = SimpleCoin::new(contract.address(), contract.client());

    let _tx_hash = receipt.transaction_hash;
    let _bn = receipt.block_number.unwrap();
    let _bh = receipt.block_hash.unwrap();

    let mut coin_call: TestContractCall<U256> = contract.get_balance(from.eth_addr);
    mw.fill_transaction(
        &mut coin_call.tx,
        Some(BlockId::Number(BlockNumber::Latest)),
    )
    .await?;

    let coin_balance: U256 = coin_call.call().await.context("coin balance call failed")?;

    if coin_balance != U256::from(10000) {
        bail!("unexpected coin balance: {coin_balance}");
    }

    Ok(())
}

async fn make_transfer(mw: &TestMiddleware, to: TestAccount) -> anyhow::Result<TypedTransaction> {
    // Create a transaction to transfer 1000 atto.
    let tx = Eip1559TransactionRequest::new().to(to.eth_addr).value(1000);

    // Set the gas based on the testkit so it doesn't trigger estimation.
    let mut tx = tx
        .gas(ENOUGH_GAS)
        .max_fee_per_gas(0)
        .max_priority_fee_per_gas(0)
        .into();

    // Fill in the missing fields like `from` and `nonce` (which involves querying the API).
    mw.fill_transaction(&mut tx, None).await?;

    Ok(tx)
}

async fn send_transaction(
    mw: &TestMiddleware,
    tx: TypedTransaction,
) -> anyhow::Result<TransactionReceipt> {
    // `send_transaction` will fill in the missing fields like `from` and `nonce` (which involves querying the API).
    let receipt = mw
        .send_transaction(tx, None)
        .await
        .context("failed to send transaction")?
        .log_msg("Pending transaction")
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
