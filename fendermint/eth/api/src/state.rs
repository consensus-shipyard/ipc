// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tendermint RPC helper methods for the implementation of the APIs.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Context};
use ethers_core::types::{self as et, BlockId};
use fendermint_rpc::client::{FendermintClient, TendermintClient};
use fendermint_rpc::query::QueryClient;
use fendermint_vm_actor_interface::{evm, system};
use fendermint_vm_message::{chain::ChainMessage, conv::from_eth::to_fvm_address};
use futures::StreamExt;
use fvm_ipld_encoding::{de::DeserializeOwned, RawBytes};
use fvm_shared::{chainid::ChainID, econ::TokenAmount, error::ExitCode, message::Message};
use tendermint::block::Height;
use tendermint_rpc::{
    endpoint::{block, block_by_hash, block_results, commit, header, header_by_hash},
    Client,
};
use tendermint_rpc::{Subscription, SubscriptionClient};

use crate::filters::{FilterId, FilterKind, FilterRecords, FilterState};
use crate::{
    conv::from_tm::{
        map_rpc_block_txs, message_hash, to_chain_message, to_eth_block, to_eth_transaction,
    },
    error, JsonRpcResult,
};

type FilterMap = Arc<Mutex<HashMap<FilterId, Arc<Mutex<FilterState>>>>>;

// Made generic in the client type so we can mock it if we want to test API
// methods without having to spin up a server. In those tests the methods
// below would not be used, so those aren't generic; we'd directly invoke
// e.g. `fendermint_eth_api::apis::eth::accounts` with some mock client.
pub struct JsonRpcState<C> {
    pub client: FendermintClient<C>,
    filter_timeout: Duration,
    next_filter_id: AtomicUsize,
    filters: FilterMap,
}

impl<C> JsonRpcState<C> {
    pub fn new(client: C, filter_timeout: Duration) -> Self {
        Self {
            client: FendermintClient::new(client),
            filter_timeout,
            next_filter_id: Default::default(),
            filters: Default::default(),
        }
    }

    /// The underlying Tendermint RPC client.
    pub fn tm(&self) -> &C {
        self.client.underlying()
    }
}

