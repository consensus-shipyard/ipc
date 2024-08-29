// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod actor;

pub type Gas = u64;

/// The gas market for fendermint. This should be backed by an fvm actor.
pub trait GasMarket {
    /// Obtain the current block gas available for execution
    fn available_block_gas(&self) -> Gas;

    /// Tracks the amount of gas consumed by a transaction
    fn record_gas_used(&mut self, gas: Gas) -> anyhow::Result<()>;
}
