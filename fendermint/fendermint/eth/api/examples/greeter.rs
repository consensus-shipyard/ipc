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
//! cargo run -p fendermint_eth_api --release --example GREETER --
//! ```

use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Context};
use clap::Parser;
use ethers::{contract::LogMeta, providers::StreamExt};
use ethers::{
    prelude::{abigen, ContractCall, ContractFactory, SignerMiddleware},
    providers::{FilterKind, Http, JsonRpcClient, Middleware, Provider, Ws},
    signers::{Signer, Wallet},
};
use ethers_core::{
    abi::Abi,
    k256::ecdsa::SigningKey,
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, BlockNumber, Bytes,
        Eip1559TransactionRequest, Filter, Log, SyncingStatus, TransactionReceipt, TxHash, H160,
        H256, U256, U64,
    },
};
use fendermint_crypto::SecretKey;
use fendermint_rpc::message::MessageFactory;
use fendermint_vm_actor_interface::eam::EthAddress;
use tracing::Level;

use crate::common::{adjust_provider, make_middleware, TestAccount};

mod common;

/// Disabling filters helps when inspecting docker logs. The background data received for filters is rather noisy.
const FILTERS_ENABLED: bool = true;

// Generate a statically typed interface for the contract.
abigen!(Greeter, "../../testing/contracts/Greeter.abi");

const GREETER_HEX: &'static str = include_str!("../../../testing/contracts/Greeter.bin");

#[derive(Parser, Debug)]
pub struct Options {
    /// The host of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "127.0.0.1", env = "FM_ETH__LISTEN__HOST")]
    pub http_host: String,

    /// The port of the Fendermint Ethereum API endpoint.
    #[arg(long, default_value = "8545", env = "FM_ETH__LISTEN__PORT")]
    pub http_port: u32,

    /// Secret key used to deploy the contract.
    ///
    /// Assumed to exist with a non-zero balance.
    #[arg(long, short)]
    pub secret_key_from: PathBuf,

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
    run_http(provider, &opts).await?;

    Ok(())
}

async fn run<C>(provider: &Provider<C>, opts: &Options) -> anyhow::Result<()>
where
    C: JsonRpcClient + Clone + 'static,
{
    let from = TestAccount::new(&opts.secret_key_from)?;

    tracing::info!(from = ?from.eth_addr, "ethereum address");
    tracing::info!("deploying Greeter");

    let bytecode = Bytes::from(hex::decode(GREETER_HEX).context("failed to decode contract hex")?);
    let abi: Abi = GREETER_ABI.clone();

    let chain_id = provider.get_chainid().await?;

    let mw = make_middleware(provider.clone(), chain_id.as_u64(), &from)
        .context("failed to create middleware")?;

    let mw = Arc::new(mw);

    const GREETING0: &str = "Hello, weary traveller!";
    const GREETING1: &str = "Howdy doody!";

    let factory = ContractFactory::new(abi, bytecode.clone(), mw.clone());
    let mut deployer = factory.deploy((GREETING0.to_string(),))?;

    let (contract, deploy_receipt): (_, TransactionReceipt) = deployer
        .send_with_receipt()
        .await
        .context("failed to send deployment")?;

    tracing::info!(addr = ?contract.address(), "Greeter deployed");

    let contract = Greeter::new(contract.address(), contract.client());

    let greeting: String = contract
        .greet()
        .call()
        .await
        .context("failed to call greet")?;

    assert_eq!(greeting, GREETING0);

    let block_height = provider
        .get_block_number()
        .await
        .context("failed to get block number")?;

    // Set the greeting to emit an event.
    contract
        .set_greeting(GREETING1.to_string())
        .call()
        .await
        .context("failed to set greeting")?;

    let greeting: String = contract
        .greet()
        .call()
        .await
        .context("failed to call greet")?;

    assert_eq!(greeting, GREETING1);

    let logs: Vec<(_, LogMeta)> = contract
        .greeting_set_filter()
        .from_block(block_height)
        .query_with_meta()
        .await
        .context("failed to query logs")?;

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].0.greeting, GREETING1);

    let log_meta: LogMeta = logs[0].1;

    Ok(())
}

/// The HTTP interface provides JSON-RPC request/response endpoints.
async fn run_http(mut provider: Provider<Http>, opts: &Options) -> anyhow::Result<()> {
    tracing::info!("Running the tests over HTTP...");
    adjust_provider(&mut provider);
    run(&provider, opts).await?;
    tracing::info!("HTTP tests finished");
    Ok(())
}
