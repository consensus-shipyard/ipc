// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// See the following for inspiration:
// * https://github.com/evmos/ethermint/blob/ebbe0ffd0d474abd745254dc01e60273ea758dae/rpc/namespaces/ethereum/eth/api.go#L44
// * https://github.com/filecoin-project/lotus/blob/v1.23.1-rc2/api/api_full.go#L783
// * https://github.com/filecoin-project/lotus/blob/v1.23.1-rc2/node/impl/full/eth.go

use ethers_core::types::{self as et, BlockId};
use fendermint_rpc::query::QueryClient;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_message::chain::ChainMessage;
use fvm_shared::{address::Address, chainid::ChainID, error::ExitCode};
use jsonrpc_v2::{ErrorLike, Params};
use tendermint_rpc::{
    endpoint::{block, block_results, consensus_params, header},
    Client,
};

use crate::{
    conv::{self, tokens_to_u256},
    JsonRpcData, JsonRpcResult,
};

const MAX_FEE_HIST_SIZE: usize = 1024;

fn error<T>(exit_code: ExitCode, msg: impl ToString) -> JsonRpcResult<T> {
    Err(jsonrpc_v2::Error::Full {
        code: exit_code.code(),
        message: msg.to_string(),
        data: None,
    })
}

/// Returns a list of addresses owned by client.
///
/// It will always return [] since we don't expect Fendermint to manage private keys.
pub async fn accounts<C>(_data: JsonRpcData<C>) -> JsonRpcResult<Vec<et::Address>> {
    Ok(vec![])
}

/// Returns the number of most recent block.
pub async fn block_number<C>(data: JsonRpcData<C>) -> JsonRpcResult<et::U64>
where
    C: Client + Sync,
{
    let res: block::Response = data.tm().latest_block().await?;
    let height = res.block.header.height;
    Ok(et::U64::from(height.value()))
}

/// Returns the chain ID used for signing replay-protected transactions.
pub async fn chain_id<C>(data: JsonRpcData<C>) -> JsonRpcResult<et::U64>
where
    C: Client + Sync + Send,
{
    let res = data.client.state_params(None).await?;
    Ok(et::U64::from(res.value.chain_id))
}

/// Returns transaction base fee per gas and effective priority fee per gas for the requested/supported block range.
pub async fn fee_history<C>(
    data: JsonRpcData<C>,
    Params((block_count, last_block, reward_percentiles)): Params<(
        et::U256,
        et::BlockNumber,
        Vec<f64>,
    )>,
) -> JsonRpcResult<et::FeeHistory>
where
    C: Client + Sync + Send,
{
    if block_count > et::U256::from(MAX_FEE_HIST_SIZE) {
        return error(
            ExitCode::USR_ILLEGAL_ARGUMENT,
            "block_count must be <= 1024",
        );
    }

    let mut hist = et::FeeHistory {
        base_fee_per_gas: Vec::new(),
        gas_used_ratio: Vec::new(),
        oldest_block: et::U256::default(),
        reward: Vec::new(),
    };
    let mut block_number = last_block;
    let mut block_count = block_count.as_usize();

    while block_count > 0 {
        let block = data.block_by_height(block_number).await?;
        let height = block.header().height;

        // Genesis has height 1, but no relevant fees.
        if height.value() <= 1 {
            break;
        }

        hist.oldest_block = et::U256::from(height.value());

        let state_params = data.client.state_params(Some(height)).await?;
        let base_fee = &state_params.value.base_fee;

        hist.base_fee_per_gas.push(tokens_to_u256(base_fee)?);

        let consensus_params: consensus_params::Response =
            data.tm().consensus_params(height).await?;

        let block_results: block_results::Response = data.tm().block_results(height).await?;

        let mut block_gas_limit = consensus_params.consensus_params.block.max_gas;
        if block_gas_limit <= 0 {
            block_gas_limit =
                i64::try_from(fvm_shared::BLOCK_GAS_LIMIT).expect("FVM block gas limit not i64")
        };

        let txs_results = block_results.txs_results.unwrap_or_default();

        let total_gas_used: i64 = txs_results.iter().map(|r| r.gas_used).sum();

        hist.gas_used_ratio
            .push(total_gas_used as f64 / block_gas_limit as f64);

        let mut premiums = Vec::new();
        for (tx, txres) in block.data().iter().zip(txs_results) {
            let msg = fvm_ipld_encoding::from_slice::<ChainMessage>(tx)?;
            if let ChainMessage::Signed(msg) = msg {
                let premium = crate::gas::effective_gas_premium(&msg.message, base_fee);
                premiums.push((premium, txres.gas_used));
            }
        }
        premiums.sort();

        let premium_gas_used: i64 = premiums.iter().map(|(_, gas)| *gas).sum();

        let rewards: Result<Vec<et::U256>, _> = reward_percentiles
            .iter()
            .map(|p| {
                if premiums.is_empty() {
                    Ok(et::U256::zero())
                } else {
                    let threshold_gas_used = (premium_gas_used as f64 * p / 100f64) as i64;
                    let mut sum_gas_used = 0;
                    let mut idx = 0;
                    while sum_gas_used < threshold_gas_used && idx < premiums.len() - 1 {
                        sum_gas_used += premiums[idx].1;
                        idx += 1;
                    }
                    tokens_to_u256(&premiums[idx].0)
                }
            })
            .collect();

        hist.reward.push(rewards?);

        block_count -= 1;
        block_number = et::BlockNumber::Number(et::U64::from(height.value()));
    }

    // Reverse data to be oldest-to-newest.
    hist.base_fee_per_gas.reverse();
    hist.gas_used_ratio.reverse();
    hist.reward.reverse();

    Ok(hist)
}

