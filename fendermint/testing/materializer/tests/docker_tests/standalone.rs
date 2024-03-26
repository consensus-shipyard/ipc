// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{bail, Context};
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{JsonRpcClient, Middleware, PendingTransaction, Provider},
    signers::{Signer, Wallet},
    types::{Eip1559TransactionRequest, H160},
};
use fendermint_materializer::{manifest::Rootnet, materials::DefaultAccount, HasEthApi};
use futures::FutureExt;

use crate::with_testnet;

const MANIFEST: &str = "standalone.yaml";

pub type TestMiddleware<C> = SignerMiddleware<Provider<C>, Wallet<SigningKey>>;

/// Create a middleware that will assign nonces and sign the message.
async fn make_middleware<C>(
    provider: Provider<C>,
    sender: &DefaultAccount,
) -> anyhow::Result<TestMiddleware<C>>
where
    C: JsonRpcClient,
{
    let chain_id = provider
        .get_chainid()
        .await
        .context("failed to get chain ID")?;

    let wallet: Wallet<SigningKey> = Wallet::from_bytes(&sender.secret_key().serialize().as_ref())?
        .with_chain_id(chain_id.as_u64());

    Ok(SignerMiddleware::new(provider, wallet))
}

/// Test that top-down syncing and bottom-up checkpoint submission work.
#[serial_test::serial]
#[tokio::test]
async fn test_sent_tx_found_in_mempool() {
    with_testnet(
        MANIFEST,
        |manifest| {
            // Slow down consensus to where we can see the effect of the transaction not being found by Ethereum hash.
            if let Rootnet::New { ref mut env, .. } = manifest.rootnet {
                env.insert("CMT_CONSENSUS_TIMEOUT_COMMIT".into(), "10s".into());
            };
        },
        |_, _, testnet| {
            let test = async {
                let bob = testnet.account("bob")?;
                let charlie = testnet.account("charlie")?;

                let pangea = testnet.node(&testnet.root().node("pangea"))?;
                let provider = pangea
                    .ethapi_http_provider()?
                    .expect("ethapi should be enabled");

                let middleware = make_middleware(provider, bob)
                    .await
                    .context("failed to set up middleware")?;

                // Create the simplest transaction possible: send tokens between accounts.
                let to: H160 = charlie.eth_addr().into();
                let transfer = Eip1559TransactionRequest::new().to(to).value(1);

                let pending: PendingTransaction<_> = middleware
                    .send_transaction(transfer, None)
                    .await
                    .context("failed to send txn")?;

                let tx_hash = pending.tx_hash();

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

            test.boxed_local()
        },
    )
    .await
    .unwrap()
}
