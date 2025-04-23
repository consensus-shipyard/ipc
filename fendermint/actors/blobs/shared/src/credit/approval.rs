// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount};

use crate::credit::Credit;

/// A credit approval from one account to another.
#[derive(Debug, Default, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditApproval {
    /// Optional credit approval limit.
    pub credit_limit: Option<Credit>,
    /// Used to limit gas fee delegation.
    pub gas_allowance_limit: Option<TokenAmount>,
    /// Optional credit approval expiry epoch.
    pub expiry: Option<ChainEpoch>,
    /// Counter for how much credit has been used via this approval.
    pub credit_used: Credit,
    /// Used to track gas fees paid for by the delegation
    pub gas_allowance_used: TokenAmount,
}

impl CreditApproval {
    /// Returns a new credit approval.
    pub fn new(
        credit_limit: Option<Credit>,
        gas_allowance_limit: Option<TokenAmount>,
        expiry: Option<ChainEpoch>,
    ) -> Self {
        Self {
            credit_limit,
            gas_allowance_limit,
            expiry,
            ..Default::default()
        }
    }

    /// Validates whether the approval has enough allowance for the credit amount.
    pub fn validate_credit_usage(&self, amount: &TokenAmount) -> Result<(), ActorError> {
        if let Some(credit_limit) = self.credit_limit.as_ref() {
            let unused = &(credit_limit - &self.credit_used);
            if unused < amount {
                return Err(ActorError::forbidden(format!(
                    "usage would exceed approval credit limit (available: {}; required: {})",
                    unused, amount
                )));
            }
        }
        Ok(())
    }

    /// Validates whether the approval has enough allowance for the gas amount.
    pub fn validate_gas_usage(&self, amount: &TokenAmount) -> Result<(), ActorError> {
        if let Some(gas_limit) = self.gas_allowance_limit.as_ref() {
            let unused = &(gas_limit - &self.gas_allowance_used);
            if unused < amount {
                return Err(ActorError::forbidden(format!(
                    "usage would exceed approval gas allowance (available: {}; required: {})",
                    unused, amount
                )));
            }
        }
        Ok(())
    }

    /// Validates whether the approval has a valid expiration.
    pub fn validate_expiration(&self, current_epoch: ChainEpoch) -> Result<(), ActorError> {
        if let Some(expiry) = self.expiry {
            if expiry <= current_epoch {
                return Err(ActorError::forbidden("approval expired".into()));
            }
        }
        Ok(())
    }
}
