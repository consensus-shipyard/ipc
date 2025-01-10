// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::Arc;
use std::time::Duration;

use crate::make_testnet;
use anyhow::{bail, Context};
use ethers::prelude::{Block, H256};
use ethers::types::U256;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{JsonRpcClient, Middleware, PendingTransaction, Provider},
    signers::{Signer, Wallet},
    types::{Eip1559TransactionRequest, H160},
};
use fendermint_materializer::bencher::Bencher;
use fendermint_materializer::concurrency::collect::collect_blocks;
use fendermint_materializer::concurrency::nonce_manager::NonceManager;
use fendermint_materializer::concurrency::signal::Signal;
use fendermint_materializer::{
    concurrency::{self, config::Execution},
    materials::DefaultAccount,
    HasEthApi,
};
use futures::FutureExt;
use tokio::time::sleep;

const MANIFEST: &str = "benches.yaml";

pub type TestMiddleware<C> = SignerMiddleware<Provider<C>, Wallet<SigningKey>>;

/// Create a middleware that will assign nonces and sign the message.
async fn make_middleware<C>(
    provider: Provider<C>,
    sender: &DefaultAccount,
    chain_id: U256,
) -> anyhow::Result<TestMiddleware<C>>
where
    C: JsonRpcClient,
{
    let wallet: Wallet<SigningKey> = Wallet::from_bytes(sender.secret_key().serialize().as_ref())?
        .with_chain_id(chain_id.as_u64());

    Ok(SignerMiddleware::new(provider, wallet))
}

const BLOCK_GAS_LIMIT: u64 = 10_000_000_000;
const MAX_TX_GAS_LIMIT: u64 = 3_000_000;

#[serial_test::serial]
#[tokio::test]
async fn test_concurrent_transfer() -> Result<(), anyhow::Error> {
    let (testnet, cleanup) = make_testnet(MANIFEST, |_| {}).await?;

    let block_gas_limit = U256::from(BLOCK_GAS_LIMIT);
    let max_tx_gas_limit = U256::from(MAX_TX_GAS_LIMIT);

    let pangea = testnet.node(&testnet.root().node("pangea"))?;
    let provider = pangea
        .ethapi_http_provider()?
        .expect("ethapi should be enabled");
    let chain_id = provider
        .get_chainid()
        .await
        .context("failed to get chain ID")?;

    let cancel = Arc::new(Signal::new());

    // Set up background blocks collector.
    let blocks_collector = {
        let cancel = cancel.clone();
        let provider = provider.clone();
        let assert = move |block: &Block<H256>| {
            // Make sure block gas limit isn't the bottleneck.
            let unused_gas_limit = block_gas_limit - block.gas_limit;
            assert!(unused_gas_limit >= max_tx_gas_limit);
        };
        tokio::spawn(collect_blocks(cancel, provider, assert))
    };

    // Drive concurrency.
    let cfg = Execution::new()
        .add_step(1, 5)
        .add_step(10, 5)
        .add_step(100, 5)
        .add_step(150, 5)
        .add_step(200, 5);
    let testnet = Arc::new(testnet);
    let testnet_clone = testnet.clone();
    let nonce_manager = Arc::new(NonceManager::new());

    let results = concurrency::execute(cfg.clone(), move |test_id: usize, mut bencher: Bencher| {
        let testnet = testnet_clone.clone();
        let nonce_manager = nonce_manager.clone();
        let provider = provider.clone();

        let test = async move {
            let sender = testnet.account_mod_nth(test_id);
            let recipient = testnet.account_mod_nth(test_id + 1);
            println!("running (test_id={})", test_id);

            let middleware = make_middleware(provider, sender, chain_id)
                .await
                .context("make_middleware")?;

            let sender: H160 = sender.eth_addr().into();
            let nonce = nonce_manager.get_and_increment(sender).await;

            // Create the simplest transaction possible: send tokens between accounts.
            let to: H160 = recipient.eth_addr().into();
            let mut tx = Eip1559TransactionRequest::new()
                .to(to)
                .value(1)
                .nonce(nonce);

            let gas_estimation = middleware
                .estimate_gas(&tx.clone().into(), None)
                .await
                .unwrap();
            tx = tx.gas(gas_estimation);
            assert!(gas_estimation <= max_tx_gas_limit);

            bencher.start();

            let pending: PendingTransaction<_> = middleware
                .send_transaction(tx, None)
                .await
                .context("failed to send txn")?;
            let tx_hash = pending.tx_hash();
            println!("sent tx {:?} (test_id={})", tx_hash, test_id);

            // We expect that the transaction is pending, however it should not return an error.
            match middleware.get_transaction(tx_hash).await {
                Ok(Some(_)) => {}
                Ok(None) => bail!("pending transaction not found by eth hash"),
                Err(e) => {
                    bail!("failed to get pending transaction: {e}")
                }
            }
            bencher.mempool();

            loop {
                if let Ok(Some(tx)) = middleware.get_transaction_receipt(tx_hash).await {
                    println!(
                        "tx included in block {:?} (test_id={})",
                        tx.block_number, test_id
                    );
                    let block_number = tx.block_number.unwrap().as_u64();
                    bencher.block_inclusion(block_number);
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }

            Ok(bencher)
        };
        test.boxed()
    })
    .await;

    cancel.send();
    let blocks = blocks_collector.await??;
    let summary = concurrency::ExecutionSummary::new(cfg.clone(), blocks, results);
    summary.print();

    let res = summary.to_result();
    let Ok(testnet) = Arc::try_unwrap(testnet) else {
        bail!("Arc::try_unwrap(testnet)");
    };
    cleanup(res.is_err(), testnet).await;
    res
}
