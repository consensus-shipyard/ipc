// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_gas::Gas;

pub(crate) mod default;

/// Handles the gas modeling in the current blockchain
pub trait GasLayer {
    /// The state of the blockchain
    type State;

    /// Update the block gas limit. This will only take effect in the next block.
    #[allow(dead_code)]
    fn set_block_gas_limit(&self, state: &mut Self::State, limit: Gas) -> anyhow::Result<()>;

    /// Obtain the current block gas limit
    fn block_gas_limit(&self, state: &Self::State) -> anyhow::Result<Gas>;
}
