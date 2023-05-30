// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// See the following for inspiration:
// * https://github.com/evmos/ethermint/blob/ebbe0ffd0d474abd745254dc01e60273ea758dae/rpc/namespaces/ethereum/eth/api.go#L44
// * https://github.com/filecoin-project/lotus/blob/v1.23.1-rc2/api/api_full.go#L783

use ethers_core::types as ethtypes;
use jsonrpc_v2::Params;
use tendermint::block::Height;
use tendermint_rpc::{endpoint::block, Client};

use crate::{JsonRpcData, JsonRpcResult};

/// Returns a list of addresses owned by client.
///
/// It will always return [] since we don't expect Fendermint to manage private keys.
pub async fn accounts<C>(_data: JsonRpcData<C>) -> JsonRpcResult<Vec<ethtypes::Address>> {
    Ok(vec![])
}

/// Returns the number of most recent block.
pub async fn block_number<C>(data: JsonRpcData<C>) -> JsonRpcResult<ethtypes::U64>
where
    C: Client + Sync,
{
    let res: block::Response = data.client.latest_block().await?;
    let height = res.block.header.height;
    Ok(ethtypes::U64::from(height.value()))
}

/// Returns the number of transactions in a block matching the given block number.
///
/// QUANTITY|TAG - integer of a block number, or the string "earliest", "latest" or "pending", as in the default block parameter.
pub async fn get_block_transaction_count_by_number<C: Client>(
    data: JsonRpcData<C>,
    Params(params): Params<ethtypes::BlockNumber>,
) -> JsonRpcResult<ethtypes::U64>
where
    C: Client + Sync,
{
    let block = match params {
        ethtypes::BlockNumber::Number(height) => {
            let height = Height::try_from(height.as_u64())?;
            let res: block::Response = data.client.block(height).await?;
            res.block
        }
        ethtypes::BlockNumber::Finalized
        | ethtypes::BlockNumber::Latest
        | ethtypes::BlockNumber::Safe
        | ethtypes::BlockNumber::Pending => {
            let res: block::Response = data.client.latest_block().await?;
            res.block
        }
        ethtypes::BlockNumber::Earliest => {
            let res: block::Response = data.client.block(Height::from(1u32)).await?;
            res.block
        }
    };

    Ok(ethtypes::U64::from(block.data.len()))
}
