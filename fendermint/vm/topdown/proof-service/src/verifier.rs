// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle verification for block attestation
//!
//! Provides deterministic verification of proof bundles against F3 certificates.
//! Used by validators during block attestation to verify parent finality proofs.
//!
//! # Verification Flow
//!
//! The verifier checks that witness blocks in the proof bundle are certified
//! by the F3 certificates. With the two-level cache design, proofs are verified
//! against pre-merged tipsets from both the parent and child certificates.

use crate::storage_layout::NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT;
use crate::types::{EpochProofWithCertificate, FinalizedTipsets};
use anyhow::{Context, Result};
use cid::Cid;
use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::types::H256;
use fendermint_vm_evm_event_utils::{parse_u64_from_0x_word_low64, raw_log_from_event_proof};
use ipc_actors_abis::{lib_gateway, lib_power_change_log};
use proofs::proofs::common::bundle::{UnifiedProofBundle, UnifiedVerificationResult};
use proofs::proofs::events::bundle::EventProofBundle;
use proofs::proofs::events::verifier::verify_event_proof;
use proofs::proofs::storage::verifier::verify_storage_proof;

use proofs::proofs::common::evm::{ascii_to_bytes32, extract_evm_log, hash_event_signature};

pub struct ProofVerifier {
    events: Vec<Vec<[u8; 32]>>,
}

impl ProofVerifier {
    pub fn new(subnet_id: String) -> Self {
        let events = vec![
            vec![
                hash_event_signature(&lib_gateway::NewTopDownMessageFilter::abi_signature()),
                ascii_to_bytes32(&subnet_id),
            ],
            vec![hash_event_signature(
                &lib_power_change_log::NewPowerChangeRequestFilter::abi_signature(),
            )],
        ];

        Self { events }
    }

    /// Verify a inclusion proof in the proof bundle using pre-merged tipsets from certificates
    ///
    /// This is the primary verification method. It verifies that all witness
    /// blocks in the proof bundle are certified by the provided tipsets.
    ///
    /// # Arguments
    /// * `bundle` - The proof bundle to verify
    /// * `merged_tipsets` - Pre-merged tipsets from parent and child certificates
    ///
    /// # Returns
    /// Verification results for storage and event inclusion proofs
    pub fn verify_proof_bundle_with_tipsets(
        &self,
        bundle: &UnifiedProofBundle,
        finalized_tipsets: &FinalizedTipsets,
    ) -> Result<UnifiedVerificationResult> {
        let tipset_verifier = |epoch: i64, cid: &Cid| -> bool {
            finalized_tipsets
                .iter()
                .any(|ts| ts.epoch == epoch && ts.block_cids == cid.to_bytes())
        };

        self.verify_with_verifier(bundle, &tipset_verifier)
    }

    /// Verify a proof bundle from a cache entry
    ///
    /// # Arguments
    /// * `entry` - The epoch proof entry with its certificates
    ///
    /// # Returns
    /// Verification results for storage and event proofs
    pub fn verify_epoch_proof(
        &self,
        entry: &EpochProofWithCertificate,
    ) -> Result<UnifiedVerificationResult> {
        self.verify_proof_bundle_with_tipsets(&entry.proof_bundle, &entry.finalized_tipsets)
    }

    /// Internal verification using a tipset verifier closure
    fn verify_with_verifier<F>(
        &self,
        bundle: &UnifiedProofBundle,
        tipset_verifier: &F,
    ) -> Result<UnifiedVerificationResult>
    where
        F: Fn(i64, &Cid) -> bool,
    {
        // Verify storage proofs
        let mut storage_results = Vec::new();
        for proof in &bundle.storage_proofs {
            let result = verify_storage_proof(proof, &bundle.blocks, tipset_verifier)?;
            storage_results.push(result);
        }

        // Verify event proofs
        let event_bundle = EventProofBundle {
            proofs: bundle.event_proofs.clone(),
            blocks: bundle.blocks.clone(),
        };

        let parent_tipset_verifier = |epoch: i64, cids: &[Cid]| -> bool {
            cids.iter().all(|cid| tipset_verifier(epoch, cid))
        };

        let event_results = verify_event_proof(
            &event_bundle,
            &parent_tipset_verifier,
            tipset_verifier,
            Some(&self.create_event_filter()),
        )?;

        Ok(UnifiedVerificationResult {
            storage_results,
            event_results,
        })
    }

