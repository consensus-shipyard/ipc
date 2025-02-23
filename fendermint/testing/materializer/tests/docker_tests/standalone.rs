// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::make_testnet;
use anyhow::{bail, Context};
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::{
    providers::{Middleware, PendingTransaction},
    signers::{Signer},
    types::{Eip1559TransactionRequest, H160},
};
use fendermint_materializer::{manifest::Rootnet, HasEthApi};
use std::time::{Duration, Instant};
use crate::docker_tests::make_middleware;

const MANIFEST: &str = "standalone.yaml";

/// Test that a transaction sent to the mempool can be retrieved by its ethereum hash
/// from the ethereum API instance it was sent to even before it is included in the block.
#[serial_test::serial]
#[tokio::test]
async fn test_sent_tx_found_in_mempool() -> Result<(), anyhow::Error> {
    let (testnet, cleanup) = make_testnet(MANIFEST, |manifest| {
        // Slow down consensus to where we can see the effect of the transaction not being found by Ethereum hash.
        if let Rootnet::New { ref mut env, .. } = manifest.rootnet {
            env.insert("CMT_CONSENSUS_TIMEOUT_COMMIT".into(), "10s".into());
        };
    })
    .await?;

    let res = {
        let bob = testnet.account("bob")?;
        let charlie = testnet.account("charlie")?;

        let pangea = testnet.node(&testnet.root().node("pangea"))?;
        let provider = pangea
            .ethapi_http_provider()?
            .expect("ethapi should be enabled");

        let middleware = make_middleware(provider, bob, None)
            .await
            .context("failed to set up middleware")?;

        eprintln!("middleware ready, pending tests");

        // Create the simplest transaction possible: send tokens between accounts.
        let to: H160 = charlie.eth_addr().into();
        let transfer = Eip1559TransactionRequest::new().to(to).value(1);

        let pending: PendingTransaction<_> = middleware
            .send_transaction(transfer, None)
            .await
            .context("failed to send txn")?;

        let tx_hash = pending.tx_hash();

        eprintln!("sent pending txn {:?}", tx_hash);

        // We expect that the transaction is pending, however it should not return an error.
        match middleware.get_transaction(tx_hash).await {
            Ok(Some(_)) => {}
            Ok(None) => bail!("pending transaction not found by eth hash"),
            Err(e) => {
                bail!("failed to get pending transaction: {e}")
            }
        }
        Ok(())
    };

    cleanup(res.is_err(), testnet).await;
    res
}

/// Test that transactions sent out-of-order with regards to the nonce are not rejected,
/// but rather get included in block eventually, their submission managed by the ethereum
/// API facade.
#[serial_test::serial]
#[tokio::test]
async fn test_out_of_order_mempool() {
    const MAX_WAIT_TIME: Duration = Duration::from_secs(10);
    const SLEEP_TIME: Duration = Duration::from_secs(1);

    let (testnet, cleanup) = make_testnet(MANIFEST, |_| {}).await.unwrap();

    let res = async {
        let bob = testnet.account("bob")?;
        let charlie = testnet.account("charlie")?;

        let pangea = testnet.node(&testnet.root().node("pangea"))?;
        let provider = pangea
            .ethapi_http_provider()?
            .expect("ethapi should be enabled");

        let middleware = make_middleware(provider, bob, None)
            .await
            .context("failed to set up middleware")?;

        // Create the simplest transaction possible: send tokens between accounts.
        let to: H160 = charlie.eth_addr().into();
        let tx = Eip1559TransactionRequest::new().to(to).value(1);
        let mut tx: TypedTransaction = tx.into();

        // Fill out the nonce, gas, etc.
        middleware
            .fill_transaction(&mut tx, None)
            .await
            .context("failed to fill tx")?;

        // Create a few more transactions to be sent out-of-order.
        let mut txs = vec![tx];

        for i in 1..5 {
            let mut tx = txs[0].clone();
            let nonce = tx.nonce().expect("fill_transaction filled the nonce");
            tx.set_nonce(nonce.saturating_add(i.into()));
            txs.push(tx)
        }

        let mut pending_txs = Vec::new();

        // Submit transactions in opposite order.
        for (i, tx) in txs.iter().enumerate().rev() {
            let sig = middleware
                .signer()
                .sign_transaction(tx)
                .await
                .context("failed to sign tx")?;

            let rlp = tx.rlp_signed(&sig);

            let pending_tx: PendingTransaction<_> = middleware
                .send_raw_transaction(rlp)
                .await
                .with_context(|| format!("failed to send tx {i}"))?;

            pending_txs.push(pending_tx)
        }

        // Check that they eventually get included.
        let start = Instant::now();
        'pending: loop {
            for tx in pending_txs.iter() {
                let receipt = middleware
                    .get_transaction_receipt(tx.tx_hash())
                    .await
                    .context("failed to get receipt")?;

                if receipt.is_none() {
                    if start.elapsed() > MAX_WAIT_TIME {
                        bail!("some transactions are still not executed")
                    } else {
                        tokio::time::sleep(SLEEP_TIME).await;
                        continue 'pending;
                    }
                }
            }
            // All of them have receipt.
            break 'pending;
        }

        Ok(())
    };

    let res = res.await;
    cleanup(res.is_err(), testnet).await;
    res.unwrap()
}
