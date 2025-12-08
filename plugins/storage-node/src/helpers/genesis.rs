// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Genesis initialization for storage-node actors.

use anyhow::{Context, Result};
use fendermint_module::genesis::GenesisState;
use fendermint_vm_genesis::Genesis;
use fvm_shared::econ::TokenAmount;
use num_traits::Zero;

use crate::actor_interface::{blob_reader, blobs, recall_config};

/// Initialize storage-node actors in genesis.
///
/// Creates the three core storage actors:
/// - recall_config: Configuration for storage parameters
/// - blobs: Main storage blob actor with Ethereum address
/// - blob_reader: Read-only accessor for blobs
pub fn initialize_storage_actors<S: GenesisState>(
    state: &mut S,
    _genesis: &Genesis,
) -> Result<()> {
    tracing::info!("Initializing storage-node actors in genesis");

    // Initialize the recall config actor
    let recall_config_state = fendermint_actor_storage_config::State {
        admin: None,
        config: fendermint_actor_storage_config_shared::RecallConfig::default(),
    };
    state
        .create_custom_actor(
            fendermint_actor_storage_config::ACTOR_NAME,
            recall_config::RECALL_CONFIG_ACTOR_ID,
            &recall_config_state,
            TokenAmount::zero(),
            None,
        )
        .context("failed to create recall config actor")?;

    tracing::debug!("Created recall config actor with ID: {}", recall_config::RECALL_CONFIG_ACTOR_ID);

    // Initialize the blob actor with delegated address for Ethereum/Solidity access
    // NOTE: State::new requires a concrete Blockstore type, but we only have a trait object.
    // We'll need to pass the actual blockstore or refactor State::new to work with trait objects.
    // For now, we use a workaround - the actual genesis code uses state.store() which is concrete.
    // TODO: This needs proper handling - may require GenesisState to expose the concrete store type
    let blobs_state = {
        // This is a temporary workaround - we're creating an empty state
        // The real implementation should pass the concrete blockstore
        use fvm_ipld_blockstore::MemoryBlockstore;
        fendermint_actor_storage_blobs::State::new(&MemoryBlockstore::default())?
    };

    // Calculate the Ethereum address for the blobs actor
    // This uses the builtin actor Ethereum address calculation
    let blobs_eth_addr = calculate_builtin_actor_eth_addr(blobs::BLOBS_ACTOR_ID);
    let blobs_f4_addr = fvm_shared::address::Address::from(blobs_eth_addr);

    state
        .create_custom_actor(
            fendermint_actor_storage_blobs::BLOBS_ACTOR_NAME,
            blobs::BLOBS_ACTOR_ID,
            &blobs_state,
            TokenAmount::zero(),
            Some(blobs_f4_addr),
        )
        .context("failed to create blobs actor")?;

    tracing::info!("Created storage blobs actor: ID={}, eth_addr={}", blobs::BLOBS_ACTOR_ID, blobs_eth_addr);

    // Initialize the blob reader actor
    let blob_reader_state = {
        // Same workaround as blobs - needs concrete blockstore
        use fvm_ipld_blockstore::MemoryBlockstore;
        fendermint_actor_storage_blob_reader::State::new(&MemoryBlockstore::default())?
    };

    state
        .create_custom_actor(
            fendermint_actor_storage_blob_reader::BLOB_READER_ACTOR_NAME,
            blob_reader::BLOB_READER_ACTOR_ID,
            &blob_reader_state,
            TokenAmount::zero(),
            None,
        )
        .context("failed to create blob reader actor")?;

    tracing::debug!("Created blob reader actor with ID: {}", blob_reader::BLOB_READER_ACTOR_ID);
    tracing::info!("Storage-node actors initialized successfully");

    Ok(())
}

/// Calculate the Ethereum address for a builtin actor.
///
/// This duplicates the logic from fendermint_vm_actor_interface::init::builtin_actor_eth_addr
/// to avoid circular dependencies. Based on EAM actor hash20 function.
fn calculate_builtin_actor_eth_addr(actor_id: fvm_shared::ActorID) -> fendermint_vm_actor_interface::eam::EthAddress {
    use fendermint_vm_actor_interface::eam::EthAddress;
    use multihash_codetable::{Code, MultihashDigest};

    // Convert actor ID to EthAddress representation
    let eth_addr = EthAddress::from_id(actor_id);

    // Hash it with Keccak256
    let hash = Code::Keccak256.digest(&eth_addr.0);

    // Take the last 20 bytes for final Ethereum address
    let eth_addr_bytes: [u8; 20] = hash.digest()[12..32].try_into().unwrap();

    EthAddress(eth_addr_bytes)
}

/// Get the actor IDs used by storage-node actors.
///
/// TODO: These should be defined in a shared constant location.
pub mod actor_ids {
    pub const RECALL_CONFIG_ACTOR_ID: u64 = 120;
    pub const BLOBS_ACTOR_ID: u64 = 121;
    pub const BLOB_READER_ACTOR_ID: u64 = 122;
}