/// Returns the current price per gas in wei.
pub async fn gas_price<C>(data: JsonRpcData<C>) -> JsonRpcResult<et::U256>
where
    C: Client + Sync + Send,
{
    let res = data.client.state_params(None).await?;
    let price = conv::tokens_to_u256(&res.value.base_fee)?;
    Ok(price)
}

/// Returns the balance of the account of given address.
pub async fn get_balance<C>(
    data: JsonRpcData<C>,
    Params((addr, block_id)): Params<(et::Address, et::BlockId)>,
) -> JsonRpcResult<et::U256>
where
    C: Client + Sync + Send,
{
    let header = match block_id {
        BlockId::Number(n) => data.header_by_height(n).await?,
        BlockId::Hash(h) => data.header_by_hash(h).await?,
    };
    let height = header.height;
    let addr = h160_to_fvm_addr(addr);
    let res = data.client.actor_state(&addr, Some(height)).await?;

    match res.value {
        Some((_, state)) => Ok(conv::tokens_to_u256(&state.balance)?),
        None => error(ExitCode::USR_NOT_FOUND, format!("actor {addr} not found")),
    }
}

/// Returns information about a block by hash.
pub async fn get_block_by_hash<C>(
    data: JsonRpcData<C>,
    Params((block_hash, full_tx)): Params<(et::H256, bool)>,
) -> JsonRpcResult<Option<et::Block<serde_json::Value>>>
where
    C: Client + Sync + Send,
{
    match data.block_by_hash_opt(block_hash).await? {
        Some(block) => enrich_block(data, block, full_tx).await.map(Some),
        None => Ok(None),
    }
}

/// Returns information about a block by block number.
pub async fn get_block_by_number<C>(
    data: JsonRpcData<C>,
    Params((block_number, full_tx)): Params<(et::BlockNumber, bool)>,
) -> JsonRpcResult<Option<et::Block<serde_json::Value>>>
where
    C: Client + Sync + Send,
{
    match data.block_by_height(block_number).await? {
        block if block.header().height.value() > 0 => {
            enrich_block(data, block, full_tx).await.map(Some)
        }
        _ => Ok(None),
    }
}

/// Returns the number of transactions in a block matching the given block number.
pub async fn get_block_transaction_count_by_number<C>(
    data: JsonRpcData<C>,
    Params((block_number,)): Params<(et::BlockNumber,)>,
) -> JsonRpcResult<et::U64>
where
    C: Client + Sync,
{
    let block = data.block_by_height(block_number).await?;

    Ok(et::U64::from(block.data.len()))
}