impl<C> JsonRpcState<C>
where
    C: Client + Sync + Send,
{
    /// Get the Tendermint block at a specific height.
    pub async fn block_by_height(
        &self,
        block_number: et::BlockNumber,
    ) -> JsonRpcResult<tendermint::Block> {
        let block = match block_number {
            et::BlockNumber::Number(height) => {
                let height =
                    Height::try_from(height.as_u64()).context("failed to convert to height")?;
                let res: block::Response = self.tm().block(height).await?;
                res.block
            }
            et::BlockNumber::Finalized
            | et::BlockNumber::Latest
            | et::BlockNumber::Safe
            | et::BlockNumber::Pending => {
                // Using 1 block less than `latest_block` so if this is followed up by `block_results`
                // then we don't get an error.
                let commit: commit::Response = self.tm().latest_commit().await?;
                let height = commit.signed_header.header.height.value();
                let height = Height::try_from((height.saturating_sub(1)).max(1))
                    .context("failed to convert to height")?;
                let res: block::Response = self.tm().block(height).await?;
                res.block
            }
            et::BlockNumber::Earliest => {
                let res: block::Response = self.tm().block(Height::from(1u32)).await?;
                res.block
            }
        };
        Ok(block)
    }

    /// Get the Tendermint header at a specific height.
    pub async fn header_by_height(
        &self,
        block_number: et::BlockNumber,
    ) -> JsonRpcResult<tendermint::block::Header> {
        let header = match block_number {
            et::BlockNumber::Number(height) => {
                let height =
                    Height::try_from(height.as_u64()).context("failed to convert to height")?;
                let res: header::Response = self.tm().header(height).await?;
                res.header
            }
            et::BlockNumber::Finalized
            | et::BlockNumber::Latest
            | et::BlockNumber::Safe
            | et::BlockNumber::Pending => {
                let res: commit::Response = self.tm().latest_commit().await?;
                res.signed_header.header
            }
            et::BlockNumber::Earliest => {
                let res: header::Response = self.tm().header(Height::from(1u32)).await?;
                res.header
            }
        };
        Ok(header)
    }

    /// Get the Tendermint header at a specificed height or hash.
    pub async fn header_by_id(
        &self,
        block_id: et::BlockId,
    ) -> JsonRpcResult<tendermint::block::Header> {
        match block_id {
            et::BlockId::Number(n) => self.header_by_height(n).await,
            et::BlockId::Hash(h) => self.header_by_hash(h).await,
        }
    }

    /// Get a Tendermint block by hash, if it exists.
    pub async fn block_by_hash_opt(
        &self,
        block_hash: et::H256,
    ) -> JsonRpcResult<Option<tendermint::block::Block>> {
        let hash = tendermint::Hash::Sha256(*block_hash.as_fixed_bytes());
        let res: block_by_hash::Response = self.tm().block_by_hash(hash).await?;
        Ok(res.block)
    }

    /// Get a Tendermint height by hash, if it exists.
    pub async fn header_by_hash_opt(
        &self,
        block_hash: et::H256,
    ) -> JsonRpcResult<Option<tendermint::block::Header>> {
        let hash = tendermint::Hash::Sha256(*block_hash.as_fixed_bytes());
        let res: header_by_hash::Response = self.tm().header_by_hash(hash).await?;
        Ok(res.header)
    }

    /// Get a Tendermint header by hash.
    pub async fn header_by_hash(
        &self,
        block_hash: et::H256,
    ) -> JsonRpcResult<tendermint::block::Header> {
        match self.header_by_hash_opt(block_hash).await? {
            Some(header) => Ok(header),
            None => error(
                ExitCode::USR_NOT_FOUND,
                format!("block {block_hash} not found"),
            ),
        }
    }

    /// Fetch transaction results to produce the full block.
    pub async fn enrich_block(
        &self,
        block: tendermint::Block,
        full_tx: bool,
    ) -> JsonRpcResult<et::Block<serde_json::Value>>
    where
        C: Client + Sync + Send,
    {
        let height = block.header().height;

        let state_params = self.client.state_params(Some(height)).await?;
        let base_fee = state_params.value.base_fee;
        let chain_id = ChainID::from(state_params.value.chain_id);

        let block_results: block_results::Response = self.tm().block_results(height).await?;

        let block = to_eth_block(block, block_results, base_fee, chain_id)
            .context("failed to convert to eth block")?;

        let block = if full_tx {
            map_rpc_block_txs(block, serde_json::to_value).context("failed to convert to JSON")?
        } else {
            map_rpc_block_txs(block, |h| serde_json::to_value(h.hash))
                .context("failed to convert hash to JSON")?
        };

        Ok(block)
    }

    /// Get a transaction from a block by index.
    pub async fn transaction_by_index(
        &self,
        block: tendermint::Block,
        index: et::U64,
    ) -> JsonRpcResult<Option<et::Transaction>> {
        if let Some(msg) = block.data().get(index.as_usize()) {
            let hash = message_hash(msg)?;
            let msg = to_chain_message(msg)?;

            if let ChainMessage::Signed(msg) = msg {
                let sp = self
                    .client
                    .state_params(Some(block.header().height))
                    .await?;

                let chain_id = ChainID::from(sp.value.chain_id);
                let mut tx = to_eth_transaction(hash, *msg, chain_id)
                    .context("failed to convert to eth transaction")?;
                tx.transaction_index = Some(index);
                tx.block_hash = Some(et::H256::from_slice(block.header.hash().as_bytes()));
                tx.block_number = Some(et::U64::from(block.header.height.value()));
                Ok(Some(tx))
            } else {
                error(ExitCode::USR_ILLEGAL_ARGUMENT, "incompatible transaction")
            }
        } else {
            Ok(None)
        }
    }

    /// Send a message by the system actor to an EVM actor for a read-only query.
    pub async fn read_evm_actor<T>(
        &self,
        address: et::H160,
        method: evm::Method,
        params: RawBytes,
        block_id: BlockId,
    ) -> JsonRpcResult<T>
    where
        T: DeserializeOwned,
    {
        let header = self.header_by_id(block_id).await?;

        // We send off a read-only query to an EVM actor at the given address.
        let message = Message {
            version: Default::default(),
            from: system::SYSTEM_ACTOR_ADDR,
            to: to_fvm_address(address),
            sequence: 0,
            value: TokenAmount::from_atto(0),
            method_num: method as u64,
            params,
            gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
            gas_fee_cap: TokenAmount::from_atto(0),
            gas_premium: TokenAmount::from_atto(0),
        };

        let result = self
            .client
            .call(message, Some(header.height))
            .await
            .context("failed to call contract")?;

        if result.value.code.is_err() {
            return error(ExitCode::new(result.value.code.value()), result.value.info);
        }

        let data = fendermint_rpc::response::decode_bytes(&result.value)
            .context("failed to decode data as bytes")?;

        let data: T = fvm_ipld_encoding::from_slice(&data).context("failed to decode as IPLD")?;

        Ok(data)
    }
}

