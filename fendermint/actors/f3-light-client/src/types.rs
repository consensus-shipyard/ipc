// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Type definitions for the F3 Light Client actor.
//!
//! This module defines the core types used by the F3 Light Client actor,
//! including the light client state structure that tracks F3 finality
//! from the parent chain.

use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::clock::ChainEpoch;

/// F3 Light Client State - maintains verifiable parent finality from the parent chain.
///
/// This structure represents the essential state needed to track F3 finality:
/// - Latest Instance ID: The latest F3 instance that has been committed
/// - Latest Finalized Height: The highest epoch that has been finalized
/// - Power Table: Current validator power table (can change between instances)
///
/// This state is extracted from F3 certificates received from the parent chain
/// and stored by the actor for use in finality proofs.
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct LightClientState {
    /// Latest F3 instance ID that has been committed
    pub latest_instance_id: u64,
    /// The latest finalized height
    pub latest_finalized_height: ChainEpoch,
    /// Root CID of the on-chain power table (HAMT).
    ///
    /// The actual entries are stored in the actor's blockstore and reachable from this root.
    pub power_table_root: Cid,
}

/// Power table entry for F3 consensus
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct PowerEntry {
    /// Validator ID (from F3 power table)
    pub id: u64,
    /// Public key of the validator
    pub public_key: Vec<u8>,
    /// Voting power of the validator, encoded as unsigned big-endian bytes.
    ///
    /// Filecoin power values can exceed 64 bits; storing bytes avoids lossy conversions.
    /// `[]` represents zero.
    pub power_be: Vec<u8>,
}

/// Constructor parameters for the F3 light client actor
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct ConstructorParams {
    /// Initial F3 instance ID (from genesis)
    pub latest_instance_id: u64,
    /// Initial power table (from genesis)
    pub power_table: Vec<PowerEntry>,
    /// Initial finalized height (set from genesis base epoch)
    pub latest_finalized_height: ChainEpoch,
}

/// Parameters for updating the light client state
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct UpdateStateParams {
    /// Latest F3 instance ID that has been committed
    pub latest_instance_id: u64,
    /// The latest finalized height
    pub latest_finalized_height: ChainEpoch,
    /// New power table entries for this instance (authoritative).
    pub power_table: Vec<PowerEntry>,
}

/// Response containing the current light client state
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct GetStateResponse {
    /// Latest F3 instance ID that has been committed
    pub latest_instance_id: u64,
    /// The latest finalized height
    pub latest_finalized_height: ChainEpoch,
    /// Root CID of the on-chain power table (HAMT).
    pub power_table_root: Cid,
    /// Current power table (materialized).
    ///
    /// This is derived from `power_table_root` for convenience.
    pub power_table: Vec<PowerEntry>,
}
