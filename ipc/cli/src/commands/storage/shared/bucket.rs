// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Bucket operations for on-chain storage management
//!
//! This module provides functions to interact with bucket smart contracts.

use anyhow::{anyhow, Context, Result};
use ethers::abi::{encode as abi_encode, Token};
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_actor_bucket::{
    AddParams, GetParams, ListObjectsReturn, ListParams, Method as BucketMethod, Object,
};
use fendermint_rpc::{
    message::GasParams,
    tx::{BoundClient, TxClient, TxCommit},
    QueryClient,
};
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fendermint_vm_actor_interface::evm;
use fvm_shared::{address::Address, chainid::ChainID, econ::TokenAmount, message::Message};
use num_traits::Zero;
use std::collections::HashMap;

const READ_GAS_LIMIT: u64 = 10_000_000_000;
const DEFAULT_GAS_PREMIUM_ATTO: u64 = 100_000;
/// Fallback gas limit when estimate_gas returns 0 (e.g. node rejects sequence=0 simulation).
/// Must cover the most expensive write path (deleteObject calls into the blobs actor).
const DEFAULT_TX_GAS_LIMIT: u64 = 500_000_000;
/// Headroom multiplier applied on top of the estimated gas limit to absorb estimation drift.
const GAS_LIMIT_BUFFER_NUM: u64 = 5;
const GAS_LIMIT_BUFFER_DEN: u64 = 4; // × 1.25

/// Estimate dynamic gas parameters for a transaction.
pub async fn tx_gas_params<C>(
    client: &C,
    from: Address,
    to: Address,
    method_num: u64,
    params: RawBytes,
    value: TokenAmount,
) -> Result<GasParams>
where
    C: QueryClient + Send + Sync,
{
    let state_params = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("failed to query state params for gas estimation")?;
    let base_fee = state_params.value.base_fee;

    let estimate_msg = Message {
        version: Default::default(),
        from,
        to,
        sequence: 0,
        value: value.clone(),
        method_num,
        params,
        gas_limit: 0,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };
    let gas_estimate = client
        .estimate_gas(estimate_msg, FvmQueryHeight::default())
        .await
        .context("failed to estimate gas")?;

    let gas_premium = TokenAmount::from_atto(DEFAULT_GAS_PREMIUM_ATTO);
    let gas_fee_cap = base_fee + gas_premium.clone();

    let gas_limit = if gas_estimate.value.gas_limit == 0 {
        // Some RPC configurations return 0 from estimate_gas (e.g. when sequence=0 is
        // rejected). Fall back to a generous limit that covers the most expensive write path.
        DEFAULT_TX_GAS_LIMIT
    } else {
        // Apply a 25% headroom buffer to absorb estimation drift.
        gas_estimate.value.gas_limit * GAS_LIMIT_BUFFER_NUM / GAS_LIMIT_BUFFER_DEN
    };

    Ok(GasParams {
        gas_limit,
        gas_fee_cap,
        gas_premium,
    })
}