    fn create_event_filter(&self) -> impl Fn(&fvm_shared::event::ActorEvent) -> bool + '_ {
        |ev: &fvm_shared::event::ActorEvent| -> bool {
            if let Some(log) = extract_evm_log(ev) {
                self.events.iter().any(|expected_topics| {
                    log.topics.len() >= expected_topics.len()
                        && expected_topics
                            .iter()
                            .zip(log.topics.iter())
                            .all(|(expected, actual)| expected == actual)
                })
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let verifier = ProofVerifier::new("test-subnet".to_string());
        assert_eq!(verifier.events.len(), 2);
    }
}

/// Verify semantic properties of the EVM events included in a proof bundle.
///
/// This is **not** inclusion verification (that is handled by [`ProofVerifier::verify_proof_bundle_with_tipsets`]).
/// Instead, this checks properties like contiguity of top-down nonces and power-change configuration numbers.
///
/// Notes on scope:
/// - These checks are meaningful for the proof-generator as a correctness guardrail.
/// - The strongest possible checks should be anchored to proved storage (when available).
pub fn verify_event_number_continuity(
    parent_epoch: i64,
    bundle: &UnifiedProofBundle,
) -> Result<()> {
    // 1) Extract values.
    let mut nums = extract_epoch_event_numbers(parent_epoch, bundle)
        .with_context(|| format!("failed to extract event numbers for epoch {parent_epoch}"))?;

    // 2) Verify local contiguity within the epoch.
    verify_contiguous_u64(&mut nums.topdown_nonces, "top-down nonces")?;
    verify_contiguous_u64(
        &mut nums.config_numbers,
        "power change configuration numbers",
    )?;

    // 3) Anchor power-change event numbers against proved storage post-state.
    verify_next_config_number_matches_events(bundle, &nums.config_numbers)?;

    Ok(())
}

fn verify_contiguous_u64(values: &mut Vec<u64>, what: &str) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }
    values.sort_unstable();
    values.dedup();
    for w in values.windows(2) {
        let a = w[0];
        let b = w[1];
        if b != a + 1 {
            anyhow::bail!("{what} not contiguous: {a} -> {b}");
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
struct EpochEventNumbers {
    topdown_nonces: Vec<u64>,
    config_numbers: Vec<u64>,
}

fn extract_epoch_event_numbers(
    parent_epoch: i64,
    bundle: &UnifiedProofBundle,
) -> Result<EpochEventNumbers> {
    let mut out = EpochEventNumbers::default();

    let topdown_sig: H256 = lib_gateway::NewTopDownMessageFilter::signature();
    let power_sig: H256 = lib_power_change_log::NewPowerChangeRequestFilter::signature();

    for ep in &bundle.event_proofs {
        if ep.parent_epoch != parent_epoch {
            continue;
        }

        let RawLog { topics, data } = raw_log_from_event_proof(ep)?;
        if topics.is_empty() {
            continue;
        }

        if topics[0] == topdown_sig {
            let decoded =
                lib_gateway::NewTopDownMessageFilter::decode_log(&RawLog { topics, data })
                    .context("failed to decode NewTopDownMessage")?;
            out.topdown_nonces.push(decoded.message.local_nonce);
        } else if topics[0] == power_sig {
            let decoded = lib_power_change_log::NewPowerChangeRequestFilter::decode_log(&RawLog {
                topics,
                data,
            })
            .context("failed to decode NewPowerChangeRequest")?;
            out.config_numbers.push(decoded.configuration_number);
        }
    }

    Ok(out)
}

fn verify_next_config_number_matches_events(
    bundle: &UnifiedProofBundle,
    config_numbers: &[u64],
) -> Result<()> {
    if config_numbers.is_empty() {
        return Ok(());
    }

    let expected_slot = format!("0x{:064x}", NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT);
    let storage = bundle
        .storage_proofs
        .iter()
        .find(|sp| sp.slot.eq_ignore_ascii_case(&expected_slot))
        .context("missing storage proof for nextConfigurationNumber (slot 20)")?;
    let end_next = parse_u64_from_0x_word_low64(&storage.value)
        .context("failed to parse nextConfigurationNumber from storage proof")?;

    // If we observed k events ending at N, then the post-state should be N+1.
    // Also, the first observed event must be `end_next - k`.
    let last = *config_numbers.last().unwrap();
    let count = config_numbers.len() as u64;
    if end_next != last + 1 {
        anyhow::bail!(
            "nextConfigurationNumber mismatch: end_next {} != last_event+1 {}",
            end_next,
            last + 1
        );
    }
    let first = config_numbers[0];
    if end_next < count || end_next - count != first {
        anyhow::bail!(
            "nextConfigurationNumber mismatch: expected first_event {} from end_next {} and count {}",
            first,
            end_next,
            count
        );
    }

    Ok(())
}

#[cfg(test)]
mod event_number_continuity_tests {
    use super::*;
    use ethers::abi::{encode, Token};
    use ethers::types::Address as EthAddress;
    use ethers::types::U256;
    use proofs::proofs::events::bundle::{EventData, EventProof};
    use proofs::proofs::storage::bundle::StorageProof;

    fn h256_to_0x(h: H256) -> String {
        format!("0x{}", hex::encode(h.as_bytes()))
    }

    fn bytes_to_0x(b: &[u8]) -> String {
        format!("0x{}", hex::encode(b))
    }

    fn mk_event_proof(parent_epoch: i64, raw: RawLog) -> EventProof {
        EventProof {
            parent_epoch,
            child_epoch: parent_epoch + 1,
            parent_tipset_cids: vec!["bafy...parent".to_string()],
            child_block_cid: "bafy...child".to_string(),
            message_cid: "bafy...msg".to_string(),
            exec_index: 0,
            event_index: 0,
            event_data: EventData {
                emitter: 1000,
                topics: raw.topics.into_iter().map(h256_to_0x).collect(),
                data: bytes_to_0x(&raw.data),
            },
        }
    }

    fn mk_storage_proof(slot_u64: u64, value_u64: u64) -> StorageProof {
        let slot = format!("0x{:064x}", slot_u64);
        let mut word = [0u8; 32];
        word[24..].copy_from_slice(&value_u64.to_be_bytes());
        StorageProof {
            child_epoch: 0,
            child_block_cid: "bafy...child".to_string(),
            parent_state_root: "bafy...state".to_string(),
            actor_id: 1000,
            actor_state_cid: "bafy...actor".to_string(),
            storage_root: "bafy...storage".to_string(),
            slot,
            value: bytes_to_0x(&word),
        }
    }

    fn topic_from_address(addr: EthAddress) -> H256 {
        let mut b = [0u8; 32];
        b[12..].copy_from_slice(addr.as_bytes());
        H256(b)
    }

    fn topic_from_bytes32(id: [u8; 32]) -> H256 {
        H256(id)
    }

    fn mk_topdown_rawlog(subnet: EthAddress, id: [u8; 32], local_nonce: u64) -> RawLog {
        let sig: H256 = lib_gateway::NewTopDownMessageFilter::signature();
        let topics = vec![sig, topic_from_address(subnet), topic_from_bytes32(id)];

        // Encode IpcEnvelope as the only non-indexed event arg.
        // Tuple layout:
        // (uint8 kind,uint64 localNonce,uint64 originalNonce,uint256 value,
        //  ((uint64,address[]),(uint8,bytes)) to,
        //  ((uint64,address[]),(uint8,bytes)) from,
        //  bytes message)
        let subnet_id = Token::Tuple(vec![Token::Uint(U256::from(0u64)), Token::Array(vec![])]);
        let fvm_addr = Token::Tuple(vec![Token::Uint(U256::from(0u8)), Token::Bytes(vec![])]);
        let ipc_address = Token::Tuple(vec![subnet_id, fvm_addr]);

        let env = Token::Tuple(vec![
            Token::Uint(U256::from(0u8)),         // kind
            Token::Uint(U256::from(local_nonce)), // local_nonce
            Token::Uint(U256::from(local_nonce)), // original_nonce
            Token::Uint(U256::zero()),            // value
            ipc_address.clone(),                  // to
            ipc_address,                          // from
            Token::Bytes(vec![]),                 // message
        ]);

        RawLog {
            topics,
            data: encode(&[env]),
        }
    }

    fn mk_power_change_rawlog(configuration_number: u64) -> RawLog {
        let sig: H256 = lib_power_change_log::NewPowerChangeRequestFilter::signature();
        RawLog {
            topics: vec![sig],
            data: encode(&[
                Token::Uint(U256::from(0u8)), // op
                Token::Address(EthAddress::zero()),
                Token::Bytes(vec![]),
                Token::Uint(U256::from(configuration_number)),
            ]),
        }
    }

    #[test]
    fn continuity_check_passes_for_contiguous_nonces_and_config_numbers() -> Result<()> {
        let epoch = 100;

        // Two topdown messages with contiguous nonces: 10, 11.
        let td0 = mk_topdown_rawlog(EthAddress::random(), [7u8; 32], 10);
        let td1 = mk_topdown_rawlog(EthAddress::random(), [8u8; 32], 11);

        // Power changes with contiguous configuration numbers: 7, 8.
        let pc0 = mk_power_change_rawlog(7);
        let pc1 = mk_power_change_rawlog(8);

        // nextConfigurationNumber after applying 2 changes should be 9.
        let next_config_storage = mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 9);

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![next_config_storage],
            event_proofs: vec![
                mk_event_proof(epoch, td0),
                mk_event_proof(epoch, td1),
                mk_event_proof(epoch, pc0),
                mk_event_proof(epoch, pc1),
            ],
            blocks: vec![],
        };

        verify_event_number_continuity(epoch, &bundle)?;
        Ok(())
    }

    #[test]
    fn continuity_check_fails_on_config_storage_mismatch() -> Result<()> {
        let epoch = 100;

        let pc0 = mk_power_change_rawlog(7);
        let pc1 = mk_power_change_rawlog(8);

        // WRONG: should be 9, but we claim 10.
        let next_config_storage = mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 10);

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![next_config_storage],
            event_proofs: vec![mk_event_proof(epoch, pc0), mk_event_proof(epoch, pc1)],
            blocks: vec![],
        };

        let err = verify_event_number_continuity(epoch, &bundle)
            .expect_err("expected mismatch to be rejected");
        let msg = err.to_string();
        assert!(msg.contains("nextConfigurationNumber mismatch"));
        Ok(())
    }

    #[test]
    fn continuity_check_fails_on_nonce_gap() -> Result<()> {
        let epoch = 100;

        let td0 = mk_topdown_rawlog(EthAddress::random(), [7u8; 32], 10);
        let td1 = mk_topdown_rawlog(EthAddress::random(), [8u8; 32], 12); // gap!

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![],
            event_proofs: vec![mk_event_proof(epoch, td0), mk_event_proof(epoch, td1)],
            blocks: vec![],
        };

        let err = verify_event_number_continuity(epoch, &bundle)
            .expect_err("expected nonce gap to be rejected");
        assert!(err.to_string().contains("top-down nonces not contiguous"));
        Ok(())
    }
}