impl<C> JsonRpcState<C>
where
    C: SubscriptionClient,
{
    /// Create a new filter with the next available ID and insert it into the filters collection.
    fn new_filter_state(&self, filter: &FilterKind) -> (FilterId, Arc<Mutex<FilterState>>) {
        let id = FilterId::from(self.next_filter_id.fetch_add(1, Ordering::Relaxed));
        let state = FilterState::new(id, self.filter_timeout, filter);
        let state = Arc::new(Mutex::new(state));
        let mut filters = self.filters.lock().expect("lock poisoned");
        filters.insert(id, state.clone());
        (id, state)
    }

    /// Create a new filter, subscribe with Tendermint and start handlers in the background.
    pub async fn new_filter(&self, filter: FilterKind) -> anyhow::Result<FilterId> {
        let queries = filter
            .to_queries()
            .context("failed to convert filter to queries")?;

        let mut subs = Vec::new();

        for query in queries {
            let sub: Subscription = self
                .tm()
                .subscribe(query)
                .await
                .context("failed to subscribe to query")?;

            subs.push(sub);
        }

        let (id, state) = self.new_filter_state(&filter);

        for sub in subs {
            spawn_subscription_handler(self.filters.clone(), id, state.clone(), sub);
        }

        Ok(id)
    }
}

impl<C> JsonRpcState<C> {
    pub fn uninstall_filter(&self, filter_id: FilterId) -> bool {
        let removed = {
            let mut filters = self.filters.lock().expect("lock poisoned");
            filters.remove(&filter_id)
        };

        if let Some(filter) = removed {
            // Signal to the background tasks that they can unsubscribe.
            let mut filter = filter.lock().expect("lock poisoned");
            filter.finish(None);
            true
        } else {
            false
        }
    }

    /// Take the currently accumulated changes, and remove the filter if it's finished.
    pub fn take_filter_changes(
        &self,
        filter_id: FilterId,
    ) -> anyhow::Result<Option<FilterRecords>> {
        let mut filters = self.filters.lock().expect("lock poisoned");

        let result = match filters.get(&filter_id) {
            None => Ok(None),
            Some(state) => {
                let mut state = state.lock().expect("lock poisoned");
                state.try_take()
            }
        };

        let keep = match result {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        };

        if !keep {
            // Filter won't produce more data, remove it from the registry.
            filters.remove(&filter_id);
        }

        result
    }
}

/// Spawn a subscription handler in a new task.
fn spawn_subscription_handler(
    filters: FilterMap,
    id: FilterId,
    state: Arc<Mutex<FilterState>>,
    mut sub: Subscription,
) {
    tokio::spawn(async move {
        tracing::debug!(
            ?id,
            query = sub.query().to_string(),
            "polling filter subscription"
        );
        while let Some(result) = sub.next().await {
            tracing::debug!(?id, ?result, "next filter event");
            let mut state = state.lock().expect("lock poisoned");

            if state.is_finished() {
                tracing::debug!(?id, "filter already finished");
                return;
            } else if state.is_timed_out() {
                tracing::warn!(?id, "removing timed out filter");
                // Clean up because the reader won't do it.
                filters.lock().expect("lock poisoned").remove(&id);
                state.finish(Some(anyhow!("filter timeout")));
                return;
            } else {
                match result {
                    Ok(event) => {
                        if let Err(err) = state.update(event) {
                            tracing::error!(?id, "failed to update filter: {err}");
                            state.finish(Some(anyhow!("update failed: {err}")));
                            return;
                        }
                        tracing::debug!(?id, "filter updated");
                    }
                    Err(err) => {
                        tracing::error!(?id, "filter subscription failed: {err}");
                        state.finish(Some(anyhow!("subscription failed: {err}")));
                        return;
                    }
                }
            }
        }
        // Mark the state as finished, but don't remove; let the poller consume whatever is left.
        tracing::debug!(?id, "finishing filter");
        state.lock().expect("lock poisoned").finish(None)

        // Dropping the `Subscription` should cause the client to unsubscribe,
        // if this was the last one interested in that query; we don't have to
        // call the unsubscribe method explicitly.
        // See https://docs.rs/tendermint-rpc/0.31.1/tendermint_rpc/client/struct.WebSocketClient.html
    });
}