/// Add an object to a bucket
///
/// This registers an object's metadata on-chain after the blob has been uploaded
/// to the gateway and distributed to storage nodes.
pub async fn add_object<C>(
    client: &mut C,
    bucket_address: Address,
    source: B256,
    key: String,
    hash: B256,
    recovery_hash: B256,
    size: u64,
    metadata: HashMap<String, String>,
    data_shards: u16,
    parity_shards: u16,
) -> Result<()>
where
    C: BoundClient + QueryClient + TxClient<TxCommit> + Send + Sync,
{
    let params = AddParams {
        source,
        key: key.into_bytes(),
        hash,
        recovery_hash,
        size,
        ttl: None, // Use default TTL
        metadata,
        overwrite: false,
        data_shards,
        parity_shards,
    };

    let key = String::from_utf8(params.key.clone()).context("Invalid UTF-8 in object key")?;
    let key_for_err = key.clone();
    let calldata = {
        let mut bytes = Vec::with_capacity(4 + 32 * 7);
        // addObject(bytes32,string,bytes32,bytes32,uint64,uint16,uint16)
        bytes.extend_from_slice(&[0x95, 0x79, 0xba, 0xf9]);
        bytes.extend_from_slice(&abi_encode(&[
            Token::FixedBytes(params.source.0.to_vec()),
            Token::String(key),
            Token::FixedBytes(params.hash.0.to_vec()),
            Token::FixedBytes(params.recovery_hash.0.to_vec()),
            Token::Uint(params.size.into()),
            Token::Uint((params.data_shards as u64).into()),
            Token::Uint((params.parity_shards as u64).into()),
        ]));
        bytes
    };
    let invoke_params = RawBytes::serialize(BytesSer(&calldata))
        .context("Failed to serialize FEVM calldata for addObject")?;

    let sender = client.address();
    let gas_params = tx_gas_params(
        client,
        sender,
        bucket_address,
        evm::Method::InvokeContract as u64,
        invoke_params,
        TokenAmount::zero(),
    )
    .await
    .context("Failed to estimate AddObject gas parameters")?;

    let res = TxClient::<TxCommit>::fevm_invoke(
        client,
        bucket_address,
        calldata.into(),
        TokenAmount::zero(),
        gas_params,
    )
    .await
    .map_err(|e| {
        anyhow!(
            "Failed to send AddObject transaction: {} (sender={} bucket={} key={})",
            e,
            sender,
            bucket_address,
            key_for_err
        )
    })?;

    if res.response.check_tx.code.is_err() {
        let log = &res.response.check_tx.log;
        let info = &res.response.check_tx.info;
        return Err(anyhow!(
            "AddObject check_tx failed (code {:?}): log={} info={} sender={} bucket={} key={}",
            res.response.check_tx.code,
            if log.is_empty() { "<empty>" } else { log },
            if info.is_empty() { "<empty>" } else { info },
            sender,
            bucket_address,
            key_for_err
        ));
    }

    if res.response.deliver_tx.code.is_err() {
        let log = &res.response.deliver_tx.log;
        let info = &res.response.deliver_tx.info;
        return Err(anyhow!(
            "AddObject deliver_tx failed (code {:?}): log={} info={} sender={} bucket={} key={}",
            res.response.deliver_tx.code,
            if log.is_empty() { "<empty>" } else { log },
            if info.is_empty() { "<empty>" } else { info },
            sender,
            bucket_address,
            key_for_err
        ));
    }

    Ok(())
}

/// Get an object from a bucket
pub async fn get_object<C>(
    client: &mut C,
    bucket_address: Address,
    key: String,
) -> Result<Option<Object>>
where
    C: QueryClient + Send + Sync,
{
    let params = GetParams(key.into_bytes());
    let params_bytes = RawBytes::serialize(params).context("Failed to serialize GetParams")?;

    let msg = Message {
        version: Default::default(),
        from: fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR,
        to: bucket_address,
        sequence: 0,
        value: TokenAmount::zero(),
        method_num: BucketMethod::GetObject as u64,
        params: params_bytes,
        gas_limit: READ_GAS_LIMIT,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };

    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("Failed to execute GetObject call")?;

    if response.value.code.is_err() {
        return Err(anyhow!("GetObject query failed: {}", response.value.info));
    }

    let return_data = fendermint_rpc::response::decode_data(&response.value.data)
        .context("Failed to decode response data")?;

    let result = fvm_ipld_encoding::from_slice::<Option<Object>>(&return_data)
        .context("Failed to decode GetObject response")?;

    Ok(result)
}

/// List objects in a bucket
pub async fn list_objects<C>(
    client: &C,
    bucket_address: Address,
    prefix: Option<String>,
    delimiter: Option<String>,
    start_key: Option<String>,
    limit: u64,
) -> Result<ListObjectsReturn>
where
    C: QueryClient + Send + Sync,
{
    let params = ListParams {
        prefix: prefix.unwrap_or_default().into_bytes(),
        delimiter: delimiter.unwrap_or_default().into_bytes(),
        start_key: start_key.map(|s| s.into_bytes()),
        limit,
    };

    let params_bytes = RawBytes::serialize(params).context("Failed to serialize ListParams")?;

    let msg = fvm_shared::message::Message {
        version: Default::default(),
        from: fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR,
        to: bucket_address,
        sequence: 0,
        value: TokenAmount::zero(),
        method_num: BucketMethod::ListObjects as u64,
        params: params_bytes,
        gas_limit: READ_GAS_LIMIT,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };

    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("Failed to execute ListObjects call")?;

    if response.value.code.is_err() {
        return Err(anyhow!("ListObjects query failed: {}", response.value.info));
    }

    let return_data = fendermint_rpc::response::decode_data(&response.value.data)
        .context("Failed to decode response data")?;

    let result = fvm_ipld_encoding::from_slice::<ListObjectsReturn>(&return_data)
        .context("Failed to decode ListObjects response")?;

    Ok(result)
}

