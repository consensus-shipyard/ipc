// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Utilities related to caching and buffering Ethereum transactions.
use std::time::Duration;

use ethers_core::types as et;
use fendermint_rpc::{client::TendermintClient, FendermintClient, QueryClient};
use fendermint_vm_message::{chain::ChainMessage, query::FvmQueryHeight, signed::DomainHash};
use futures::StreamExt;
use fvm_shared::chainid::ChainID;
use tendermint::Block;
use tendermint_rpc::{
    event::EventData,
    query::{EventType, Query},
    SubscriptionClient,
};

use crate::{cache::Cache, HybridClient};

const RETRY_SLEEP_SECS: u64 = 5;

/// Cache submitted transactions by their Ethereum hash, because the CometBFT
/// API would not be able to find them until they are delivered to the application
/// and indexed by their domain hash, which some tools interpret as the transaction
/// being dropped from the mempool.
pub type TransactionCache = Cache<et::TxHash, et::Transaction>;

pub fn start_tx_cache_clearing(client: FendermintClient<HybridClient>, cache: TransactionCache) {
    tokio::task::spawn(async move {
        let chain_id = get_chain_id(&client).await;
        tx_cache_clearing_loop(client, cache, chain_id).await;
    });
}

async fn tx_cache_clearing_loop(
    client: FendermintClient<HybridClient>,
    tx_cache: TransactionCache,
    chain_id: ChainID,
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
                                let tx_hashes = collect_tx_hashes(&block, &chain_id);
                                tx_cache.remove_many(&tx_hashes);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn collect_tx_hashes(block: &Block, chain_id: &ChainID) -> Vec<et::TxHash> {
    let mut tx_hashes = Vec::new();
    for tx in &block.data {
        if let Ok(ChainMessage::Signed(msg)) = fvm_ipld_encoding::from_slice(tx) {
            if let Ok(Some(DomainHash::Eth(h))) = msg.domain_hash(chain_id) {
                tx_hashes.push(et::TxHash::from(h))
            }
        }
    }
    tx_hashes
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