/// Returns the number of transactions in a block from a block matching the given block hash.
pub async fn get_block_transaction_count_by_hash<C>(
    data: JsonRpcData<C>,
    Params((block_hash,)): Params<(et::H256,)>,
) -> JsonRpcResult<et::U64>
where
    C: Client + Sync,
{
    let block = data.block_by_hash_opt(block_hash).await?;
    let count = block
        .map(|b| et::U64::from(b.data.len()))
        .unwrap_or_default();
    Ok(count)
}

/// Returns the information about a transaction requested by transaction hash.
pub async fn get_transaction_by_block_hash_and_index<C>(
    data: JsonRpcData<C>,
    Params((block_hash, index)): Params<(et::H256, et::U64)>,
) -> JsonRpcResult<Option<et::Transaction>>
where
    C: Client + Sync + Send,
{
    if let Some(block) = data.block_by_hash_opt(block_hash).await? {
        transaction_by_index(data, block, index).await
    } else {
        Ok(None)
    }
}

/// Returns the information about a transaction requested by transaction hash.
pub async fn get_transaction_by_block_number_and_index<C>(
    data: JsonRpcData<C>,
    Params((block_number, index)): Params<(et::BlockNumber, et::U64)>,
) -> JsonRpcResult<Option<et::Transaction>>
where
    C: Client + Sync + Send,
{
    let block = data.block_by_height(block_number).await?;
    transaction_by_index(data, block, index).await
}

/// Returns the information about a transaction requested by transaction hash.
pub async fn get_transaction_by_hash<C>(
    data: JsonRpcData<C>,
    Params((tx_hash,)): Params<(et::H256,)>,
) -> JsonRpcResult<Option<et::Transaction>>
where
    C: Client + Sync + Send,
{
    let hash = tendermint::Hash::try_from(tx_hash.as_bytes().to_vec())?;
    match data.tm().tx(hash, false).await {
        Ok(res) => {
            let msg = fvm_ipld_encoding::from_slice::<ChainMessage>(&res.tx)?;
            if let ChainMessage::Signed(msg) = msg {
                let header: header::Response = data.tm().header(res.height).await?;
                let sp = data.client.state_params(Some(res.height)).await?;
                let chain_id = ChainID::from(sp.value.chain_id);
                let mut tx = conv::to_rpc_transaction(hash, *msg, chain_id)?;
                tx.transaction_index = Some(et::U64::from(res.index));
                tx.block_hash = Some(et::H256::from_slice(header.header.hash().as_bytes()));
                tx.block_number = Some(et::U64::from(res.height.value()));
                Ok(Some(tx))
            } else {
                error(ExitCode::USR_ILLEGAL_ARGUMENT, "incompatible transaction")
            }
        }
        Err(e) if e.to_string().contains("not found") => Ok(None),
        Err(e) => error(ExitCode::USR_UNSPECIFIED, e),
    }
}

/// Returns the number of transactions sent from an address, up to a specific block.
///
/// This is done by looking up the nonce of the account.
pub async fn get_transaction_count<C>(
    data: JsonRpcData<C>,
    Params((addr, block_id)): Params<(et::Address, et::BlockId)>,
) -> JsonRpcResult<et::U64>
where
    C: Client + Sync + Send,
{
    let header = match block_id {
        BlockId::Number(n) => data.header_by_height(n).await?,
        BlockId::Hash(h) => data.header_by_hash(h).await?,
    };
    let height = header.height;
    let addr = h160_to_fvm_addr(addr);
    let res = data.client.actor_state(&addr, Some(height)).await?;

    match res.value {
        Some((_, state)) => {
            let nonce = state.sequence;
            Ok(et::U64::from(nonce))
        }
        None => error(ExitCode::USR_NOT_FOUND, format!("actor {addr} not found")),
    }
}

