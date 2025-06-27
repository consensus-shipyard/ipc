// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::IsHumanReadable;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FvmSettings {
    /// Overestimation rate applied to gas estimations to ensure that the
    /// message goes through
    pub gas_overestimation_rate: f64,
    /// Gas search step increase used to find the optimal gas limit.
    /// It determines how fine-grained we want the gas estimation to be.
    pub gas_search_step: f64,

    /// Gas fee used when broadcasting transactions.
    #[serde_as(as = "IsHumanReadable")]
    pub gas_fee_cap: TokenAmount,
    /// Gas premium used when broadcasting transactions.
    #[serde_as(as = "IsHumanReadable")]
    pub gas_premium: TokenAmount,
}

impl Default for FvmSettings {
    fn default() -> Self {
        FvmSettings {
            gas_overestimation_rate: 1.25,
            gas_search_step: 1.25,
            gas_fee_cap: TokenAmount::from_atto(0),
            gas_premium: TokenAmount::from_atto(0),
        }
    }
}
