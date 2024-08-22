// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::gas::{Available, Gas, GasMarket};
use crate::fvm::FvmMessage;
use anyhow::Context;

use crate::fvm::cometbft::ConsensusBlockUpdate;
use fendermint_actor_gas_market::{GasMarketReading, SetConstants};
use fendermint_vm_actor_interface::gas::GAS_MARKET_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::clock::ChainEpoch;

#[derive(Default)]
pub struct ActorGasMarket {
    /// The block gas limit
    block_gas_limit: Gas,
    /// The accumulated gas usage so far
    block_gas_used: Gas,
    /// Pending update to the underlying gas actor
    constant_update: Option<SetConstants>,
}

impl GasMarket for ActorGasMarket {
    type Constant = SetConstants;

    fn get_constants(&self) -> anyhow::Result<Self::Constant> {
        todo!()
    }

    fn set_constants(&mut self, constants: Self::Constant) {
        self.constant_update = Some(constants);
    }

    fn available(&self) -> Available {
        Available {
            block_gas: self.block_gas_limit - self.block_gas_used.min(self.block_gas_limit),
        }
    }

    fn record_utilization(&mut self, gas: Gas) {
        self.block_gas_used += gas;

        // sanity check
        if self.block_gas_used >= self.block_gas_limit {
            tracing::warn!("out of block gas, vm execution more than available gas limit");
        }
    }
}

impl ActorGasMarket {
    pub fn create<E: Executor>(
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<ActorGasMarket> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_gas_market::Method::CurrentReading as u64,
            params: fvm_ipld_encoding::RawBytes::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let apply_ret = executor.execute_message(msg, ApplyKind::Implicit, raw_length)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to read gas market state: {}", err);
        }

        let reading =
            fvm_ipld_encoding::from_slice::<GasMarketReading>(&apply_ret.msg_receipt.return_data)
                .context("failed to parse gas market readying")?;

        Ok(Self {
            block_gas_limit: reading.block_gas_limit,
            block_gas_used: 0,
            constant_update: None,
        })
    }

    pub fn process_consensus_update(&self, update: &mut ConsensusBlockUpdate) {
        if let Some(ref set_constant) = self.constant_update {
            update.process_block_size(set_constant.block_gas_limit);
        }
    }

    pub fn commit<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<()> {
        self.commit_constants(executor, block_height)?;
        self.commit_utilization(executor, block_height)
    }

    fn commit_constants<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<()> {
        let Some(ref constants) = self.constant_update else {
            return Ok(());
        };

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_gas_market::Method::SetConstants as u64,
            params: fvm_ipld_encoding::RawBytes::serialize(constants)?,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };
        self.call_fvm(msg, executor)?;

        Ok(())
    }

    fn commit_utilization<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<()> {
        let block_gas_used = self.block_gas_used.min(self.block_gas_limit);
        let params = fvm_ipld_encoding::RawBytes::serialize(
            fendermint_actor_gas_market::BlockGasUtilization { block_gas_used },
        )?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_gas_market::Method::UpdateUtilization as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        self.call_fvm(msg, executor)?;
        Ok(())
    }

    fn call_fvm<E: Executor>(&self, msg: FvmMessage, executor: &mut E) -> anyhow::Result<ApplyRet> {
        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let apply_ret = executor.execute_message(msg, ApplyKind::Implicit, raw_length)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to update EIP1559 gas state: {}", err)
        } else {
            Ok(apply_ret)
        }
    }
}
