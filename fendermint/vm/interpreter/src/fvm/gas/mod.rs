// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod actor;

pub type Gas = u64;

pub struct Available {
    pub block_gas: Gas,
}

/// The gas market for fendermint. This should be backed by an fvm actor.
pub trait GasMarket {
    /// The constant parameters that determines the readings of gas market, such as block gas limit.
    type Constant;

    #[allow(dead_code)]
    fn get_constants(&self) -> anyhow::Result<Self::Constant>;

    /// Update the constants of the gas market. If the gas market is actor based, then it's recommended
    /// to flush at EndBlock.
    #[allow(dead_code)]
    fn set_constants(&mut self, constants: Self::Constant);

    /// Obtain the current block gas available for execution
    fn available(&self) -> Available;

    /// Tracks the amount of gas consumed by a transaction
    fn record_utilization(&mut self, gas: Gas);
}
