// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;

pub use crate::state::State;

pub const BLOBS_ACTOR_NAME: &str = "blobs";
/// The default total storage capacity of the subnet.
pub const DEFAULT_BLOB_CAPACITY: u64 = 0;
/// The default byte-blocks per atto token rate.
pub const DEFAULT_BLOB_CREDIT_DEBIT_RATE: u64 = 1;

/// Params for actor construction.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// The total storage capacity of the subnet.
    pub blob_capacity: u64,
    /// The byte-blocks per atto token rate.
    pub blob_credit_debit_rate: u64,
}
