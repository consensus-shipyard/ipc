// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::checkpoint::Validators;
use fil_actors_runtime::runtime::Runtime;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;

impl Validators {
    /// Get the weight of a validator
    /// It expects ID addresses as an input
    pub fn get_validator_weight(&self, rt: &impl Runtime, addr: &Address) -> Option<TokenAmount> {
        self.validators
            .validators()
            .iter()
            .find(|x| match rt.resolve_address(&x.addr) {
                Some(id) => id == *addr,
                None => false,
            })
            .map(|v| v.weight.clone())
    }
}
