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

use crate::storage_layout::{
    NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, SUBNETS_MAPPING_SLOT, SUBNET_TOPDOWN_NONCE_OFFSET,
};
use crate::types::{EpochProofWithCertificate, FinalizedTipsets};
use anyhow::{Context, Result};
use cid::Cid;
use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::types::H256;
use fendermint_vm_evm_event_utils::{
    decode_new_power_change_request, decode_new_topdown_message, parse_0x_bytes,
    parse_u64_from_0x_word_low64, raw_log_from_event_proof,
};
use ipc_actors_abis::{lib_gateway, lib_power_change_log};
use proofs::proofs::common::bundle::{UnifiedProofBundle, UnifiedVerificationResult};
use proofs::proofs::events::bundle::EventProofBundle;
use proofs::proofs::events::verifier::verify_event_proof;
use proofs::proofs::storage::verifier::verify_storage_proof;

use proofs::proofs::common::evm::{extract_evm_log, hash_event_signature};
use proofs::proofs::storage::utils::compute_mapping_slot;

pub struct ProofVerifier {
    events: Vec<Vec<[u8; 32]>>,
    subnet_hash_key: [u8; 32],
    expected_topdown_topic_1: Option<[u8; 32]>,
}

/// Cursor derived from *proved* end-of-epoch storage values.
///
/// If provided across epochs, this lets us detect omitted events at the beginning of an epoch by
/// checking that the storage delta matches the number of events observed in the bundle.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EventNumberCursor {
    /// Next top-down message nonce on the **parent gateway** (`subnets[...].topDownNonce`)
    /// after applying the epoch.
    pub next_parent_topdown_nonce: u64,
    /// Next power-change configuration number on the **parent gateway**
    /// (`validatorsTracker.changes.nextConfigurationNumber`) after applying the epoch.
    pub next_parent_power_change_config_number: u64,
}