/// Delete an object from a bucket
pub async fn delete_object<C>(client: &mut C, bucket_address: Address, key: String) -> Result<()>
where
    C: BoundClient + QueryClient + TxClient<TxCommit> + Send + Sync,
{
    let key_for_err = key.clone();
    let calldata = {
        let mut bytes = Vec::with_capacity(4 + 64);
        // deleteObject(string)
        bytes.extend_from_slice(&[0x2d, 0x7c, 0xb6, 0x00]);
        bytes.extend_from_slice(&abi_encode(&[Token::String(key)]));
        bytes
    };
    let invoke_params = RawBytes::serialize(BytesSer(&calldata))
        .context("Failed to serialize FEVM calldata for deleteObject")?;

    let sender = client.address();
    let gas_params = tx_gas_params(
        client,
        sender,
        bucket_address,
        evm::Method::InvokeContract as u64,
        invoke_params,
        TokenAmount::zero(),
    )
    .await
    .context("Failed to estimate DeleteObject gas parameters")?;

    let res = TxClient::<TxCommit>::fevm_invoke(
        client,
        bucket_address,
        calldata.into(),
        TokenAmount::zero(),
        gas_params,
    )
    .await
    .map_err(|e| {
        anyhow!(
            "Failed to send DeleteObject transaction: {} (sender={} bucket={} key={})",
            e,
            sender,
            bucket_address,
            key_for_err
        )
    })?;

    if res.response.check_tx.code.is_err() {
        let log = &res.response.check_tx.log;
        let info = &res.response.check_tx.info;
        return Err(anyhow!(
            "DeleteObject check_tx failed (code {:?}): log={} info={} sender={} bucket={} key={}",
            res.response.check_tx.code,
            if log.is_empty() { "<empty>" } else { log },
            if info.is_empty() { "<empty>" } else { info },
            sender,
            bucket_address,
            key_for_err
        ));
    }

    if res.response.deliver_tx.code.is_err() {
        let log = &res.response.deliver_tx.log;
        let info = &res.response.deliver_tx.info;
        return Err(anyhow!(
            "DeleteObject deliver_tx failed (code {:?}): log={} info={} sender={} bucket={} key={}",
            res.response.deliver_tx.code,
            if log.is_empty() { "<empty>" } else { log },
            if info.is_empty() { "<empty>" } else { info },
            sender,
            bucket_address,
            key_for_err
        ));
    }

    Ok(())
}

/// Query the chain ID from the network
pub async fn query_chain_id<C>(client: &C) -> Result<ChainID>
where
    C: QueryClient + Send + Sync,
{
    let state_params = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("Failed to query state params for chain ID")?;

    Ok(ChainID::from(state_params.value.chain_id))
}

/// Convert a hex string to a B256, with length validation.
///
/// Accepts with or without "0x" prefix. Returns an error if the decoded
/// bytes are not exactly 32 bytes long.
pub fn hex_to_b256(hex_str: &str) -> Result<B256> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).context("Invalid hex string")?;
    if bytes.len() != 32 {
        return Err(anyhow!(
            "Expected 32 bytes, got {} bytes from hex string",
            bytes.len()
        ));
    }
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    Ok(B256(array))
}

/// Convert a hash string to B256, auto-detecting hex or base32 encoding.
///
/// Supports:
/// - Hex with "0x" prefix
/// - Hex (64 hex chars)
/// - Base32 lower-case no-padding (iroh/blake3 format, 52 chars)
pub fn hash_to_b256(s: &str) -> Result<B256> {
    if s.starts_with("0x") || (s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())) {
        return hex_to_b256(s);
    }
    // Try base32 (lower-case no-padding, as used by iroh)
    let bytes = base32_decode_nopad(s).context("Failed to decode as base32")?;
    if bytes.len() < 32 {
        return Err(anyhow!(
            "Expected at least 32 bytes, got {} from base32 string",
            bytes.len()
        ));
    }
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes[..32]);
    Ok(B256(array))
}

/// Decode RFC 4648 base32 (case-insensitive, no padding required).
fn base32_decode_nopad(input: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    fn val(c: u8) -> Result<u8> {
        let c = c.to_ascii_uppercase();
        ALPHABET
            .iter()
            .position(|&a| a == c)
            .map(|p| p as u8)
            .ok_or_else(|| anyhow!("invalid base32 character: {}", c as char))
    }

    let input = input.as_bytes();
    let mut buf = Vec::with_capacity(input.len() * 5 / 8);
    let mut bits: u32 = 0;
    let mut n_bits: u32 = 0;

    for &c in input {
        if c == b'=' {
            break;
        }
        bits = (bits << 5) | val(c)? as u32;
        n_bits += 5;
        if n_bits >= 8 {
            n_bits -= 8;
            buf.push((bits >> n_bits) as u8);
            bits &= (1 << n_bits) - 1;
        }
    }
    Ok(buf)
}
