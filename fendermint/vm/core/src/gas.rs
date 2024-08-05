// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub type Gas = u64;

/// Handles the gas modeling in the current blockchain
pub trait GasLayer {
    /// The state of the blockchain
    type State;

    /// Update the block gas limit
    fn set_block_gas_limit(&self, state: &mut Self::State);

    /// Obtain the current block gas limit
    fn block_gas_limit(&self, state: &Self::State) -> Gas;
}