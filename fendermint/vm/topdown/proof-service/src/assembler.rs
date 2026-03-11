// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle assembler
//!
//! Generates cryptographic proofs for parent chain finality using the
//! ipc-filecoin-proofs library. The assembler is only responsible for
//! proof generation - it has no knowledge of cache entries or storage.

use crate::observe::{OperationStatus, ProofBundleGenerated};
use crate::storage_layout::{
    NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT, SUBNETS_MAPPING_SLOT, SUBNET_TOPDOWN_NONCE_OFFSET,
};
use crate::types::FinalizedTipset;
use anyhow::{Context, Result};
use ethers::abi::Token;
use ethers::contract::EthEvent;
use ethers::types::{Address as EthAddress, H256, U256};
use fvm_ipld_encoding;
use ipc_api::subnet_id::SubnetID;
use ipc_actors_abis::{lib_gateway, lib_power_change_log};
use ipc_observability::emit;
use proofs::proofs::storage::utils::compute_mapping_slot;
use proofs::{
    client::LotusClient,
    proofs::{
        common::bundle::UnifiedProofBundle, generate_proof_bundle, EventProofSpec, StorageProofSpec,
    },
};
use std::str::FromStr;
use std::time::Instant;
use url::Url;

// Event signatures for proof generation.
//
// The proofs library expects the Solidity *canonical ABI signature* string.
// Instead of hard-coding it, derive it from the contract bindings.
fn new_topdown_message_signature() -> String {
    lib_gateway::NewTopDownMessageFilter::abi_signature().into_owned()
}

fn new_power_change_request_signature() -> String {
    lib_power_change_log::NewPowerChangeRequestFilter::abi_signature().into_owned()
}

// Storage slots are defined in `storage_layout.rs` (derived from Foundry `storageLayout`).

#[cfg(test)]
mod signature_tests {
    use super::*;
    use ethers::contract::EthEvent;
    use proofs::proofs::common::evm::hash_event_signature;

    #[test]
    fn abi_signature_strings_match_contract_bindings_topic0() {
        let expected_topdown: H256 = lib_gateway::NewTopDownMessageFilter::signature();
        let got_topdown: H256 = H256(hash_event_signature(&new_topdown_message_signature()));
        assert_eq!(got_topdown, expected_topdown);

        let expected_power: H256 = lib_power_change_log::NewPowerChangeRequestFilter::signature();
        let got_power: H256 = H256(hash_event_signature(&new_power_change_request_signature()));
        assert_eq!(got_power, expected_power);
    }
}

/// Assembles proof bundles from F3 certificates and parent chain data
///
/// # Thread Safety
///
/// LotusClient from the proofs library uses Rc/RefCell internally, so it's not Send.
/// We store the URL and create clients on-demand instead of storing the client.
pub struct ProofAssembler {
    rpc_url: Url,
    gateway_actor_id: u64,
    subnet_hash_key: [u8; 32],
    topdown_topic_1: String,
}

#[derive(Clone, Debug)]
pub struct SubnetProofContext {
    pub subnet_hash_key: [u8; 32],
    pub subnet_actor_eth_topic: String,
    pub subnet_actor_topic_bytes: Option<[u8; 32]>,
}

impl ProofAssembler {
    /// Create a new proof assembler
    pub fn new(
        rpc_url: String,
        gateway_actor_id: u64,
        subnet_context: SubnetProofContext,
    ) -> Result<Self> {
        let url = Url::parse(&rpc_url).context("Failed to parse RPC URL")?;
        Ok(Self {
            rpc_url: url,
            gateway_actor_id,
            subnet_hash_key: subnet_context.subnet_hash_key,
            topdown_topic_1: subnet_context.subnet_actor_eth_topic,
        })
    }

    fn build_storage_specs(&self) -> Vec<StorageProofSpec> {
        // The gateway maps `subnets[SubnetID.toHash()]`.
        let base = compute_mapping_slot(self.subnet_hash_key, SUBNETS_MAPPING_SLOT);
        // Struct member is at relative slot 3.
        let mut slot_bytes = base;
        let base_u256 = U256::from_big_endian(&base);
        let slot_u256 = base_u256 + U256::from(SUBNET_TOPDOWN_NONCE_OFFSET);
        slot_u256.to_big_endian(&mut slot_bytes);

        vec![
            StorageProofSpec {
                actor_id: self.gateway_actor_id,
                // `subnets[<key>].topDownNonce`
                slot: H256::from(slot_bytes),
            },
            StorageProofSpec {
                actor_id: self.gateway_actor_id,
                // Fixed storage slot (not a mapping): `validatorsTracker.changes.nextConfigurationNumber`.
                slot: H256::from_low_u64_be(NEXT_CONFIG_NUMBER_ABSOLUTE_SLOT),
            },
        ]
    }

