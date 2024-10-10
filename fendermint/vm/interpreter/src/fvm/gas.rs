// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::FvmMessage;
use anyhow::Context;

use fendermint_actor_gas_market_eip1559::SetConstants;
use fendermint_actors_api::gas_market::{Gas, Reading, Utilization};
use fendermint_crypto::PublicKey;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::gas_market::GAS_MARKET_ACTOR_ADDR;
use fendermint_vm_actor_interface::{reward, system};
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::METHOD_SEND;
use num_traits::Zero;

pub struct BlockGasTracker {
    /// The address of the actor that will receive the gas premium. Usually the block producer.
    premium_recipient: Address,
    /// The current base fee.
    base_fee: TokenAmount,
    /// The current block gas limit.
    block_gas_limit: Gas,
    /// The cumulative gas premiums claimable by the block producer.
    cumul_gas_premium: TokenAmount,
    /// The accumulated gas usage throughout the block.
    cumul_gas_used: Gas,
}

impl BlockGasTracker {
    pub fn create<E: Executor>(
        executor: &mut E,
        premium_recipient: Address,
    ) -> anyhow::Result<BlockGasTracker> {
        let mut ret = Self {
            premium_recipient,
            base_fee: Zero::zero(),
            block_gas_limit: Zero::zero(),
            cumul_gas_premium: Zero::zero(),
            cumul_gas_used: Zero::zero(),
        };

        ret.load_reading(executor)?;

        Ok(ret)
    }

    pub fn available(&self) -> Gas {
        self.block_gas_limit.saturating_sub(self.cumul_gas_used)
    }

    pub fn record_utilization(&mut self, ret: &ApplyRet) {
        self.cumul_gas_premium += ret.miner_tip.clone();
        self.cumul_gas_used = self.cumul_gas_used.saturating_add(ret.msg_receipt.gas_used);

        // sanity check, should not happen; only trace if it does so we can debug later.
        if self.cumul_gas_used >= self.block_gas_limit {
            tracing::warn!("out of block gas; cumulative gas used exceeds block gas limit!");
        }
    }

    pub fn finalize<E: Executor>(&self, executor: &mut E) -> anyhow::Result<TokenAmount> {
        self.distribute_premiums(executor)
            .and_then(|_| self.commit_utilization(executor))
    }

    fn load_reading<E: Executor>(&mut self, executor: &mut E) -> anyhow::Result<()> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: 0, // irrelevant for implicit executions.
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actors_api::gas_market::Method::CurrentReading as u64,
            params: fvm_ipld_encoding::RawBytes::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = Self::apply_implicit_message(executor, msg)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to acquire gas market reading: {}", err);
        }

        let reading = fvm_ipld_encoding::from_slice::<Reading>(&apply_ret.msg_receipt.return_data)
            .context("failed to parse gas market reading")?;

        self.base_fee = reading.base_fee;
        self.block_gas_limit = reading.block_gas_limit;

        Ok(())
    }

    fn commit_utilization<E: Executor>(&self, executor: &mut E) -> anyhow::Result<TokenAmount> {
        let params = fvm_ipld_encoding::RawBytes::serialize(Utilization {
            block_gas_used: self.cumul_gas_used,
        })?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: 0, // irrelevant for implicit executions.
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actors_api::gas_market::Method::UpdateUtilization as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = Self::apply_implicit_message(executor, msg)?;
        fvm_ipld_encoding::from_slice::<Reading>(&apply_ret.msg_receipt.return_data)
            .map(|r| r.base_fee)
            .context("failed to parse gas utilization result")
    }

    fn distribute_premiums<E: Executor>(&self, executor: &mut E) -> anyhow::Result<()> {
        if self.cumul_gas_premium.is_zero() {
            return Ok(());
        }

        let msg = FvmMessage {
            from: reward::REWARD_ACTOR_ADDR,
            to: self.premium_recipient.clone(),
            sequence: 0, // irrelevant for implicit executions.
            gas_limit: i64::MAX as u64,
            method_num: METHOD_SEND,
            params: fvm_ipld_encoding::RawBytes::default(),
            value: self.cumul_gas_premium.clone(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };
        Self::apply_implicit_message(executor, msg)?;

        Ok(())
    }

    fn apply_implicit_message<E: Executor>(
        executor: &mut E,
        msg: FvmMessage,
    ) -> anyhow::Result<ApplyRet> {
        let apply_ret = executor.execute_message(msg, ApplyKind::Implicit, 0)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to apply message: {}", err)
        }
        Ok(apply_ret)
    }
}
