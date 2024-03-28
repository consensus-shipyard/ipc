// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Utilities related to caching and buffering Ethereum transactions.
use std::{collections::BTreeMap, time::Duration};

use ethers_core::types as et;
use fendermint_rpc::{client::TendermintClient, FendermintClient, QueryClient};
use fendermint_vm_message::{chain::ChainMessage, query::FvmQueryHeight, signed::DomainHash};
use futures::StreamExt;
use fvm_shared::{address::Address, chainid::ChainID};
use tendermint::Block;
use tendermint_rpc::{
    event::EventData,
    query::{EventType, Query},
    SubscriptionClient,
};

use crate::{cache::Cache, state::Nonce, HybridClient};

const RETRY_SLEEP_SECS: u64 = 5;

/// Cache submitted transactions by their Ethereum hash, because the CometBFT
/// API would not be able to find them until they are delivered to the application
/// and indexed by their domain hash, which some tools interpret as the transaction
/// being dropped from the mempool.
pub type TransactionCache = Cache<et::TxHash, et::Transaction>;

/// Buffer out-of-order messages until they can be sent to the chain.
#[derive(Clone)]
pub struct TransactionBuffer(pub Cache<Address, BTreeMap<Nonce, ChainMessage>>);

impl TransactionBuffer {
    /// Remove all (sender, nonce) pairs which were included in a block.
    ///
    /// Also remove and return all transactions that can now be submitted in turn.
    pub fn remove_many<'a, I>(&self, txs: I)
    where
        I: Iterator<Item = (&'a Address, Nonce)>,
    {
        self.0.with(|c| {
            for (sender, nonce) in txs {
                if let Some(buffer) = c.get_mut(sender) {
                    buffer.remove(&nonce);
                    // TODO: Check if there is a nonce that _follows_ this one which can be submitted now.
                }
            }
        })
    }

    /// Insert a transaction we could not submit straight away into the buffer.
    pub fn insert(&self, sender: Address, nonce: Nonce, msg: ChainMessage) {
        self.0.with(|c| {
            let buffer = c.entry(sender).or_insert_with(BTreeMap::new);
            // Overwrite any previous entry to protect against DoS attack; it wouldn't make sense to submit them anyway.
            buffer.insert(nonce, msg);
        })
    }
}

/// Subscribe to `NewBlock  notifications and clear transactions from the caches.`
pub fn start_tx_cache_clearing(
    client: FendermintClient<HybridClient>,
    tx_cache: TransactionCache,
    tx_buffer: TransactionBuffer,
) {
    tokio::task::spawn(async move {
        let chain_id = get_chain_id(&client).await;
        tx_cache_clearing_loop(client, chain_id, tx_cache, tx_buffer).await;
    });
}

async fn tx_cache_clearing_loop(
    client: FendermintClient<HybridClient>,
    chain_id: ChainID,
    tx_cache: TransactionCache,
    tx_buffer: TransactionBuffer,
) {
    loop {
        let query = Query::from(EventType::NewBlock);

        match client.underlying().subscribe(query).await {
            Err(e) => {
                tracing::warn!(error=?e, "failed to subscribe to NewBlocks; retrying later...");
                tokio::time::sleep(Duration::from_secs(RETRY_SLEEP_SECS)).await;
            }
            Ok(mut subscription) => {
                while let Some(result) = subscription.next().await {
                    match result {
                        Err(e) => {
                            tracing::warn!(error=?e, "NewBlocks subscription failed; resubscribing...");
                            break;
                        }
                        Ok(event) => {
                            if let EventData::NewBlock {
                                block: Some(block), ..
                            } = event.data
                            {
                                let txs = collect_txs(&block, &chain_id);

                                if txs.is_empty() {
                                    continue;
                                }

                                tx_cache.remove_many(txs.iter().map(|(h, _, _)| h));
                                tx_buffer.remove_many(txs.iter().map(|(_, s, n)| (s, *n)));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Collect the identifiers of the transactions in the block.
fn collect_txs(block: &Block, chain_id: &ChainID) -> Vec<(et::TxHash, Address, Nonce)> {
    let mut txs = Vec::new();
    for tx in &block.data {
        if let Ok(ChainMessage::Signed(msg)) = fvm_ipld_encoding::from_slice(tx) {
            if let Ok(Some(DomainHash::Eth(h))) = msg.domain_hash(chain_id) {
                txs.push((et::TxHash::from(h), msg.message.from, msg.message.sequence))
            }
        }
    }
    txs
}

async fn get_chain_id(client: &FendermintClient<HybridClient>) -> ChainID {
    loop {
        match client.state_params(FvmQueryHeight::default()).await {
            Ok(sp) => {
                return ChainID::from(sp.value.chain_id);
            }
            Err(e) => {
                tracing::warn!(error=?e, "failed to get chain ID; retrying later...");
                tokio::time::sleep(Duration::from_secs(RETRY_SLEEP_SECS)).await;
            }
        }
    }
}
