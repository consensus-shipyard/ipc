// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tendermint RPC helper methods for the implementation of the APIs.

use std::time::Duration;

use anyhow::{anyhow, Context};
use ethers_core::types::{self as et, BlockId};
use fendermint_rpc::client::{FendermintClient, TendermintClient};
use fendermint_rpc::query::QueryClient;
use fendermint_vm_actor_interface::{evm, system};
use fendermint_vm_message::{chain::ChainMessage, conv::from_eth::to_fvm_address};
use fvm_ipld_encoding::{de::DeserializeOwned, RawBytes};
use fvm_shared::{chainid::ChainID, econ::TokenAmount, error::ExitCode, message::Message};
use rand::Rng;
use tendermint::block::Height;
use tendermint_rpc::{
    endpoint::{block, block_by_hash, block_results, commit, header, header_by_hash},
    Client,
};
use tendermint_rpc::{Subscription, SubscriptionClient};
use tokio::sync::mpsc::Sender;

use crate::filters::{
    run_subscription, FilterCommand, FilterId, FilterKind, FilterMap, FilterRecords, FilterState,
};
use crate::{
    conv::from_tm::{
        map_rpc_block_txs, message_hash, to_chain_message, to_eth_block, to_eth_transaction,
    },
    error, JsonRpcResult,
};

// Made generic in the client type so we can mock it if we want to test API
// methods without having to spin up a server. In those tests the methods
// below would not be used, so those aren't generic; we'd directly invoke
// e.g. `fendermint_eth_api::apis::eth::accounts` with some mock client.
pub struct JsonRpcState<C> {
    pub client: FendermintClient<C>,
    filter_timeout: Duration,
    filters: FilterMap,
}

impl<C> JsonRpcState<C> {
    pub fn new(client: C, filter_timeout: Duration) -> Self {
        Self {
            client: FendermintClient::new(client),
            filter_timeout,
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
    async fn insert_filter(&self, filter: &FilterKind) -> (FilterState, Sender<FilterCommand>) {
        let mut filters = self.filters.write().await;

        // Choose an unpredictable filter, so it's not so easy to clear out someone else's logs.
        let mut id: et::U256;
        loop {
            id = FilterId::from(rand::thread_rng().gen::<u64>());
            if !filters.contains_key(&id) {
                break;
            }
        }

        let (state, tx) = FilterState::new(id, self.filter_timeout, filter);

        // Inserting happens here, while removal will be handled by the `FilterState` itself.
        filters.insert(id, tx.clone());

        (state, tx)
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

        let (state, tx) = self.insert_filter(&filter).await;
        let id = state.id();
        let filters = self.filters.clone();

        tokio::spawn(async move { state.run(filters).await });

        for sub in subs {
            let tx = tx.clone();
            tokio::spawn(async move { run_subscription(id, sub, tx).await });
        }

        Ok(id)
    }
}

impl<C> JsonRpcState<C> {
    pub async fn uninstall_filter(&self, filter_id: FilterId) -> anyhow::Result<bool> {
        let filters = self.filters.read().await;

        if let Some(tx) = filters.get(&filter_id) {
            // Signal to the background tasks that they can unsubscribe.
            tx.send(FilterCommand::Uninstall)
                .await
                .map_err(|e| anyhow!("failed to send command: {e}"))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Take the currently accumulated changes.
    pub async fn take_filter_changes(
        &self,
        filter_id: FilterId,
    ) -> anyhow::Result<Option<FilterRecords>> {
        let filters = self.filters.read().await;

        match filters.get(&filter_id) {
            None => Ok(None),
            Some(tx) => {
                let (tx_res, rx_res) = tokio::sync::oneshot::channel();

                tx.send(FilterCommand::Take(tx_res))
                    .await
                    .map_err(|e| anyhow!("failed to send command: {e}"))?;

                rx_res.await.context("failed to receive response")?
            }
        }
    }
}
