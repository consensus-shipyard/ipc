// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Genesis initialization for storage-node actors.
//!
//! This module provides the logic to initialize storage-node actors during genesis.
//! The actual implementation requires access to FvmGenesis methods that are not yet
//! exposed through the GenesisState trait.

use anyhow::Result;
use fendermint_module::genesis::GenesisState;
use fendermint_vm_genesis::Genesis;

/// Initialize storage-node actors in genesis.
///
/// TODO: This is a placeholder implementation. The full implementation needs:
/// 1. Access to `create_custom_actor` method (currently only on FvmGenesis)
/// 2. Actor ID constants to be defined in a shared location
/// 3. Proper Ethereum address calculation for blobs actor
///
/// The actual initialization code is currently in:
/// `fendermint/vm/interpreter/src/genesis.rs` lines 406-448 behind `#[cfg(feature = "storage-node")]`
pub fn initialize_storage_actors<S: GenesisState>(
    _state: &mut S,
    _genesis: &Genesis,
) -> Result<()> {
    tracing::info!("Storage-node genesis initialization called");

    // TODO: Implement actor initialization when GenesisState trait is extended
    // The storage actors to initialize are:
    // - recall_config (storage_config actor)
    // - blobs (storage_blobs actor)
    // - blob_reader (storage_blob_reader actor)

    tracing::warn!("Storage-node genesis initialization is not yet fully implemented in plugin");
    tracing::warn!("Actor initialization still happens in fendermint/vm/interpreter/src/genesis.rs");

    Ok(())
}

/// Get the actor IDs used by storage-node actors.
///
/// TODO: These should be defined in a shared constant location.
pub mod actor_ids {
    pub const RECALL_CONFIG_ACTOR_ID: u64 = 120;
    pub const BLOBS_ACTOR_ID: u64 = 121;
    pub const BLOB_READER_ACTOR_ID: u64 = 122;
}
