// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::fvm::gas::{Gas, GasMarket};
use crate::fvm::FvmMessage;
use anyhow::Context;

use fendermint_actor_gas_market::GasMarketReading;
use fendermint_vm_actor_interface::gas::GAS_MARKET_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm::executor::{ApplyKind, Executor};
use fvm_shared::clock::ChainEpoch;

#[derive(Default)]
pub struct ActorGasMarket {
    /// The block gas limit
    block_gas_limit: Gas,
    /// The accumulated gas usage so far
    block_gas_used: Gas,
}

impl GasMarket for ActorGasMarket {
    fn available_block_gas(&self) -> Gas {
        self.block_gas_limit - self.block_gas_used
    }

    fn record_gas_used(&mut self, gas: Gas) -> anyhow::Result<()> {
        if self.block_gas_used + gas >= self.block_gas_limit {
            tracing::warn!("out of block gas, should not have happened")
        }
        self.block_gas_used = self.block_gas_used.saturating_add(gas);

        Ok(())
    }
}

impl ActorGasMarket {
    pub fn new<E: Executor>(
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<ActorGasMarket> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
            method_num: fendermint_actor_gas_market::Method::CurrentGasReading as u64,
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
        })
    }

    pub fn commit<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<()> {
        let block_gas_used = self.block_gas_used;
        let params = fvm_ipld_encoding::RawBytes::serialize(
            fendermint_actor_gas_market::BlockGasUtilization { block_gas_used },
        )?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: fvm_shared::BLOCK_GAS_LIMIT,
            method_num: fendermint_actor_gas_market::Method::UpdateUtilization as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let apply_ret = executor.execute_message(msg, ApplyKind::Implicit, raw_length)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to update EIP1559 gas state: {}", err)
        } else {
            Ok(())
        }
    }
}
