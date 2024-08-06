// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::gas::GasLayer;
use crate::fvm::state::{read_actor_state, FvmExecState};
use crate::fvm::FvmMessage;
use anyhow::anyhow;
use fendermint_actor_gas::{Gas, State};
use fendermint_vm_actor_interface::gas::GAS_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm_ipld_blockstore::Blockstore;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

type AtomicGas = AtomicU64;

pub struct DefaultGas<DB> {
    /// The total gas available to be used by transactions in the current block
    /// The executor requires Send + Sync, using an atomic variable instead of u64.
    block_gas_quota: AtomicGas,
    p: PhantomData<DB>,
}

impl<DB: Blockstore + Clone + 'static> DefaultGas<DB> {
    pub fn new() -> Self {
        Self {
            block_gas_quota: AtomicGas::new(0),
            p: Default::default(),
        }
    }

    pub fn reset_block_gas_quota(&self, state: &mut <Self as GasLayer>::State) -> anyhow::Result<()> {
        let old_limit = self.block_gas_quota.load(Ordering::SeqCst);
        self.atomic_set_block_gas_quota(old_limit, self.block_gas_limit(state)?)?;
        Ok(())
    }

    pub fn available_block_gas(&self) -> Gas {
        self.block_gas_quota.load(Ordering::SeqCst)
    }

    pub fn deduct_block_gas_quota(&self, gas: Gas) -> anyhow::Result<()> {
        let quota = self.block_gas_quota.load(Ordering::SeqCst);
        match quota.checked_sub(gas) {
            Some(v) => {
                self.atomic_set_block_gas_quota(quota, v)?;
                Ok(())
            }
            None => Err(anyhow!("out of block gas")),
        }
    }

    fn atomic_set_block_gas_quota(&self, old: Gas, new: Gas) -> anyhow::Result<()> {
        self.block_gas_quota.compare_exchange(
            old,
            new,
            Ordering::SeqCst,
            Ordering::SeqCst
        )
            .map_err(|_| anyhow!("concurrent update to block gas available, should not happen"))?;
        Ok(())
    }
}

impl<DB> GasLayer for DefaultGas<DB>
where
    DB: Blockstore + Clone + 'static,
{
    type State = FvmExecState<DB>;

    fn set_block_gas_limit(&self, state: &mut Self::State, limit: Gas) -> anyhow::Result<()> {
        let params = fvm_ipld_encoding::RawBytes::serialize(limit)?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_ACTOR_ADDR,
            sequence: state.block_height() as u64,
            // exclude this from gas restriction
            gas_limit: u64::MAX,
            method_num: fendermint_actor_gas::Method::SetBlockGasLimit as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let (apply_ret, _) = state.execute_implicit(msg)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to update block gas limit: {}", err)
        } else {
            Ok(())
        }
    }

    fn block_gas_limit(&self, state: &Self::State) -> anyhow::Result<Gas> {
        let s =
            read_actor_state::<State, DB>(state, fendermint_vm_actor_interface::gas::GAS_ACTOR_ID)?;
        Ok(s.block_gas_limit())
    }
}

impl <DB> Clone for DefaultGas<DB> {
    fn clone(&self) -> Self {
        Self { block_gas_quota: AtomicU64::new(self.block_gas_quota.load(Ordering::SeqCst)), p: Default::default() }
    }
}