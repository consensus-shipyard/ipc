// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, econ::TokenAmount};

use crate::credit::Credit;

/// Credit allowance for an account.
#[derive(Debug, Default, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditAllowance {
    /// The amount from the account.
    pub amount: Credit,
    /// The account's default sponsor.
    pub sponsor: Option<Address>,
    /// The amount from the account's default sponsor.
    pub sponsored_amount: Credit,
}

impl CreditAllowance {
    /// Returns the total allowance from self and default sponsor.
    pub fn total(&self) -> Credit {
        &self.amount + &self.sponsored_amount
    }
}

/// Gas allowance for an account.
#[derive(Debug, Default, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct GasAllowance {
    /// The amount from the account.
    pub amount: TokenAmount,
    /// The account's default sponsor.
    pub sponsor: Option<Address>,
    /// The amount from the account's default sponsor.
    pub sponsored_amount: TokenAmount,
}

impl GasAllowance {
    /// Returns the total allowance from self and default sponsor.
    pub fn total(&self) -> TokenAmount {
        &self.amount + &self.sponsored_amount
    }
}