impl ProofVerifier {
    pub fn new(subnet_hash_key: [u8; 32], subnet_actor_topic_1: Option<[u8; 32]>) -> Self {
        let mut topdown_topics = vec![hash_event_signature(
            &lib_gateway::NewTopDownMessageFilter::abi_signature(),
        )];
        if let Some(topic_1) = subnet_actor_topic_1 {
            topdown_topics.push(topic_1);
        }

        let events = vec![
            topdown_topics,
            vec![hash_event_signature(
                &lib_power_change_log::NewPowerChangeRequestFilter::abi_signature(),
            )],
        ];

        Self {
            events,
            subnet_hash_key,
            expected_topdown_topic_1: subnet_actor_topic_1,
        }
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

    /// Verify semantic properties of the EVM events included in a proof bundle.
    ///
    /// This is **not** inclusion verification (that is handled by [`ProofVerifier::verify_proof_bundle_with_tipsets`]).
    /// Instead, this checks properties like:
    /// - contiguity of top-down event nonces
    /// - contiguity of power-change configuration numbers
    ///
    /// All checks are anchored to proved end-of-epoch storage values:
    /// - `subnets[...].topDownNonce`
    /// - `validatorsTracker.changes.nextConfigurationNumber`
    ///
    /// If `cursor` is provided (derived from the previous epoch's proved end values), we also
    /// verify that `end - prev_end == observed_count`, which detects omitted events at the
    /// beginning of an epoch.
    pub(crate) fn verify_event_number_continuity(
        &self,
        parent_epoch: i64,
        bundle: &UnifiedProofBundle,
        cursor: &mut EventNumberCursor,
    ) -> Result<()> {
        // 1) Extract values.
        let mut nums =
            extract_epoch_event_numbers(parent_epoch, bundle, self.expected_topdown_topic_1)
                .with_context(|| {
                    format!("failed to extract event numbers for epoch {parent_epoch}")
                })?;

        // 2) Verify local contiguity within the epoch.
        verify_contiguous_u64(&mut nums.topdown_nonces, "top-down message nonces")?;
        verify_contiguous_u64(
            &mut nums.config_numbers,
            "power-change configuration numbers",
        )?;

        // 3) Anchor both sequences to proved "next" storage values.
        // Storage holds the next nonce/config-number *after* applying the epoch.
        let next_topdown = next_topdown_message_nonce_from_storage(bundle, self.subnet_hash_key)?;
        let next_cfg = next_power_change_config_number_from_storage(bundle)?;

        verify_sequence_against_storage_next(
            "top-down message nonces",
            next_topdown,
            Some(cursor.next_parent_topdown_nonce),
            &nums.topdown_nonces,
        )?;
        verify_sequence_against_storage_next(
            "power-change configuration numbers",
            next_cfg,
            Some(cursor.next_parent_power_change_config_number),
            &nums.config_numbers,
        )?;

        cursor.next_parent_topdown_nonce = next_topdown;
        cursor.next_parent_power_change_config_number = next_cfg;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let verifier = ProofVerifier::new([0u8; 32], None);
        assert_eq!(verifier.events.len(), 2);
    }
}

// (Semantic continuity verification lives on `ProofVerifier` to access `subnet_id`.)

fn h256_to_0x(h: H256) -> String {
    format!("0x{}", hex::encode(h.as_bytes()))
}

fn expected_topdown_nonce_slot(subnet_hash_key: [u8; 32]) -> H256 {
    let base = compute_mapping_slot(subnet_hash_key, SUBNETS_MAPPING_SLOT);
    let mut slot_bytes = base;
    let base_u256 = ethers::types::U256::from_big_endian(&base);
    let slot_u256 = base_u256 + ethers::types::U256::from(SUBNET_TOPDOWN_NONCE_OFFSET);
    slot_u256.to_big_endian(&mut slot_bytes);
    H256::from(slot_bytes)
}

fn next_topdown_message_nonce_from_storage(
    bundle: &UnifiedProofBundle,
    subnet_hash_key: [u8; 32],
) -> Result<u64> {
    let expected_slot = h256_to_0x(expected_topdown_nonce_slot(subnet_hash_key));
    let storage = bundle
        .storage_proofs
        .iter()
        .find(|sp| sp.slot.eq_ignore_ascii_case(&expected_slot))
        .context("missing storage proof for subnets[...].topDownNonce")?;
    parse_u64_from_0x_word_low64(&storage.value)
        .context("failed to parse topDownNonce from storage proof")
}

fn next_power_change_config_number_from_storage(bundle: &UnifiedProofBundle) -> Result<u64> {
    let expected_slot = format!("0x{:064x}", NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT);
    let storage = bundle
        .storage_proofs
        .iter()
        .find(|sp| sp.slot.eq_ignore_ascii_case(&expected_slot))
        .context("missing storage proof for nextConfigurationNumber (slot 20)")?;
    parse_low64_from_0x_word(&storage.value)
        .context("failed to parse nextConfigurationNumber from storage proof")
}

/// Parse the low 64 bits from a 32-byte EVM storage word.
///
/// `nextConfigurationNumber` may live in a packed slot where higher bits are used by
/// neighboring fields, so we intentionally read only the low 64 bits here.
fn parse_low64_from_0x_word(word_0x: &str) -> Result<u64> {
    let mut b = parse_0x_bytes(word_0x)?;
    if b.len() > 32 {
        anyhow::bail!("expected <= 32 bytes, got {}", b.len());
    }
    if b.len() < 32 {
        let mut padded = vec![0u8; 32 - b.len()];
        padded.append(&mut b);
        b = padded;
    }
    let tail: [u8; 8] = b[24..32].try_into().expect("slice is 8 bytes");
    Ok(u64::from_be_bytes(tail))
}

fn verify_sequence_against_storage_next(
    what: &str,
    storage_next: u64,
    prev_storage_next: Option<u64>,
    values: &[u64],
) -> Result<()> {
    let count = values.len() as u64;

    // If we have a previous cursor, enforce that the storage delta matches the number of
    // observed events. This detects omitted initial events (or invented extras).
    if let Some(prev) = prev_storage_next {
        let delta = storage_next.checked_sub(prev).with_context(|| {
            format!("{what} mismatch: storage_next {storage_next} < prev {prev}")
        })?;
        if delta != count {
            anyhow::bail!(
                "{what} event-count mismatch: storage_delta {delta} != observed_count {count}"
            );
        }
    }

    if values.is_empty() {
        return Ok(());
    }

    let last = *values.last().unwrap();
    if storage_next != last + 1 {
        anyhow::bail!(
            "{what} mismatch: storage_next {storage_next} != last_event+1 {}",
            last + 1
        );
    }

    Ok(())
}

fn verify_contiguous_u64(values: &mut [u64], what: &str) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }
    values.sort_unstable();
    for w in values.windows(2) {
        let a = w[0];
        let b = w[1];
        if b == a {
            anyhow::bail!("{what} contains duplicate value: {a}");
        }
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
    expected_topdown_topic_1: Option<[u8; 32]>,
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
            if let Some(expected_topic_1) = expected_topdown_topic_1 {
                if topics.get(1).copied() != Some(H256::from(expected_topic_1)) {
                    continue;
                }
            }
            let decoded = decode_new_topdown_message(&RawLog { topics, data })?;
            out.topdown_nonces.push(decoded.message.local_nonce);
        } else if topics[0] == power_sig {
            let decoded = decode_new_power_change_request(&RawLog { topics, data })?;
            out.config_numbers.push(decoded.configuration_number);
        }
    }

    Ok(out)
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

    fn mk_storage_proof_h256(slot: H256, value_u64: u64) -> StorageProof {
        let slot = h256_to_0x(slot);
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
        let verifier = ProofVerifier::new([1u8; 32], None);
        let mut cursor = EventNumberCursor {
            next_parent_topdown_nonce: 10,
            next_parent_power_change_config_number: 7,
        };

        // Two topdown messages with contiguous nonces: 10, 11.
        let td0 = mk_topdown_rawlog(EthAddress::random(), [7u8; 32], 10);
        let td1 = mk_topdown_rawlog(EthAddress::random(), [8u8; 32], 11);

        // Power changes with contiguous configuration numbers: 7, 8.
        let pc0 = mk_power_change_rawlog(7);
        let pc1 = mk_power_change_rawlog(8);

        // nextConfigurationNumber after applying 2 changes should be 9.
        let next_config_storage = mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 9);
        // topDownNonce after applying 2 messages with nonces 10,11 should be 12.
        let topdown_nonce_storage =
            mk_storage_proof_h256(expected_topdown_nonce_slot([1u8; 32]), 12);

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![next_config_storage, topdown_nonce_storage],
            event_proofs: vec![
                mk_event_proof(epoch, td0),
                mk_event_proof(epoch, td1),
                mk_event_proof(epoch, pc0),
                mk_event_proof(epoch, pc1),
            ],
            blocks: vec![],
        };

        verifier.verify_event_number_continuity(epoch, &bundle, &mut cursor)?;
        Ok(())
    }

    #[test]
    fn continuity_check_fails_on_config_storage_mismatch() -> Result<()> {
        let epoch = 100;
        let verifier = ProofVerifier::new([1u8; 32], None);
        let mut cursor = EventNumberCursor {
            next_parent_topdown_nonce: 0,
            next_parent_power_change_config_number: 7,
        };

        let pc0 = mk_power_change_rawlog(7);
        let pc1 = mk_power_change_rawlog(8);

        // WRONG: should be 9, but we claim 10.
        let next_config_storage = mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 10);
        let topdown_nonce_storage =
            mk_storage_proof_h256(expected_topdown_nonce_slot([1u8; 32]), 0);

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![next_config_storage, topdown_nonce_storage],
            event_proofs: vec![mk_event_proof(epoch, pc0), mk_event_proof(epoch, pc1)],
            blocks: vec![],
        };

        let err = verifier
            .verify_event_number_continuity(epoch, &bundle, &mut cursor)
            .expect_err("expected mismatch to be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("power-change configuration numbers mismatch")
                || msg.contains("power-change configuration numbers event-count mismatch"),
            "unexpected error message: {msg}"
        );
        Ok(())
    }

    #[test]
    fn continuity_check_fails_on_nonce_gap() -> Result<()> {
        let epoch = 100;
        let verifier = ProofVerifier::new([1u8; 32], None);
        let mut cursor = EventNumberCursor {
            next_parent_topdown_nonce: 10,
            next_parent_power_change_config_number: 0,
        };

        let td0 = mk_topdown_rawlog(EthAddress::random(), [7u8; 32], 10);
        let td1 = mk_topdown_rawlog(EthAddress::random(), [8u8; 32], 12); // gap!
        let next_config_storage = mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 0);
        let topdown_nonce_storage =
            mk_storage_proof_h256(expected_topdown_nonce_slot([1u8; 32]), 13);

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![next_config_storage, topdown_nonce_storage],
            event_proofs: vec![mk_event_proof(epoch, td0), mk_event_proof(epoch, td1)],
            blocks: vec![],
        };

        let err = verifier
            .verify_event_number_continuity(epoch, &bundle, &mut cursor)
            .expect_err("expected nonce gap to be rejected");
        assert!(err
            .to_string()
            .contains("top-down message nonces not contiguous"));
        Ok(())
    }

    #[test]
    fn continuity_check_fails_on_duplicate_nonce() -> Result<()> {
        let epoch = 100;
        let verifier = ProofVerifier::new([1u8; 32], None);
        let mut cursor = EventNumberCursor {
            next_parent_topdown_nonce: 10,
            next_parent_power_change_config_number: 0,
        };

        // Duplicate nonce 10 twice.
        let td0 = mk_topdown_rawlog(EthAddress::random(), [7u8; 32], 10);
        let td1 = mk_topdown_rawlog(EthAddress::random(), [8u8; 32], 10);

        // Storage indicates two messages were applied (delta=2) ending at nonce 12.
        let next_config_storage = mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 0);
        let topdown_nonce_storage =
            mk_storage_proof_h256(expected_topdown_nonce_slot([1u8; 32]), 12);

        let bundle = UnifiedProofBundle {
            storage_proofs: vec![next_config_storage, topdown_nonce_storage],
            event_proofs: vec![mk_event_proof(epoch, td0), mk_event_proof(epoch, td1)],
            blocks: vec![],
        };

        let err = verifier
            .verify_event_number_continuity(epoch, &bundle, &mut cursor)
            .expect_err("expected duplicate nonce to be rejected");
        assert!(err.to_string().contains("contains duplicate"));
        Ok(())
    }

    #[test]
    fn continuity_check_detects_omitted_initial_events_via_storage_delta() -> Result<()> {
        let verifier = ProofVerifier::new([1u8; 32], None);
        // Epoch 100 starts at nonce 10 (two events) and config-number 0 (no events).
        let mut cursor = EventNumberCursor {
            next_parent_topdown_nonce: 10,
            next_parent_power_change_config_number: 0,
        };

        // Epoch 100: two topdown messages (10,11) -> end nonce 12.
        let epoch0 = 100;
        let td0 = mk_topdown_rawlog(EthAddress::random(), [7u8; 32], 10);
        let td1 = mk_topdown_rawlog(EthAddress::random(), [8u8; 32], 11);
        let bundle0 = UnifiedProofBundle {
            storage_proofs: vec![
                mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 0),
                mk_storage_proof_h256(expected_topdown_nonce_slot([1u8; 32]), 12),
            ],
            event_proofs: vec![mk_event_proof(epoch0, td0), mk_event_proof(epoch0, td1)],
            blocks: vec![],
        };
        verifier.verify_event_number_continuity(epoch0, &bundle0, &mut cursor)?;

        // Epoch 101: actual storage end indicates 3 messages (delta=3), but we only include 2 events.
        // This simulates omitting the first event in the epoch while keeping contiguity.
        let epoch1 = 101;
        let td2 = mk_topdown_rawlog(EthAddress::random(), [9u8; 32], 13);
        let td3 = mk_topdown_rawlog(EthAddress::random(), [10u8; 32], 14);
        let bundle1 = UnifiedProofBundle {
            storage_proofs: vec![
                mk_storage_proof(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, 0),
                mk_storage_proof_h256(expected_topdown_nonce_slot([1u8; 32]), 15),
            ],
            event_proofs: vec![mk_event_proof(epoch1, td2), mk_event_proof(epoch1, td3)],
            blocks: vec![],
        };
        let err = verifier
            .verify_event_number_continuity(epoch1, &bundle1, &mut cursor)
            .expect_err("expected omitted-initial-event to be detected");
        assert!(err.to_string().contains("event-count mismatch"));
        Ok(())
    }
}
