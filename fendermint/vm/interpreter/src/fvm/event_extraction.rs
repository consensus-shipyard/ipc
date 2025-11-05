// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Event extraction from F3 proof bundles
//!
//! This module provides functionality to extract and decode events from proof bundles,
//! including topdown messages and validator change events.

use anyhow::{anyhow, Context, Result};
use ethers::abi::{RawLog};
use ethers::contract::EthLogDecode;
use ethers::types as et;
use ipc_actors_abis::{lib_gateway, lib_power_change_log};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use proofs::proofs::common::bundle::UnifiedProofBundle;
use tracing::{debug, trace};

/// Extract topdown messages from a proof bundle
///
/// This function iterates through event proofs in the bundle and extracts
/// NewTopDownMessage events by:
/// 1. Finding events matching the signature
/// 2. Decoding the IpcEnvelope from the event data using contract bindings
/// 3. Returning all extracted messages
pub fn extract_topdown_messages(
    proof_bundle: &UnifiedProofBundle,
) -> Result<Vec<IpcEnvelope>> {
    let mut messages = Vec::new();

    for event_proof in &proof_bundle.event_proofs {
        let event_log = extract_event_from_proof(event_proof)?;
        
        // Try to decode as NewTopDownMessage event
        if let Ok(event) = decode_topdown_message_event(&event_log) {
            trace!(
                emitter = event_log.emitter,
                subnet = ?event.subnet,
                "Found NewTopDownMessage event"
            );
            
            // Convert from contract binding type to IPC type
            let envelope = IpcEnvelope::try_from(event.message)
                .context("Failed to convert gateway IpcEnvelope to IPC IpcEnvelope")?;
            messages.push(envelope);
        }
    }

    debug!(
        message_count = messages.len(),
        "Extracted topdown messages from proof bundle"
    );

    Ok(messages)
}

/// Extract validator changes from a proof bundle
///
/// This function iterates through event proofs and extracts
/// NewPowerChangeRequest events by:
/// 1. Finding events matching the signature
/// 2. Decoding the PowerChangeRequest from the event data using contract bindings
/// 3. Returning all extracted changes
pub fn extract_validator_changes(
    proof_bundle: &UnifiedProofBundle,
) -> Result<Vec<PowerChangeRequest>> {
    let mut changes = Vec::new();

    for event_proof in &proof_bundle.event_proofs {
        let event_log = extract_event_from_proof(event_proof)?;
        
        // Try to decode as NewPowerChangeRequest event
        if let Ok(event) = decode_power_change_event(&event_log) {
            trace!(
                emitter = event_log.emitter,
                validator = ?event.validator,
                op = event.op,
                "Found NewPowerChangeRequest event"
            );
            
            // Convert to PowerChangeRequest
            let change_request = PowerChangeRequest::try_from(event)
                .context("Failed to convert power change event to PowerChangeRequest")?;
            changes.push(change_request);
        }
    }

    debug!(
        change_count = changes.len(),
        "Extracted validator changes from proof bundle"
    );

    Ok(changes)
}

/// Extract events from a single event proof
///
/// The EventProof contains EventData which includes:
/// - emitter: actor ID that emitted the event
/// - topics: hex-encoded topics (event signature, indexed params)
/// - data: hex-encoded event data (often ABI encoded for cross-chain)
fn extract_event_from_proof(event_proof: &proofs::proofs::events::bundle::EventProof) -> Result<EventLog> {
    // Convert hex-encoded topics to H256
    let topics: Result<Vec<et::H256>> = event_proof
        .event_data
        .topics
        .iter()
        .map(|topic| {
            // Remove 0x prefix if present and parse hex
            let topic_str = topic.trim_start_matches("0x");
            let bytes = hex::decode(topic_str)
                .context(format!("Failed to decode topic hex: {}", topic))?;
            
            if bytes.len() != 32 {
                return Err(anyhow!("Topic must be 32 bytes, got {} bytes", bytes.len()));
            }
            
            Ok(et::H256::from_slice(&bytes))
        })
        .collect();
    
    let topics = topics?;
    
    // Convert hex-encoded data
    let data_str = event_proof.event_data.data.trim_start_matches("0x");
    let data = hex::decode(data_str)
        .context(format!("Failed to decode event data hex: {}", event_proof.event_data.data))?;
    
    Ok(EventLog {
        emitter: event_proof.event_data.emitter,
        topics,
        data,
    })
}

/// Helper struct to represent an event log
#[derive(Debug, Clone)]
struct EventLog {
    emitter: u64,
    topics: Vec<et::H256>,
    data: Vec<u8>,
}

/// Decode a NewTopDownMessage event using the contract bindings
fn decode_topdown_message_event(event_log: &EventLog) -> Result<lib_gateway::NewTopDownMessageFilter> {
    // Create RawLog from our EventLog
    let raw_log = RawLog {
        topics: event_log.topics.clone(),
        data: event_log.data.clone(),
    };
    
    // Use the contract binding's decoding
    lib_gateway::NewTopDownMessageFilter::decode_log(&raw_log)
        .map_err(|e| anyhow!("Failed to decode NewTopDownMessage event: {}", e))
}

/// Decode a NewPowerChangeRequest event using the contract bindings  
fn decode_power_change_event(event_log: &EventLog) -> Result<lib_power_change_log::NewPowerChangeRequestFilter> {
    // Create RawLog from our EventLog
    let raw_log = RawLog {
        topics: event_log.topics.clone(),
        data: event_log.data.clone(),
    };
    
    // Use the contract binding's decoding
    lib_power_change_log::NewPowerChangeRequestFilter::decode_log(&raw_log)
        .map_err(|e| anyhow!("Failed to decode NewPowerChangeRequest event: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_signature_generation() {
        // Test that we can create event logs and decode them properly
        // This would require mock data, so we just verify compilation
        let _log = EventLog {
            emitter: 0,
            topics: vec![],
            data: vec![],
        };
    }
}