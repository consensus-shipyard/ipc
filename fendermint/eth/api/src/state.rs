// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tendermint RPC helper methods for the implementation of the APIs.

use ethers_core::types::{self as ethtypes};
use fendermint_rpc::client::TendermintClient;
use fvm_shared::error::ExitCode;
use jsonrpc_v2::ErrorLike;
use tendermint::block::Height;
use tendermint_rpc::{
    endpoint::{block, block_by_hash, commit, header, header_by_hash},
    Client,
};

use crate::{JsonRpcResult, JsonRpcState};

impl<C> JsonRpcState<C>
where
    C: Client + Sync,
{
    /// The underlying Tendermint RPC client.
    pub fn tm(&self) -> &C {
        self.client.underlying()
    }

    /// Get the Tendermint block at a specific height.
    pub async fn block_by_height(
        &self,
        block_number: ethtypes::BlockNumber,
    ) -> JsonRpcResult<tendermint::Block> {
        let block = match block_number {
            ethtypes::BlockNumber::Number(height) => {
                let height = Height::try_from(height.as_u64())?;
                let res: block::Response = self.tm().block(height).await?;
                res.block
            }
            ethtypes::BlockNumber::Finalized
            | ethtypes::BlockNumber::Latest
            | ethtypes::BlockNumber::Safe
            | ethtypes::BlockNumber::Pending => {
                let res: block::Response = self.tm().latest_block().await?;
                res.block
            }
            ethtypes::BlockNumber::Earliest => {
                let res: block::Response = self.tm().block(Height::from(1u32)).await?;
                res.block
            }
        };
        Ok(block)
    }

    /// Get the Tendermint header at a specific height.
    pub async fn header_by_height(
        &self,
        block_number: ethtypes::BlockNumber,
    ) -> JsonRpcResult<tendermint::block::Header> {
        let header = match block_number {
            ethtypes::BlockNumber::Number(height) => {
                let height = Height::try_from(height.as_u64())?;
                let res: header::Response = self.tm().header(height).await?;
                res.header
            }
            ethtypes::BlockNumber::Finalized
            | ethtypes::BlockNumber::Latest
            | ethtypes::BlockNumber::Safe
            | ethtypes::BlockNumber::Pending => {
                let res: commit::Response = self.tm().latest_commit().await?;
                res.signed_header.header
            }
            ethtypes::BlockNumber::Earliest => {
                let res: header::Response = self.tm().header(Height::from(1u32)).await?;
                res.header
            }
        };
        Ok(header)
    }

    /// Get a Tendermint block by hash, if it exists.
    pub async fn block_by_hash_opt(
        &self,
        block_hash: ethtypes::H256,
    ) -> JsonRpcResult<Option<tendermint::block::Block>> {
        let hash = tendermint::Hash::Sha256(*block_hash.as_fixed_bytes());
        let res: block_by_hash::Response = self.tm().block_by_hash(hash).await?;
        Ok(res.block)
    }

    /// Get a Tendermint height by hash, if it exists.
    pub async fn header_by_hash_opt(
        &self,
        block_hash: ethtypes::H256,
    ) -> JsonRpcResult<Option<tendermint::block::Header>> {
        let hash = tendermint::Hash::Sha256(*block_hash.as_fixed_bytes());
        let res: header_by_hash::Response = self.tm().header_by_hash(hash).await?;
        Ok(res.header)
    }

    /// Get a Tendermint header by hash.
    pub async fn header_by_hash(
        &self,
        block_hash: ethtypes::H256,
    ) -> JsonRpcResult<tendermint::block::Header> {
        match self.header_by_hash_opt(block_hash).await? {
            Some(header) => Ok(header),
            None => Err(jsonrpc_v2::Error::Full {
                code: ExitCode::USR_NOT_FOUND.code(),
                message: format!("block {block_hash} not found"),
                data: None,
            }),
        }
    }
}
