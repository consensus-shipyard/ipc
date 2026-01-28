// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{bail, Context};
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::ParentFinalityWithCert;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::BytesSer;
use fvm_shared::clock::ChainEpoch;
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use multihash_codetable::{Code, MultihashDigest};
use std::sync::Arc;
use thiserror::Error;

use fendermint_vm_message::ipc::IpcMessage;
use fendermint_vm_topdown_proof_service::types::SerializableF3Certificate;
use fendermint_vm_topdown_proof_service::PowerEntries;

use crate::fvm::event_extraction::{extract_topdown_messages, extract_validator_changes};
use crate::fvm::state::ipc::F3LightClientCaller;
use crate::fvm::state::FvmExecState;

#[derive(Debug, Error)]
pub enum F3TopDownError {
    #[error("proof bundle not found in local cache for height {height}")]
    CacheMiss { height: ChainEpoch },
}

#[derive(Debug)]
pub struct ExtractedTopDownEffects {
    pub topdown_msgs: Vec<IpcEnvelope>,
    pub validator_changes: Vec<PowerChangeRequest>,
    pub instance_id: u64,
    pub parent_eth_block_hash: [u8; 32],
}

fn eth_hash_from_tipset_key_bytes(
    tipset: &fendermint_vm_topdown_proof_service::types::FinalizedTipset,
) -> anyhow::Result<[u8; 32]> {
    // Lotus eth block hash for a tipset is the multihash digest of `TipSetKey.Cid()`, where
    // `TipSetKey` is the concatenation of block header CID bytes (order-sensitive), and
    // `TipSetKey.Cid()` is computed over the DAG-CBOR bytestring-wrapped key bytes.
    //
    // See: https://docs.filecoin.io/basics/the-blockchain/blocks-and-tipsets#tipsets-in-the-ethereum-json-rpc
    // Force CBOR *bytestring* encoding (not array-of-ints) for the key bytes.
    let wrapped = fvm_ipld_encoding::to_vec(&BytesSer(&tipset.block_cids))
        .context("failed to CBOR-encode tipset key bytes as bytestring")?;
    let digest = Code::Blake2b256.digest(&wrapped);

    let mut out = [0u8; 32];
    out.copy_from_slice(digest.digest());
    Ok(out)
}

/// F3 finality handler - handles all F3 proof-based finality logic
/// This module encapsulates all F3-specific concerns, keeping TopDownManager clean
#[derive(Clone)]
pub struct F3TopDownHandler {
    /// Proof cache for F3-based parent finality (off-chain, local).
    proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>,
    /// F3 Light Client **actor** caller (on-chain state in the FVM).
    f3_light_client_actor_caller: F3LightClientCaller,
}

impl F3TopDownHandler {
    pub fn new(proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>) -> Self {
        Self {
            proof_cache,
            f3_light_client_actor_caller: F3LightClientCaller::new(),
        }
    }

    /// Get reference to the proof cache
    pub fn proof_cache(&self) -> &Arc<fendermint_vm_topdown_proof_service::ProofCache> {
        &self.proof_cache
    }

    /// Query proof cache for next uncommitted proof and create a chain message with proof bundle.
    ///
    /// This is the v2 proof-based approach that replaces voting with cryptographic verification.
    ///
    /// Returns `None` if:
    /// - No proof available for next height
    /// - Cache is temporarily empty (graceful degradation)
    pub fn chain_message_from_proof_cache(&self) -> Option<ChainMessage> {
        // Get next uncommitted proof (epoch after last_committed)
        let epoch_with_cert = self.proof_cache.get_next_uncommitted_epoch_with_cert()?;

        tracing::debug!(
            instance_id = epoch_with_cert.certificate.gpbft_instance,
            epoch = epoch_with_cert.epoch,
            "found next uncommitted epoch with certificate in cache"
        );

        // Convert FinalityCertificate to SerializableF3Certificate for message
        let serializable_cert = SerializableF3Certificate::from(&epoch_with_cert.certificate);

        Some(ChainMessage::Ipc(IpcMessage::ParentFinalityWithCert(
            ParentFinalityWithCert {
                height: epoch_with_cert.epoch,
                certificate: fendermint_vm_message::ipc::Certificate::FilecoinF3(serializable_cert),
            },
        )))
    }