    fn build_event_specs(&self) -> Vec<EventProofSpec> {
        vec![
            EventProofSpec {
                event_signature: new_topdown_message_signature(),
                topic_1: self.topdown_topic_1.clone(),
                actor_id_filter: Some(self.gateway_actor_id),
            },
            EventProofSpec {
                event_signature: new_power_change_request_signature(),
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
        LotusClient::new(self.rpc_url.clone(), None::<&str>)
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

        emit(ProofBundleGenerated {
            highest_epoch: parent_epoch,
            storage_proofs: bundle.storage_proofs.len(),
            event_proofs: bundle.event_proofs.len(),
            witness_blocks: bundle.blocks.len(),
            bundle_size_bytes,
            status: OperationStatus::Success,
            latency: generation_start.elapsed().as_secs_f64(),
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

async fn resolve_filecoin_address_to_eth(client: &LotusClient, addr: &str) -> Result<EthAddress> {
    let raw: serde_json::Value = client
        .request(
            "Filecoin.FilecoinAddressToEthAddress",
            serde_json::json!([addr]),
        )
        .await
        .with_context(|| format!("failed to resolve Filecoin address to eth: {addr}"))?;

    let eth = if let Some(s) = raw.as_str() {
        s.to_string()
    } else if let Some(s) = raw.get("EthAddress").and_then(|v| v.as_str()) {
        s.to_string()
    } else {
        anyhow::bail!(
            "unexpected response resolving Filecoin address {addr}: {}",
            raw
        );
    };

    EthAddress::from_str(&eth)
        .with_context(|| format!("invalid eth address returned for {addr}: {eth}"))
}

pub async fn derive_subnet_proof_context(
    parent_rpc_url: &str,
    subnet_id: &SubnetID,
) -> Result<SubnetProofContext> {
    let url = Url::parse(parent_rpc_url).context("Failed to parse parent RPC URL")?;
    let client = LotusClient::new(url, None::<&str>);

    let mut route = Vec::with_capacity(subnet_id.children_as_ref().len());
    for addr in subnet_id.children_as_ref() {
        route.push(resolve_filecoin_address_to_eth(&client, &addr.to_string()).await?);
    }

    let route_tokens = route
        .iter()
        .copied()
        .map(Token::Address)
        .collect::<Vec<_>>();
    let encoded = ethers::abi::encode(&[
        Token::Uint(U256::from(subnet_id.root_id())),
        Token::Array(route_tokens),
    ]);
    let subnet_hash_key = ethers::utils::keccak256(encoded);

    let subnet_actor_eth_topic = route
        .last()
        .map(|a| format!("{:#x}", a))
        .unwrap_or_default();

    let subnet_actor_topic_bytes = route.last().map(|addr| {
        let mut topic = [0u8; 32];
        topic[12..].copy_from_slice(addr.as_bytes());
        topic
    });

    Ok(SubnetProofContext {
        subnet_hash_key,
        subnet_actor_eth_topic,
        subnet_actor_topic_bytes,
    })
}

/// Resolve an Ethereum address to a Filecoin actor ID on the parent chain.
///
/// Used at proof-service startup when `gateway_id` is configured as an Ethereum address.
pub async fn resolve_eth_address_to_actor_id(parent_rpc_url: &str, eth_addr: &str) -> Result<u64> {
    let url = Url::parse(parent_rpc_url).context("Failed to parse parent RPC URL")?;
    let client = LotusClient::new(url, None::<&str>);
    let actor_id = proofs::proofs::resolve_eth_address_to_actor_id(&client, eth_addr)
        .await
        .with_context(|| {
            format!(
                "Failed to resolve gateway Ethereum address to actor id: {}",
                eth_addr
            )
        })?;
    Ok(actor_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembler_creation() {
        let subnet_context = SubnetProofContext {
            subnet_hash_key: [0u8; 32],
            subnet_actor_eth_topic: String::new(),
            subnet_actor_topic_bytes: None,
        };
        let assembler = ProofAssembler::new(
            "http://localhost:1234".to_string(),
            1001,
            subnet_context,
        );
        assert!(assembler.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let subnet_context = SubnetProofContext {
            subnet_hash_key: [0u8; 32],
            subnet_actor_eth_topic: String::new(),
            subnet_actor_topic_bytes: None,
        };
        let assembler = ProofAssembler::new("not a url".to_string(), 1001, subnet_context);
        assert!(assembler.is_err());
    }
}
