// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::state::FvmExecState;
use fvm_ipld_blockstore::Blockstore;

pub mod eip1559;

pub type Gas = u64;

/// The gas market for fendermint. This should be backed by an fvm actor.
pub trait GasMarket {
    /// The gas market state
    type State;

    /// Reset the gas market based on the current block chain state
    fn reload_from_chain<DB: Blockstore + Clone + 'static>(
        &self,
        chain_state: &FvmExecState<DB>,
    ) -> anyhow::Result<()>;

    /// Obtain the current block gas available for execution
    fn available_block_gas(&self) -> Gas;

    /// Tracks the amount of gas consumed by a transaction
    fn consume_gas(&self, gas: Gas) -> anyhow::Result<()>;

    /// Update the gas market params to blockchain state. This usually happens at the end of the block
    fn update_params<DB: Blockstore + Clone + 'static>(
        &self,
        chain_state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<()>;
}
