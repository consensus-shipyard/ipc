// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle assembler
//!
//! Generates cryptographic proofs for parent chain finality using the
//! ipc-filecoin-proofs library. The assembler is only responsible for
//! proof generation - it has no knowledge of cache entries or storage.

use crate::observe::{OperationStatus, ProofBundleGenerated};
use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
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
const NEW_TOPDOWN_MESSAGE_SIGNATURE: &str = "NewTopDownMessage(address,IpcEnvelope,bytes32)";

/// Event signature for NewPowerChangeRequest from LibPowerChangeLog.sol
/// Event: NewPowerChangeRequest(PowerOperation op, address validator, bytes payload, uint64 configurationNumber)
/// Bindings: contract_bindings::lib_power_change_log::NewPowerChangeRequestFilter
/// This captures validator power changes that need to be reflected in the subnet
const NEW_POWER_CHANGE_REQUEST_SIGNATURE: &str =
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

    /// Create a LotusClient for making requests
    ///
    /// LotusClient is not Send, so we create it on-demand in each async function
    /// rather than storing it as a field.
    fn create_client(&self) -> LotusClient {
        LotusClient::new(self.rpc_url.clone(), None)
    }

    /// Generate proof bundle for a certificate
    ///
    /// Fetches tipsets and generates storage and event proofs.
    ///
    /// # Arguments
    /// * `certificate` - Cryptographically validated F3 certificate
    ///
    /// # Returns
    /// Typed unified proof bundle (storage + event proofs + witness blocks)
    pub async fn generate_proof_bundle(
        &self,
        certificate: &FinalityCertificate,
    ) -> Result<UnifiedProofBundle> {
        let generation_start = Instant::now();
        let instance_id = certificate.gpbft_instance;

        // Get the highest (most recent) epoch from the certificate
        // F3 certificates contain finalized epochs. On testnets like Calibration,
        // F3 may lag significantly behind the current chain head (sometimes days).
        // This can cause issues with RPC lookback limits.
        let highest_epoch = certificate
            .ec_chain
            .suffix()
            .last()  // Get the most recent epoch in the suffix
            .or_else(|| certificate.ec_chain.base())  // Fallback to base if suffix is empty
            .map(|ts| ts.epoch)
            .context("Certificate has no epochs in suffix or base")?;

        tracing::debug!(
            instance_id,
            highest_epoch,
            "Generating proof bundle - fetching tipsets"
        );

        // Fetch tipsets from Lotus using proofs library client
        // We need both parent and child tipsets to generate storage/event proofs:
        // - Parent tipset (at highest_epoch): Contains the state root we're proving against
        // - Child tipset (at highest_epoch + 1): Needed to prove state transitions and events
        //   that occurred when moving from parent to child
        //
        // The F3 certificate contains only the tipset CID and epoch, not the full tipset data.
        // We fetch the actual tipsets here to extract block headers, state roots, and receipts.
        let client = self.create_client();

        let parent_tipset = client
            .request(
                "Filecoin.ChainGetTipSetByHeight",
                serde_json::json!([highest_epoch, null]),
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to fetch parent tipset at epoch {} - RPC may not serve old tipsets (check lookback limit)",
                    highest_epoch
                )
            })?;

        // Child tipset is needed for proof generation - it contains the receipts and
        // state transitions from the parent tipset
        let child_tipset = client
            .request(
                "Filecoin.ChainGetTipSetByHeight",
                serde_json::json!([highest_epoch + 1, null]),
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to fetch child tipset at epoch {} - RPC may not serve old tipsets (check lookback limit)",
                    highest_epoch + 1
                )
            })?;

        tracing::debug!(
            instance_id = certificate.gpbft_instance,
            highest_epoch,
            "Fetched tipsets successfully"
        );

        // Deserialize tipsets from JSON
        let parent_api: proofs::client::types::ApiTipset =
            serde_json::from_value(parent_tipset).context("Failed to deserialize parent tipset")?;
        let child_api: proofs::client::types::ApiTipset =
            serde_json::from_value(child_tipset).context("Failed to deserialize child tipset")?;

        // Configure proof specs for Gateway contract
        // Storage:
        //   - subnets[subnetKey].topDownNonce: For topdown message ordering
        //   - nextConfigurationNumber: For power change tracking
        // Events:
        //   - NewTopDownMessage: Captures topdown messages for this subnet
        //   - NewPowerChangeRequest: Captures validator power changes
        let storage_specs = vec![
            StorageProofSpec {
                actor_id: self.gateway_actor_id,
                // Calculate slot for subnets[subnetKey].topDownNonce in the mapping
                slot: calculate_storage_slot(&self.subnet_id, TOPDOWN_NONCE_STORAGE_OFFSET),
            },
            StorageProofSpec {
                actor_id: self.gateway_actor_id,
                // nextConfigurationNumber is a direct storage variable at slot 20
                // Using an empty key with the slot offset to get the direct variable
                slot: calculate_storage_slot("", NEXT_CONFIG_NUMBER_STORAGE_SLOT),
            },
        ];

        let event_specs = vec![
            // Capture topdown messages for this specific subnet
            EventProofSpec {
                event_signature: NEW_TOPDOWN_MESSAGE_SIGNATURE.to_string(),
                // topic_1 is the indexed subnet address
                topic_1: self.subnet_id.clone(),
                actor_id_filter: Some(self.gateway_actor_id),
            },
            // Capture ALL power change requests from the gateway
            // These affect validator sets and need to be processed
            EventProofSpec {
                event_signature: NEW_POWER_CHANGE_REQUEST_SIGNATURE.to_string(),
                // No topic_1 filter - we want all power changes
                topic_1: String::new(),
                actor_id_filter: Some(self.gateway_actor_id),
            },
        ];

        tracing::debug!(
            highest_epoch,
            storage_specs_count = storage_specs.len(),
            event_specs_count = event_specs.len(),
            "Configured proof specs"
        );

        // Create LotusClient for this request (not stored due to Rc/RefCell)
        let lotus_client = self.create_client();

        // Generate proof bundle in blocking task
        // CRITICAL: The proofs library uses Rc/RefCell internally making LotusClient and
        // related types non-Send. We must use spawn_blocking to run the proof generation
        // in a separate thread, then use futures::executor::block_on to bridge the
        // async/sync worlds. This prevents blocking the main tokio runtime while
        // handling non-Send types correctly.
        let bundle = tokio::task::spawn_blocking(move || {
            // Use futures::executor to run async code without blocking the parent runtime
            futures::executor::block_on(generate_proof_bundle(
                &lotus_client,
                &parent_api,
                &child_api,
                storage_specs,
                event_specs,
            ))
        })
        .await
        .context("Proof generation task panicked")?
        .context("Failed to generate proof bundle")?;

        // Calculate bundle size for metrics
        let bundle_size_bytes = fvm_ipld_encoding::to_vec(&bundle)
            .map(|v| v.len())
            .unwrap_or(0);

        let latency = generation_start.elapsed().as_secs_f64();

        emit(ProofBundleGenerated {
            instance: instance_id,
            highest_epoch,
            storage_proofs: bundle.storage_proofs.len(),
            event_proofs: bundle.event_proofs.len(),
            witness_blocks: bundle.blocks.len(),
            bundle_size_bytes,
            status: OperationStatus::Success,
            latency,
        });

        tracing::info!(
            instance_id,
            highest_epoch,
            storage_proofs = bundle.storage_proofs.len(),
            event_proofs = bundle.event_proofs.len(),
            witness_blocks = bundle.blocks.len(),
            "Generated proof bundle"
        );

        Ok(bundle)
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