    /// Attest a generalised top-down message during the attestation phase.
    ///
    /// Cache-first attestation.
    ///
    /// We require that:
    /// - there is an epoch proof in the local cache for `msg.height`
    /// - the certificate attached to the message matches the certificate referenced by the cache entry
    ///
    /// Proof bundle validity is verified at proof generation time (before insertion into the cache).
    pub async fn attest<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        msg: &ParentFinalityWithCert,
    ) -> anyhow::Result<()>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        let msg_cert = match &msg.certificate {
            fendermint_vm_message::ipc::Certificate::FilecoinF3(cert) => cert,
        };

        let cached = self
            .proof_cache
            .get_epoch_proof_with_certificate(msg.height)
            .ok_or_else(|| anyhow::Error::new(F3TopDownError::CacheMiss { height: msg.height }))?;

        let cached_cert = SerializableF3Certificate::from(&cached.certificate);
        if &cached_cert != msg_cert {
            bail!(
                "certificate mismatch for epoch {} (message instance {}, cache instance {})",
                msg.height,
                msg_cert.gpbft_instance,
                cached_cert.gpbft_instance
            );
        }

        // Check on-chain continuity (this needs actor state access, hence in attestation).
        let f3_state = self.get_f3_light_client_actor_state(state)?;
        let instance_id = cached.certificate.gpbft_instance;

        // Certificate instance must not go backwards; it either stays the same (multiple epochs can
        // be proven under the same certificate) or advances by exactly 1.
        if instance_id < f3_state.latest_instance_id {
            bail!(
                "certificate instance went backwards: {} < {}",
                instance_id,
                f3_state.latest_instance_id
            );
        }
        if instance_id > f3_state.latest_instance_id + 1 {
            bail!(
                "certificate instance jumped: {} > {}",
                instance_id,
                f3_state.latest_instance_id + 1
            );
        }

        // Epoch ordering:
        //
        // Filecoin can have null rounds (epochs with no tipsets), so tipset heights may skip.
        // We must not require strict +1 sequencing. Instead, require that the proposal targets
        // the next *available* cached epoch after the last committed one.
        let expected_epoch = self
            .proof_cache
            .get_next_uncommitted_epoch()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no cached proof available after last committed epoch {}",
                    f3_state.latest_finalized_height
                )
            })?;
        if msg.height != expected_epoch {
            bail!(
                "unexpected epoch: message height {} != expected {}",
                msg.height,
                expected_epoch
            );
        }

        Ok(())
    }

    /// Execute F3-specific logic for a generalised top-down message.
    /// Returns the topdown messages and validator changes to be processed by TopDownManager.
    pub fn extract_top_down_effects(
        &self,
        msg: &ParentFinalityWithCert,
    ) -> anyhow::Result<ExtractedTopDownEffects> {
        // Cache is the source of truth: get the proof + certificate for this epoch.
        let cached = self
            .proof_cache
            .get_epoch_proof_with_certificate(msg.height)
            .ok_or_else(|| anyhow::Error::new(F3TopDownError::CacheMiss { height: msg.height }))?;

        // We don't validate the message certificate here; that happens during attestation.
        let instance_id = cached.certificate.gpbft_instance;

        // Deterministically derive the FEVM/Ethereum-view block hash from the cached tipset key.
        //
        // In Lotus, the eth "block hash" for a tipset is `EthHashFromCid(TipSetKey.Cid())`,
        // where `TipSetKey.Cid()` is the CID of the DAG-CBOR bytestring-wrapped key bytes.
        //
        // We already have the former tipset key bytes in the cache as `FinalizedTipset.block_cids`.
        let tipset = cached
            .finalized_tipsets
            .iter()
            .find(|t| t.epoch == msg.height)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "tipset key bytes not found in certificate for epoch {}",
                    msg.height
                )
            })?;
        let parent_eth_block_hash = eth_hash_from_tipset_key_bytes(tipset)
            .context("failed to derive parent eth block hash from cached tipset key")?;

        tracing::debug!(
            instance = instance_id,
            height = msg.height,
            "executing F3 generalised top-down"
        );

        let msgs = extract_topdown_messages(&cached.proof_bundle)?;
        let validator_changes = extract_validator_changes(&cached.proof_bundle)?;

        tracing::debug!(
            message_count = msgs.len(),
            validator_changes_count = validator_changes.len(),
            "extracted topdown effects from proof bundle"
        );

        Ok(ExtractedTopDownEffects {
            topdown_msgs: msgs,
            validator_changes,
            instance_id,
            parent_eth_block_hash,
        })
    }

    /// Finalize F3 execution after all top-down effects have been applied successfully.
    ///
    /// This updates the on-chain F3 light client state and marks the epoch as committed in the proof cache.
    pub fn finalize_after_execution<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        epoch: fvm_shared::clock::ChainEpoch,
        instance_id: u64,
    ) -> anyhow::Result<()>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        // Update F3LightClientActor with new certificate state (on-chain).
        let power_table = ActorPowerTable::try_from(&self.get_power_table(instance_id)?)?.0;
        self.update_f3_light_client_actor_state(state, instance_id, epoch, power_table)?;
        tracing::debug!(instance = instance_id, "updated F3LightClientActor state");

        // Mark epoch as committed in cache.
        if let Err(e) = self.mark_committed(epoch, instance_id) {
            tracing::warn!(
                error = %e,
                epoch,
                instance = instance_id,
                "failed to mark epoch as committed in cache"
            );
        } else {
            tracing::debug!(
                epoch,
                instance = instance_id,
                "marked epoch as committed in cache"
            );
        }

        Ok(())
    }

    /// Get power table for a certificate instance from the **cache** (off-chain).
    fn get_power_table(&self, instance_id: u64) -> anyhow::Result<PowerEntries> {
        let cert_entry = self
            .proof_cache
            .get_certificate(instance_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "certificate not found in cache for instance {}",
                    instance_id
                )
            })?;

        Ok(cert_entry.power_table)
    }

    /// Mark epoch as committed in the **cache** (off-chain).
    fn mark_committed(
        &self,
        epoch: fvm_shared::clock::ChainEpoch,
        instance_id: u64,
    ) -> anyhow::Result<()> {
        self.proof_cache
            .mark_committed(epoch, instance_id)
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to mark epoch {} as committed in cache: {}",
                    epoch,
                    e
                )
            })
    }

    /// Get F3 Light Client **actor** state (on-chain).
    fn get_f3_light_client_actor_state<DB>(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<fendermint_vm_actor_interface::f3_light_client::GetStateResponse>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        self.f3_light_client_actor_caller
            .get_state(state)
            .context("failed to get F3LightClientActor state")
    }

    /// Update F3 Light Client **actor** state (on-chain).
    fn update_f3_light_client_actor_state<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        latest_instance_id: u64,
        latest_finalized_height: fvm_shared::clock::ChainEpoch,
        power_table: Vec<fendermint_vm_actor_interface::f3_light_client::PowerEntry>,
    ) -> anyhow::Result<()>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        self.f3_light_client_actor_caller
            .update_state(
                state,
                latest_instance_id,
                latest_finalized_height,
                power_table,
            )
            .context("failed to update F3LightClientActor state")
    }
}

