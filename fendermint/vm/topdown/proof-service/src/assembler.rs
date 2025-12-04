// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle assembler
//!
//! Generates cryptographic proofs for parent chain finality using the
//! ipc-filecoin-proofs library. The assembler is only responsible for
//! proof generation - it has no knowledge of cache entries or storage.

use crate::observe::{OperationStatus, ProofBundleGenerated};
use crate::types::FinalizedTipset;
use anyhow::{Context, Result};
use fvm_ipld_encoding;
use ipc_observability::emit;
use proofs::{
    client::LotusClient,
    proofs::{
        calculate_storage_slot, common::bundle::UnifiedProofBundle, generate_proof_bundle,
        EventProofSpec, StorageProofSpec,
    },
};
use std::time::Instant;
use url::Url;

// Event signatures for proof generation
// These use Solidity's canonical format (type names, not ABI encoding)
// For contract bindings, see: contract_bindings::lib_gateway::NewTopDownMessageFilter
// and contract_bindings::lib_power_change_log::NewPowerChangeRequestFilter

/// Event signature for NewTopDownMessage from LibGateway.sol
/// Event: NewTopDownMessage(address indexed subnet, IpcEnvelope message, bytes32 indexed id)
/// Bindings: contract_bindings::lib_gateway::NewTopDownMessageFilter
pub const NEW_TOPDOWN_MESSAGE_SIGNATURE: &str = "NewTopDownMessage(address,IpcEnvelope,bytes32)";

/// Event signature for NewPowerChangeRequest from LibPowerChangeLog.sol
/// Event: NewPowerChangeRequest(PowerOperation op, address validator, bytes payload, uint64 configurationNumber)
/// Bindings: contract_bindings::lib_power_change_log::NewPowerChangeRequestFilter
/// This captures validator power changes that need to be reflected in the subnet
pub const NEW_POWER_CHANGE_REQUEST_SIGNATURE: &str =
    "NewPowerChangeRequest(PowerOperation,address,bytes,uint64)";

/// Storage slot offset for topDownNonce in the Subnet struct
/// In the Gateway actor's subnets mapping: mapping(SubnetID => Subnet)
/// The Subnet struct field layout (see contracts/contracts/structs/Subnet.sol):
///   - id (SubnetID): slot 0-1 (SubnetID has 2 fields)
///   - stake (uint256): slot 2
///   - topDownNonce (uint64): slot 3
///   - appliedBottomUpNonce (uint64): slot 3 (packed with topDownNonce)
///   - genesisEpoch (uint256): slot 4
/// We need the nonce to verify top-down message ordering
const TOPDOWN_NONCE_STORAGE_OFFSET: u64 = 3;

/// Storage slot for nextConfigurationNumber in GatewayActorStorage
/// This is used to track configuration changes for power updates
/// Based on the storage layout, nextConfigurationNumber is at slot 20
const NEXT_CONFIG_NUMBER_STORAGE_SLOT: u64 = 20;

/// Assembles proof bundles from F3 certificates and parent chain data
///
/// # Thread Safety
///
/// LotusClient from the proofs library uses Rc/RefCell internally, so it's not Send.
/// We store the URL and create clients on-demand instead of storing the client.
pub struct ProofAssembler {
    rpc_url: Url,
    gateway_actor_id: u64,
    subnet_id: String,
}

impl ProofAssembler {
    /// Create a new proof assembler
    pub fn new(rpc_url: String, gateway_actor_id: u64, subnet_id: String) -> Result<Self> {
        let url = Url::parse(&rpc_url).context("Failed to parse RPC URL")?;
        Ok(Self {
            rpc_url: url,
            gateway_actor_id,
            subnet_id,
        })
    }

    fn build_storage_specs(&self) -> Vec<StorageProofSpec> {
        vec![
            StorageProofSpec {
                actor_id: self.gateway_actor_id,
                slot: calculate_storage_slot(&self.subnet_id, TOPDOWN_NONCE_STORAGE_OFFSET),
            },
            StorageProofSpec {
                actor_id: self.gateway_actor_id,
                slot: calculate_storage_slot("", NEXT_CONFIG_NUMBER_STORAGE_SLOT),
            },
        ]
    }

    fn build_event_specs(&self) -> Vec<EventProofSpec> {
        vec![
            EventProofSpec {
                event_signature: NEW_TOPDOWN_MESSAGE_SIGNATURE.to_string(),
                topic_1: self.subnet_id.clone(),
                actor_id_filter: Some(self.gateway_actor_id),
            },
            EventProofSpec {
                event_signature: NEW_POWER_CHANGE_REQUEST_SIGNATURE.to_string(),
                topic_1: String::new(),
                actor_id_filter: Some(self.gateway_actor_id),
            },
        ]
    }

