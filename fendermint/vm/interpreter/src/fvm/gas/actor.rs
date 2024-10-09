// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::gas::{Available, CommitRet, Gas, GasMarket, GasUtilization};
use crate::fvm::FvmMessage;
use anyhow::Context;

use fendermint_actor_gas_market_eip1559::{SetConstants};
use fendermint_crypto::PublicKey;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::gas_market::GAS_MARKET_ACTOR_ADDR;
use fendermint_vm_actor_interface::{reward, system};
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::METHOD_SEND;
use fendermint_actors_api::gas_market::{Reading, Utilization};

#[derive(Default)]
pub struct ActorGasMarket {
    /// The base fee for fvm
    base_fee: TokenAmount,
    /// The total gas premium for the miner
    gas_premium: TokenAmount,
    /// The block gas limit
    block_gas_limit: Gas,
    /// The accumulated gas usage so far
    block_gas_used: Gas,
    /// Pending update to the underlying gas actor
    constant_update: Option<SetConstants>,
}

impl GasMarket for ActorGasMarket {
    type Constant = SetConstants;

    fn set_constants(&mut self, constants: Self::Constant) {
        self.constant_update = Some(constants);
    }

    fn available(&self) -> Available {
        Available {
            block_gas: self.block_gas_limit - self.block_gas_used.min(self.block_gas_limit),
        }
    }

    fn record_utilization(&mut self, utilization: GasUtilization) {
        self.gas_premium += utilization.gas_premium;
        self.block_gas_used += utilization.gas_used;

        // sanity check
        if self.block_gas_used >= self.block_gas_limit {
            tracing::warn!("out of block gas, vm execution more than available gas limit");
        }
    }
}

impl ActorGasMarket {
    pub fn current_reading<E: Executor>(
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<Reading> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_gas_market_eip1559::Method::CurrentReading as u64,
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

        let r = fvm_ipld_encoding::from_slice::<Reading>(&apply_ret.msg_receipt.return_data)
            .context("failed to parse gas market readying")?;
        Ok(r)
    }

    pub fn create<E: Executor>(
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<ActorGasMarket> {
        let reading = Self::current_reading(executor, block_height)?;
        Ok(Self {
            base_fee: reading.base_fee,
            gas_premium: TokenAmount::from_atto(0),
            block_gas_limit: reading.block_gas_limit,
            block_gas_used: 0,
            constant_update: None,
        })
    }

    pub fn take_constant_update(&mut self) -> Option<SetConstants> {
        self.constant_update.take()
    }

    pub fn commit<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
        validator: Option<PublicKey>,
    ) -> anyhow::Result<CommitRet> {
        self.distribute_reward(executor, block_height, validator)?;
        self.commit_constants(executor, block_height)?;
        self.commit_utilization(executor, block_height)
    }

    fn distribute_reward<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
        validator: Option<PublicKey>,
    ) -> anyhow::Result<()> {
        if validator.is_none() || self.gas_premium.is_zero() {
            return Ok(());
        }

        let validator = validator.unwrap();
        let validator = Address::from(EthAddress::new_secp256k1(&validator.serialize())?);

        let msg = FvmMessage {
            from: reward::REWARD_ACTOR_ADDR,
            to: validator,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: METHOD_SEND,
            params: fvm_ipld_encoding::RawBytes::default(),
            value: self.gas_premium.clone(),

            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };
        self.apply_implicit_message(msg, executor)?;

        Ok(())
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
            method_num: fendermint_actor_gas_market_eip1559::Method::SetConstants as u64,
            params: fvm_ipld_encoding::RawBytes::serialize(constants)?,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };
        self.apply_implicit_message(msg, executor)?;

        Ok(())
    }

    fn commit_utilization<E: Executor>(
        &self,
        executor: &mut E,
        block_height: ChainEpoch,
    ) -> anyhow::Result<CommitRet> {
        let block_gas_used = self.block_gas_used.min(self.block_gas_limit);
        let params = fvm_ipld_encoding::RawBytes::serialize(
            Utilization { block_gas_used },
        )?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_MARKET_ACTOR_ADDR,
            sequence: block_height as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_gas_market_eip1559::Method::UpdateUtilization as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = self.apply_implicit_message(msg, executor)?;
        let r = fvm_ipld_encoding::from_slice::<Reading>(&apply_ret.msg_receipt.return_data)
            .context("failed to parse gas utilization result")?;
        Ok(CommitRet {
            base_fee: r.base_fee,
        })
    }

    fn apply_implicit_message<E: Executor>(
        &self,
        msg: FvmMessage,
        executor: &mut E,
    ) -> anyhow::Result<ApplyRet> {
        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let apply_ret = executor.execute_message(msg, ApplyKind::Implicit, raw_length)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to update EIP1559 gas state: {}", err)
        } else {
            Ok(apply_ret)
        }
    }
}