/// Local helper newtype so we can provide a clean `From` impl at the conversion boundary.
struct ActorPowerTable(Vec<fendermint_vm_actor_interface::f3_light_client::PowerEntry>);

impl TryFrom<&PowerEntries> for ActorPowerTable {
    type Error = anyhow::Error;

    fn try_from(entries: &PowerEntries) -> Result<Self, Self::Error> {
        use num_bigint::Sign;

        let out = entries
            .iter()
            .map(|pe| {
                let (sign, power_be) = pe.power.to_bytes_be();
                if sign == Sign::Minus {
                    anyhow::bail!("negative power for participant id {}", pe.id);
                }

                Ok(fendermint_vm_actor_interface::f3_light_client::PowerEntry {
                    id: pe.id,
                    public_key: pe.pub_key.0.clone(),
                    power_be,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::F3TopDownHandler;
    use crate::fvm::state::FvmGenesisState;
    use crate::fvm::store::memory::MemoryBlockstore;
    use anyhow::Context;
    use cid::multihash::Multihash;
    use fendermint_vm_actor_interface::{f3_light_client, gas_market, init, system};
    use fendermint_vm_core::Timestamp;
    use fendermint_vm_genesis::PowerScale;
    use fendermint_vm_message::chain::ChainMessage;
    use fendermint_vm_message::ipc::{Certificate, IpcMessage};
    use fendermint_vm_topdown_proof_service::config::CacheConfig;
    use fendermint_vm_topdown_proof_service::types::{
        CertificateEntry, EpochProofEntry, SerializableCertificateEntry, SerializableECChainEntry,
        SerializableF3Certificate, SerializablePowerEntries, SerializablePowerEntry,
        SerializableSupplementalData,
    };
    use fendermint_vm_topdown_proof_service::ProofCache;
    use fvm::engine::MultiEngine;
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::version::NetworkVersion;
    use num_traits::Zero;
    use proofs::proofs::common::bundle::UnifiedProofBundle;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    #[test]
    fn test_eth_hash_from_tipset_key_bytes_matches_mainnet_vector() -> anyhow::Result<()> {
        // Mainnet test vector for epoch 5707380.
        // Generated from:
        // - Filecoin.ChainGetTipSetByHeight(5707380).Cids (in RPC order)
        // - Filecoin.EthGetBlockByNumber(5707380).hash
        //
        // IMPORTANT: CID order is significant; do NOT sort these.
        let cids = [
            "bafy2bzacedqyixpeoqskjviifl6s2jmsabnexnw5ho77wakh3s3bevsdyqyle",
            "bafy2bzaceacu7rvgnnmeq2ibhtzil6ygw6govnaqdia3yr3otetfk7whiohkq",
            "bafy2bzaceccqjvlsxeesvb4mksxa62n47o7ahuvkpfue32qkwpjcbuw3lhdja",
            "bafy2bzaced46vktulg7par7y3b5uwemwsamqucdn3mpbtezcxfv36gcfwumpy",
            "bafy2bzaceb26bjmzcqkvnrwlfr7o3kxrdliqf6d5metxta5genc2fg7qz7x2y",
        ];
        let expected_hex = "b1b336d1164ed4a696920245d94d3a0c32d25b7d6d4758b51d7218e4f932b785";
        let expected = hex::decode(expected_hex)?;
        let expected: [u8; 32] = expected
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected hash must be 32 bytes"))?;

        let mut block_cids = Vec::new();
        for s in cids {
            let cid: cid::Cid = s.parse()?;
            block_cids.extend_from_slice(cid.to_bytes().as_slice());
        }

        let tipset = fendermint_vm_topdown_proof_service::types::FinalizedTipset {
            epoch: 5707380,
            block_cids,
        };

        let got = super::eth_hash_from_tipset_key_bytes(&tipset)?;
        assert_eq!(got, expected);
        Ok(())
    }

    fn mk_test_certificate_entry(instance_id: u64, epochs: Vec<ChainEpoch>) -> CertificateEntry {
        let mh = Multihash::<64>::wrap(0x12, &[0u8; 32]).expect("valid multihash");
        let power_table_cid = cid::Cid::new_v1(0x55, mh).to_string();

        let ec_chain = epochs
            .into_iter()
            .map(|epoch| SerializableECChainEntry {
                epoch,
                key: vec!["0".to_string()],
                power_table: power_table_cid.clone(),
                commitments: vec![0u8; 32],
            })
            .collect();

        let serializable = SerializableCertificateEntry {
            certificate: SerializableF3Certificate {
                gpbft_instance: instance_id,
                ec_chain,
                supplemental_data: SerializableSupplementalData {
                    power_table: power_table_cid.clone(),
                    commitments: vec![0u8; 32],
                },
                signers: vec![0],
                signature: vec![],
                power_table_delta: vec![],
            },
            power_table: SerializablePowerEntries(vec![
                SerializablePowerEntry {
                    id: 1,
                    power: "1000".to_string(),
                    pub_key: vec![1u8; 48],
                },
                SerializablePowerEntry {
                    id: 2,
                    power: "2000".to_string(),
                    pub_key: vec![2u8; 48],
                },
            ]),
            source_rpc: "test".to_string(),
            fetched_at: std::time::SystemTime::now(),
        };

        CertificateEntry::try_from(serializable).expect("valid certificate entry")
    }

    #[tokio::test]
    async fn f3_topdown_handler_end_to_end_cache_to_finalize() -> anyhow::Result<()> {
        // Minimal FVM genesis state with F3LightClientActor so attestation can query actor state.
        let store = MemoryBlockstore::new();
        let multi_engine = Arc::new(MultiEngine::new(1));
        let mut genesis_state = FvmGenesisState::new(
            store,
            multi_engine,
            actors_builtin_car::CAR,
            actors_custom_car::CAR,
        )
        .await
        .context("failed to create FVM genesis state")?;

        // System actor (required so the FVM can load the builtin actor manifest).
        genesis_state
            .create_builtin_actor(
                system::SYSTEM_ACTOR_CODE_ID,
                system::SYSTEM_ACTOR_ID,
                &system::State {
                    builtin_actors: genesis_state.manifest_data_cid,
                },
                TokenAmount::zero(),
                None,
            )
            .context("failed to create system actor")?;

        // Init actor (safe default for message execution environment).
        let (init_state, _addr_to_id) = init::State::new(
            genesis_state.store(),
            "test".to_string(),
            &[],
            &BTreeSet::new(),
            0,
        )
        .context("failed to create init state")?;
        genesis_state
            .create_builtin_actor(
                init::INIT_ACTOR_CODE_ID,
                init::INIT_ACTOR_ID,
                &init_state,
                TokenAmount::zero(),
                None,
            )
            .context("failed to create init actor")?;

        // Gas market custom actor: required by BlockGasTracker initialization.
        let gas_market_state = fendermint_actor_gas_market_eip1559::State {
            base_fee: TokenAmount::from_atto(100),
            constants: fendermint_actor_gas_market_eip1559::Constants::default(),
        };
        genesis_state
            .create_custom_actor(
                fendermint_actor_gas_market_eip1559::ACTOR_NAME,
                gas_market::GAS_MARKET_ACTOR_ID,
                &gas_market_state,
                TokenAmount::zero(),
                None,
            )
            .context("failed to create gas market actor")?;

        let instance_id = 7u64;
        let base_epoch: ChainEpoch = 50;
        let genesis_power_table = vec![f3_light_client::PowerEntry {
            id: 10,
            public_key: vec![9u8; 48],
            power_be: vec![9],
        }];

        let f3_state = fendermint_actor_f3_light_client::state::State::new(
            genesis_state.store(),
            instance_id,
            base_epoch,
            genesis_power_table,
        )
        .context("failed to create F3 light client actor state")?;
        genesis_state
            .create_custom_actor(
                fendermint_actor_f3_light_client::F3_LIGHT_CLIENT_ACTOR_NAME,
                f3_light_client::F3_LIGHT_CLIENT_ACTOR_ID,
                &f3_state,
                TokenAmount::zero(),
                None,
            )
            .context("failed to create F3 light client actor")?;

        // Initialize execution params (required for executing implicit/read-only messages).
        genesis_state
            .init_exec_state(
                Timestamp(1),
                NetworkVersion::V21,
                TokenAmount::from_atto(100),
                TokenAmount::zero(),
                1,
                0 as PowerScale,
            )
            .context("failed to init exec state")?;
        let mut exec_state = genesis_state
            .into_exec_state()
            .map_err(|_| anyhow::anyhow!("genesis exec state missing"))?;

        // Prepare a cache with exactly one next epoch proof.
        let cache = ProofCache::new(
            base_epoch,
            instance_id,
            CacheConfig {
                lookahead_instances: 10,
                retention_epochs: 10,
            },
        );
        cache
            .insert_certificate(mk_test_certificate_entry(
                instance_id,
                vec![base_epoch, base_epoch + 1],
            ))
            .context("failed to insert certificate")?;
        cache
            .insert_epoch_proofs(vec![EpochProofEntry::new(
                base_epoch + 1,
                UnifiedProofBundle {
                    storage_proofs: vec![],
                    event_proofs: vec![],
                    blocks: vec![],
                },
                instance_id,
            )])
            .context("failed to insert epoch proof")?;

        let handler = F3TopDownHandler::new(Arc::new(cache.clone()));

        // Propose from cache.
        let chain_msg = handler
            .chain_message_from_proof_cache()
            .expect("next uncommitted epoch proof exists");
        let msg = match chain_msg {
            ChainMessage::Ipc(IpcMessage::ParentFinalityWithCert(m)) => m,
            other => anyhow::bail!("unexpected chain message: {other:?}"),
        };
        assert_eq!(msg.height, base_epoch + 1);

        // Attest: cache match + on-chain continuity.
        handler
            .attest(&mut exec_state, &msg)
            .await
            .context("attestation failed")?;

        // Extract effects (should be empty in this fabricated proof bundle).
        let extracted = handler.extract_top_down_effects(&msg)?;
        assert!(extracted.topdown_msgs.is_empty());
        assert!(extracted.validator_changes.is_empty());
        assert_eq!(extracted.instance_id, instance_id);

        // Finalize: updates actor state + marks cache committed.
        handler
            .finalize_after_execution(&mut exec_state, msg.height, extracted.instance_id)
            .context("finalize failed")?;

        // Actor state updated.
        let caller = crate::fvm::state::ipc::F3LightClientCaller::new();
        let actor_state = caller.get_state(&mut exec_state)?;
        assert_eq!(actor_state.latest_instance_id, instance_id);
        assert_eq!(actor_state.latest_finalized_height, base_epoch + 1);
        assert_eq!(actor_state.power_table.len(), 2);
        assert_eq!(actor_state.power_table[0].id, 1);
        assert_eq!(actor_state.power_table[0].power_be, vec![0x03, 0xE8]);

        // Cache committed cursor updated.
        assert_eq!(
            handler.proof_cache().last_committed(),
            (base_epoch + 1, instance_id)
        );

        // Sanity: message certificate is FilecoinF3 (we don't decode internals here).
        match msg.certificate {
            Certificate::FilecoinF3(_) => {}
        }

        Ok(())
    }
}
