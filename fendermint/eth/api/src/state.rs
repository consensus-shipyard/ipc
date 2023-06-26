// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tendermint RPC helper methods for the implementation of the APIs.

use anyhow::Context;
use ethers_core::types::{self as et};
use fendermint_rpc::client::TendermintClient;
use fendermint_rpc::query::QueryClient;
use fendermint_vm_message::chain::ChainMessage;
use fvm_shared::{chainid::ChainID, error::ExitCode};
use tendermint::block::Height;
use tendermint_rpc::{
    endpoint::{block, block_by_hash, block_results, commit, header, header_by_hash},
    Client,
};

use crate::{
    conv::from_tm::{
        map_rpc_block_txs, message_hash, to_chain_message, to_eth_block, to_eth_transaction,
    },
    error, JsonRpcResult, JsonRpcState,
};

impl<C> JsonRpcState<C>
where
    C: Client + Sync + Send,
{
    /// The underlying Tendermint RPC client.
    pub fn tm(&self) -> &C {
        self.client.underlying()
    }

    /// Get the Tendermint block at a specific height.
    pub async fn block_by_height(
        &self,
        block_number: et::BlockNumber,
    ) -> JsonRpcResult<tendermint::Block> {
        let block = match block_number {
            et::BlockNumber::Number(height) => {
                let height =
                    Height::try_from(height.as_u64()).context("failed to conver to height")?;
                let res: block::Response = self.tm().block(height).await?;
                res.block
            }
            et::BlockNumber::Finalized
            | et::BlockNumber::Latest
            | et::BlockNumber::Safe
            | et::BlockNumber::Pending => {
                let res: block::Response = self.tm().latest_block().await?;
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
}
