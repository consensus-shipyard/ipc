// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Shared helpers for decoding EVM logs from proof bundles.
//!
//! Both the interpreter and the proof-service need to decode Solidity events embedded in
//! `UnifiedProofBundle` event proofs. The proofs library stores event topics/data as hex strings,
//! so these helpers provide:
//! - Hex parsing (`0x`-prefixed strings)
//! - Conversion from `EventProof` -> `RawLog`
//! - Decoding typed events using generated contract bindings

use anyhow::{anyhow, Context, Result};
use ethers::abi::RawLog;
use ethers::contract::EthLogDecode;
use ethers::types::H256;
use ipc_actors_abis::{lib_gateway, lib_power_change_log};
use proofs::proofs::events::bundle::EventProof;

/// Parse a `0x`-prefixed hex string into bytes.
pub fn parse_0x_bytes(s: &str) -> Result<Vec<u8>> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    Ok(hex::decode(s)?)
}

/// Parse a 32-byte EVM storage word (hex string) and return the low 64 bits as a `u64`.
///
/// Solidity stores integer values left-padded in a 32-byte word. For `uint64`, the value sits in the
/// low 8 bytes (big-endian).
pub fn parse_u64_from_0x_word_low64(word_0x: &str) -> Result<u64> {
    let mut b = parse_0x_bytes(word_0x)?;
    if b.len() > 32 {
        anyhow::bail!("expected <= 32 bytes, got {}", b.len());
    }
    if b.len() < 32 {
        let mut padded = vec![0u8; 32 - b.len()];
        padded.append(&mut b);
        b = padded;
    }
    // Enforce that the word actually fits in u64 (high 192 bits must be zero).
    if b[..24].iter().any(|x| *x != 0) {
        anyhow::bail!("value does not fit in u64 (high 192 bits are non-zero)");
    }
    let tail: [u8; 8] = b[24..32].try_into().expect("slice is 8 bytes");
    Ok(u64::from_be_bytes(tail))
}

/// Convert an `EventProof` into an `ethers::abi::RawLog`.
pub fn raw_log_from_event_proof(event_proof: &EventProof) -> Result<RawLog> {
    let topics: Result<Vec<H256>> = event_proof
        .event_data
        .topics
        .iter()
        .map(|t| {
            let b = parse_0x_bytes(t)?;
            if b.len() != 32 {
                return Err(anyhow!("topic must be 32 bytes, got {}", b.len()));
            }
            Ok(H256::from_slice(&b))
        })
        .collect();
    let topics = topics?;
    let data = parse_0x_bytes(&event_proof.event_data.data)
        .with_context(|| "failed to decode event data hex")?;

    Ok(RawLog { topics, data })
}

/// Attempt to decode a `NewTopDownMessage` event.
pub fn decode_new_topdown_message(raw: &RawLog) -> Result<lib_gateway::NewTopDownMessageFilter> {
    lib_gateway::NewTopDownMessageFilter::decode_log(raw)
        .map_err(|e| anyhow!("failed to decode NewTopDownMessage: {e}"))
}

/// Attempt to decode a `NewPowerChangeRequest` event.
pub fn decode_new_power_change_request(
    raw: &RawLog,
) -> Result<lib_power_change_log::NewPowerChangeRequestFilter> {
    lib_power_change_log::NewPowerChangeRequestFilter::decode_log(raw)
        .map_err(|e| anyhow!("failed to decode NewPowerChangeRequest: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u64_accepts_short_hex_quantity() {
        assert_eq!(parse_u64_from_0x_word_low64("0x01").unwrap(), 1);
    }

    #[test]
    fn parse_u64_rejects_overflow() {
        // 2^64, i.e. low64=0 but high bits non-zero.
        assert!(parse_u64_from_0x_word_low64("0x010000000000000000").is_err());
    }
}