    /// Create a LotusClient for making requests
    ///
    /// LotusClient is not Send, so we create it on-demand in each async function
    /// rather than storing it as a field.
    fn create_client(&self) -> LotusClient {
        LotusClient::new(self.rpc_url.clone(), None)
    }

    /// Fetch a tipset by epoch from Lotus RPC
    async fn fetch_tipset(&self, epoch: i64) -> Result<proofs::client::types::ApiTipset> {
        let client = self.create_client();
        let json = client
            .request(
                "Filecoin.ChainGetTipSetByHeight",
                serde_json::json!([epoch, null]),
            )
            .await
            .with_context(|| format!("Failed to fetch tipset at epoch {}", epoch))?;

        serde_json::from_value(json)
            .with_context(|| format!("Failed to deserialize tipset at epoch {}", epoch))
    }

    /// Generate a proof bundle for a single epoch transition.
    ///
    /// This is the primary method for proof generation. It creates proofs for
    /// the state and events at `parent_tipset`, using `child_tipset` to access
    /// the resulting state root and receipts.
    ///
    /// # Arguments
    /// * `parent_tipset` - The epoch to prove (state/events at this height)
    /// * `child_tipset` - The epoch containing the resulting state root (typically parent_epoch + 1)
    ///
    /// # Returns
    /// A UnifiedProofBundle containing storage proofs, event proofs, and witness blocks.
    pub async fn generate_proof_for_epoch(
        &self,
        parent_tipset: FinalizedTipset,
        child_tipset: FinalizedTipset,
    ) -> Result<UnifiedProofBundle> {
        let parent_epoch = parent_tipset.epoch;
        let child_epoch = child_tipset.epoch;

        let generation_start = Instant::now();

        tracing::debug!(
            parent_epoch,
            child_epoch,
            "Generating proof for epoch - fetching tipsets"
        );

        // Fetch tipsets from Lotus and verify they match the expected ones
        let parent_api = self.fetch_tipset(parent_epoch).await?;
        let child_api = self.fetch_tipset(child_epoch).await?;

        parent_tipset
            .verify_matches(&FinalizedTipset::try_from(&parent_api)?)
            .context("Parent tipset mismatch")?;
        child_tipset
            .verify_matches(&FinalizedTipset::try_from(&child_api)?)
            .context("Child tipset mismatch")?;

        // Generate the proof bundle
        let bundle = self
            .generate_proof_bundle_internal(parent_epoch, &parent_api, &child_api)
            .await?;

        // Emit metrics
        let bundle_size_bytes = fvm_ipld_encoding::to_vec(&bundle)
            .map(|v| v.len())
            .unwrap_or(0);

        let latency = generation_start.elapsed().as_secs_f64();

        emit(ProofBundleGenerated {
            highest_epoch: parent_epoch,
            storage_proofs: bundle.storage_proofs.len(),
            event_proofs: bundle.event_proofs.len(),
            witness_blocks: bundle.blocks.len(),
            bundle_size_bytes,
            status: OperationStatus::Success,
            latency,
        });

        tracing::info!(
            parent_epoch,
            child_epoch,
            storage_proofs = bundle.storage_proofs.len(),
            event_proofs = bundle.event_proofs.len(),
            witness_blocks = bundle.blocks.len(),
            "Generated proof bundle for epoch"
        );

        Ok(bundle)
    }

    /// Internal method to generate proof bundle from already-fetched tipsets
    async fn generate_proof_bundle_internal(
        &self,
        epoch: i64,
        parent_api: &proofs::client::types::ApiTipset,
        child_api: &proofs::client::types::ApiTipset,
    ) -> Result<UnifiedProofBundle> {
        // Build specs fresh each time (external types don't implement Clone)
        let storage_specs = self.build_storage_specs();
        let event_specs = self.build_event_specs();

        tracing::debug!(
            epoch,
            storage_specs = storage_specs.len(),
            event_specs = event_specs.len(),
            "Generating proof bundle"
        );

        // Clone data for the blocking task
        let parent_api = parent_api.clone();
        let child_api = child_api.clone();
        let lotus_client = self.create_client();

        // Generate proof bundle in blocking task
        // CRITICAL: The proofs library uses Rc/RefCell internally making LotusClient and
        // related types non-Send. We must use spawn_blocking to run the proof generation
        // in a separate thread.
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current()
                .block_on(generate_proof_bundle(
                    &lotus_client,
                    &parent_api,
                    &child_api,
                    storage_specs,
                    event_specs,
                ))
                .context("Failed to generate proof bundle")
        })
        .await
        .context("Failed to join proof generation task")?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembler_creation() {
        let assembler = ProofAssembler::new(
            "http://localhost:1234".to_string(),
            1001,
            "test-subnet".to_string(),
        );
        assert!(assembler.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let assembler =
            ProofAssembler::new("not a url".to_string(), 1001, "test-subnet".to_string());
        assert!(assembler.is_err());
    }
}
