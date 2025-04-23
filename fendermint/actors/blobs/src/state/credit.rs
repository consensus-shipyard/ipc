// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::credit::Credit;
use fvm_ipld_encoding::tuple::*;

mod approvals;
mod methods;
mod params;
#[cfg(test)]
mod tests;

pub use approvals::*;
pub use params::*;

/// Global credit-related state.
#[derive(Debug, Clone, Default, Serialize_tuple, Deserialize_tuple)]
pub struct Credits {
    /// The total number of credits sold in the subnet.
    pub credit_sold: Credit,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: Credit,
    /// The total number of credits debited in the subnet.
    pub credit_debited: Credit,
}
