// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;

pub use crate::state::State;

pub const BLOBS_ACTOR_NAME: &str = "blobs";

/// Params for actor construction.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    // The total storage capacity of the subnet.
    pub capacity: u64,
    /// The byte-blocks per atto token rate.
    pub debit_rate: u64,
}
