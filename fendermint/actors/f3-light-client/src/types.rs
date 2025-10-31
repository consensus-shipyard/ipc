// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Type definitions for the F3 Light Client actor.
//!
//! This module defines the core types used by the F3 Light Client actor,
//! including the light client state structure that tracks F3 finality
//! from the parent chain.

use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::clock::ChainEpoch;

/// F3 Light Client State - maintains verifiable parent finality from the parent chain.
///
/// This structure represents the essential state needed to track F3 finality:
/// - Instance ID: The current F3 instance (can increment during protocol upgrades)
/// - Finalized Epochs: Complete chain of finalized epochs (not just the latest)
/// - Power Table: Current validator power table (can change between instances)
///
/// This state is extracted from F3 certificates received from the parent chain
/// and stored by the actor for use in finality proofs.
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct LightClientState {
    /// Current F3 instance ID
    pub instance_id: u64,
    /// Finalized chain - full list of finalized epochs
    /// Matches ECChain from F3 certificates
    /// Empty initially at genesis until first update
    pub finalized_epochs: Vec<ChainEpoch>,
    /// Current power table for this instance
    /// Power table can change between instances
    pub power_table: Vec<PowerEntry>,
}

/// Power table entry for F3 consensus
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct PowerEntry {
    /// Public key of the validator
    pub public_key: Vec<u8>,
    /// Voting power of the validator
    pub power: u64,
}

/// Constructor parameters for the F3 light client actor
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct ConstructorParams {
    /// Initial F3 instance ID (from genesis)
    pub instance_id: u64,
    /// Initial power table (from genesis)
    pub power_table: Vec<PowerEntry>,
    /// Initial finalized epochs (from genesis certificate)
    pub finalized_epochs: Vec<ChainEpoch>,
}

/// Parameters for updating the light client state
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct UpdateStateParams {
    /// New light client state to store
    pub state: LightClientState,
}

/// Response containing the current light client state
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct GetStateResponse {
    /// Current F3 instance ID
    pub instance_id: u64,
    /// Finalized chain - full list of finalized epochs (ordered)
    pub finalized_epochs: Vec<ChainEpoch>,
    /// Current power table
    pub power_table: Vec<PowerEntry>,
}
