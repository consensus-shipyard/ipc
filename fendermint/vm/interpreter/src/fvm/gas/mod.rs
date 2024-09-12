// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm::executor::ApplyRet;
use fvm_shared::econ::TokenAmount;

pub mod actor;

pub type Gas = u64;

pub struct Available {
    pub block_gas: Gas,
}

pub struct CommitRet {
    pub base_fee: TokenAmount,
}

pub struct GasUtilization {
    gas_used: Gas,
    gas_premium: TokenAmount,
}

/// The gas market for fendermint. This should be backed by an fvm actor.
pub trait GasMarket {
    /// The constant parameters that determines the readings of gas market, such as block gas limit.
    type Constant;

    /// Update the constants of the gas market. If the gas market is actor based, then it's recommended
    /// to flush at EndBlock.
    #[allow(dead_code)]
    fn set_constants(&mut self, constants: Self::Constant);

    /// Obtain the current block gas available for execution
    fn available(&self) -> Available;

    /// Tracks the amount of gas consumed by a transaction
    fn record_utilization(&mut self, gas: GasUtilization);
}

impl From<&ApplyRet> for GasUtilization {
    fn from(ret: &ApplyRet) -> Self {
        Self {
            gas_used: ret.msg_receipt.gas_used,
            gas_premium: ret.miner_tip.clone(),
        }
    }
}
