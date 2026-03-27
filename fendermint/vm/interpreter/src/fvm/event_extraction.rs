// Copyright 2022-2026 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Event extraction from F3 proof bundles
//!
//! This module provides functionality to extract and decode events from proof bundles,
//! including topdown messages and validator change events.

use anyhow::{Context, Result};
use fendermint_vm_evm_event_utils::{
    decode_new_power_change_request, decode_new_topdown_message, raw_log_from_event_proof,
};
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
pub fn extract_topdown_messages(proof_bundle: &UnifiedProofBundle) -> Result<Vec<IpcEnvelope>> {
    let mut messages = Vec::new();

    for event_proof in &proof_bundle.event_proofs {
        let raw = raw_log_from_event_proof(event_proof)?;

        // Try to decode as NewTopDownMessage event.
        if let Ok(event) = decode_new_topdown_message(&raw) {
            trace!(
                emitter = event_proof.event_data.emitter,
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
        let raw = raw_log_from_event_proof(event_proof)?;

        // Try to decode as NewPowerChangeRequest event.
        if let Ok(event) = decode_new_power_change_request(&raw) {
            trace!(
                emitter = event_proof.event_data.emitter,
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

// (Decoding helpers moved to `fendermint_vm_evm_event_utils` so proof-service and interpreter share logic.)

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::abi::Tokenizable;
    use ethers::abi::{encode, Token};
    use ethers::contract::EthEvent;
    use ethers::types::{Address as EthAddress, H256, U256};
    use fvm_shared::address::Address as FilAddress;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::ActorID;
    use ipc_actors_abis::lib_gateway;
    use ipc_actors_abis::lib_power_change_log;
    use ipc_api::address::IPCAddress;
    use ipc_api::cross::{IpcEnvelope as ApiIpcEnvelope, IpcMsgKind};
    use ipc_api::ethers_address_to_fil_address;
    use ipc_api::subnet_id::SubnetID;
    use proofs::proofs::common::bundle::UnifiedProofBundle;
    use proofs::proofs::events::bundle::{EventData, EventProof};

    fn h256_to_0x_string(h: H256) -> String {
        format!("0x{}", hex::encode(h.as_bytes()))
    }

    fn bytes_to_0x_string(b: &[u8]) -> String {
        format!("0x{}", hex::encode(b))
    }

    fn mk_event_proof(topics: Vec<String>, data: String) -> EventProof {
        EventProof {
            parent_epoch: 100,
            child_epoch: 101,
            parent_tipset_cids: vec!["bafy...parent".to_string()],
            child_block_cid: "bafy...child".to_string(),
            message_cid: "bafy...msg".to_string(),
            exec_index: 0,
            event_index: 0,
            event_data: EventData {
                emitter: 1000,
                topics,
                data,
            },
        }
    }

    #[test]
    fn extracts_and_decodes_new_topdown_message_event() -> Result<()> {
        // Build a valid IPC envelope, then convert into the EVM binding struct.
        // This avoids guessing the `FvmAddress` encoding.
        let child_route = FilAddress::new_delegated(10 as ActorID, &[0x11; 20])
            .context("failed to create delegated route address")?;
        let subnet_id = SubnetID::new(314159, vec![child_route]);

        let raw_from = FilAddress::new_delegated(10 as ActorID, &[0x22; 20])
            .context("failed to create delegated sender address")?;
        let raw_to = FilAddress::new_delegated(10 as ActorID, &[0x33; 20])
            .context("failed to create delegated receiver address")?;

        let from = IPCAddress::new(&subnet_id, &raw_from)?;
        let to = IPCAddress::new(&subnet_id, &raw_to)?;

        let api_env = ApiIpcEnvelope {
            kind: IpcMsgKind::Transfer,
            to,
            value: TokenAmount::from_atto(0),
            from,
            message: vec![0xAA, 0xBB],
            local_nonce: 1,
            original_nonce: 2,
        };

        let evm_env = lib_gateway::IpcEnvelope::try_from(api_env.clone())
            .context("failed to convert api IpcEnvelope to evm IpcEnvelope")?;

        // Build event topics and data matching the proofs generator format.
        let subnet_eth = EthAddress::from_slice(&[0x11; 20]);
        let topic_subnet_bytes = encode(&[Token::Address(subnet_eth)]);
        let topic_subnet = H256::from_slice(&topic_subnet_bytes);

        let id = [0x42u8; 32];
        let topic_id_bytes = encode(&[Token::FixedBytes(id.to_vec())]);
        let topic_id = H256::from_slice(&topic_id_bytes);

        let topic0 = lib_gateway::NewTopDownMessageFilter::signature();
        let data_bytes = encode(&[evm_env.clone().into_token()]);

        let proof = mk_event_proof(
            vec![
                h256_to_0x_string(topic0),
                h256_to_0x_string(topic_subnet),
                h256_to_0x_string(topic_id),
            ],
            bytes_to_0x_string(&data_bytes),
        );

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![],
            event_proofs: vec![proof],
            blocks: vec![],
        };

        let out = extract_topdown_messages(&bundle)?;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], api_env);
        Ok(())
    }

    #[test]
    fn extracts_and_decodes_new_power_change_request_event() -> Result<()> {
        let validator_eth = EthAddress::from_slice(&[0x77; 20]);
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let configuration_number = 42u64;

        let topic0 = lib_power_change_log::NewPowerChangeRequestFilter::signature();
        let data_bytes = encode(&[
            Token::Uint(U256::from(1u8)), // PowerOperation::SetPower
            Token::Address(validator_eth),
            Token::Bytes(payload.clone()),
            Token::Uint(U256::from(configuration_number)),
        ]);

        let proof = mk_event_proof(
            vec![h256_to_0x_string(topic0)],
            bytes_to_0x_string(&data_bytes),
        );

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![],
            event_proofs: vec![proof],
            blocks: vec![],
        };

        let out = extract_validator_changes(&bundle)?;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].configuration_number, configuration_number);

        let expected_validator = ethers_address_to_fil_address(&validator_eth)?;
        assert_eq!(out[0].change.validator, expected_validator);
        assert_eq!(out[0].change.payload, payload);

        Ok(())
    }
}