/// Returns the receipt of a transaction by transaction hash.
pub async fn get_transaction_receipt<C>(
    data: JsonRpcData<C>,
    Params((tx_hash,)): Params<(et::H256,)>,
) -> JsonRpcResult<Option<et::TransactionReceipt>>
where
    C: Client + Sync + Send,
{
    let hash = tendermint::Hash::try_from(tx_hash.as_bytes().to_vec())?;
    match data.tm().tx(hash, false).await {
        Ok(res) => {
            let header: header::Response = data.tm().header(res.height).await?;
            let block_results: block_results::Response =
                data.tm().block_results(res.height).await?;
            let state_params = data.client.state_params(Some(res.height)).await?;
            let msg = fvm_ipld_encoding::from_slice::<ChainMessage>(&res.tx)?;
            if let ChainMessage::Signed(msg) = msg {
                let receipt = conv::to_rpc_receipt(
                    *msg,
                    res,
                    block_results,
                    header.header,
                    state_params.value.base_fee,
                )?;
                Ok(Some(receipt))
            } else {
                error(ExitCode::USR_ILLEGAL_ARGUMENT, "incompatible transaction")
            }
        }
        Err(e) if e.to_string().contains("not found") => Ok(None),
        Err(e) => error(ExitCode::USR_UNSPECIFIED, e),
    }
}

/// Returns the number of uncles in a block from a block matching the given block hash.
///
/// It will always return 0 since Tendermint doesn't have uncles.
pub async fn get_uncle_count_by_block_hash<C>(
    _data: JsonRpcData<C>,
    _params: Params<(et::H256,)>,
) -> JsonRpcResult<et::U256> {
    Ok(et::U256::zero())
}

/// Returns the number of uncles in a block from a block matching the given block number.
///
/// It will always return 0 since Tendermint doesn't have uncles.
pub async fn get_uncle_count_by_block_number<C>(
    _data: JsonRpcData<C>,
    _params: Params<(et::BlockNumber,)>,
) -> JsonRpcResult<et::U256> {
    Ok(et::U256::zero())
}

/// Returns information about a uncle of a block by hash and uncle index position.
///
/// It will always return None since Tendermint doesn't have uncles.
pub async fn get_uncle_by_block_hash_and_index<C>(
    _data: JsonRpcData<C>,
    _params: Params<(et::H256, et::U64)>,
) -> JsonRpcResult<Option<et::Block<et::H256>>> {
    Ok(None)
}

/// Returns information about a uncle of a block by number and uncle index position.
///
/// It will always return None since Tendermint doesn't have uncles.
pub async fn get_uncle_by_block_number_and_index<C>(
    _data: JsonRpcData<C>,
    _params: Params<(et::BlockNumber, et::U64)>,
) -> JsonRpcResult<Option<et::Block<et::H256>>> {
    Ok(None)
}

fn h160_to_fvm_addr(addr: et::H160) -> fvm_shared::address::Address {
    Address::from(EthAddress(addr.0))
}

/// Fetch transaction results to produce the full block.
async fn enrich_block<C>(
    data: JsonRpcData<C>,
    block: tendermint::Block,
    full_tx: bool,
) -> JsonRpcResult<et::Block<serde_json::Value>>
where
    C: Client + Sync + Send,
{
    let height = block.header().height;

    let state_params = data.client.state_params(Some(height)).await?;
    let base_fee = state_params.value.base_fee;
    let chain_id = ChainID::from(state_params.value.chain_id);

    let block_results: block_results::Response = data.tm().block_results(height).await?;

    let block = conv::to_rpc_block(block, block_results, base_fee, chain_id)?;

    let block = if full_tx {
        conv::map_rpc_block_txs(block, serde_json::to_value)?
    } else {
        conv::map_rpc_block_txs(block, |h| serde_json::to_value(h.hash))?
    };

    Ok(block)
}

/// Get a transaction from a block by index.
async fn transaction_by_index<C>(
    data: JsonRpcData<C>,
    block: tendermint::Block,
    index: et::U64,
) -> JsonRpcResult<Option<et::Transaction>>
where
    C: Client + Sync + Send,
{
    if let Some(msg) = block.data().get(index.as_usize()) {
        let hash = tendermint::hash::Hash::from_bytes(tendermint::hash::Algorithm::Sha256, msg)?;

        let msg = fvm_ipld_encoding::from_slice::<ChainMessage>(msg)?;

        if let ChainMessage::Signed(msg) = msg {
            let sp = data
                .client
                .state_params(Some(block.header().height))
                .await?;

            let chain_id = ChainID::from(sp.value.chain_id);
            let mut tx = conv::to_rpc_transaction(hash, *msg, chain_id)?;
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
