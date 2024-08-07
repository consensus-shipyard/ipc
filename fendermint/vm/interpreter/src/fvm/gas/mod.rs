// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::state::{read_actor_state, FvmExecState};
use crate::fvm::FvmMessage;
use anyhow::anyhow;
use fendermint_actor_gas_market::EIP1559GasState;
use fendermint_vm_actor_interface::gas::GAS_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm_ipld_blockstore::Blockstore;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

type AtomicGas = AtomicU64;
pub type Gas = u64;

pub struct EIP1559GasMarket<DB> {
    /// The total gas available to be used by transactions in the current block
    /// The executor requires Send + Sync, using an atomic variable instead of u64.
    available_block_gas: AtomicGas,
    _p: PhantomData<DB>,
}

impl<DB: Blockstore + Clone + 'static> EIP1559GasMarket<DB> {
    pub fn new() -> Self {
        Self {
            available_block_gas: AtomicGas::new(0),
            _p: Default::default(),
        }
    }

    pub fn reset_from_chain_state(&self, chain_state: &FvmExecState<DB>) -> anyhow::Result<()> {
        let current = self.available_block_gas.load(Ordering::SeqCst);
        let gas_state = get_gas_state(chain_state)?;
        self.atomic_set_block_gas_quota(current, gas_state.block_gas_limit)?;
        Ok(())
    }

    pub fn available_block_gas(&self) -> Gas {
        self.available_block_gas.load(Ordering::SeqCst)
    }

    pub fn deduct_available_block_gas(&self, gas: Gas) -> anyhow::Result<()> {
        let available = self.available_block_gas.load(Ordering::SeqCst);
        match available.checked_sub(gas) {
            Some(v) => {
                self.atomic_set_block_gas_quota(available, v)?;
                Ok(())
            }
            None => Err(anyhow!("out of block gas")),
        }
    }

    fn atomic_set_block_gas_quota(&self, old: Gas, new: Gas) -> anyhow::Result<()> {
        self.available_block_gas
            .compare_exchange(old, new, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| anyhow!("concurrent update to block gas available, should not happen"))?;
        Ok(())
    }
}

impl<DB> EIP1559GasMarket<DB>
where
    DB: Blockstore + Clone + 'static,
{
    #[allow(dead_code)]
    fn update_state(
        &self,
        blockchain_state: &mut FvmExecState<DB>,
        gas_state: EIP1559GasState,
    ) -> anyhow::Result<()> {
        let params = fvm_ipld_encoding::RawBytes::serialize(gas_state)?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_ACTOR_ADDR,
            sequence: blockchain_state.block_height() as u64,
            // exclude this from gas restriction
            gas_limit: u64::MAX,
            method_num: fendermint_actor_gas_market::Method::UpdateGasMarketState as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let (apply_ret, _) = blockchain_state.execute_implicit(msg)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to update EIP1559 gas state: {}", err)
        } else {
            Ok(())
        }
    }
}

fn get_gas_state<DB: Blockstore + Clone + 'static>(
    state: &FvmExecState<DB>,
) -> anyhow::Result<EIP1559GasState> {
    let s = read_actor_state::<EIP1559GasState, DB>(
        state,
        fendermint_vm_actor_interface::gas::GAS_ACTOR_ID,
    )?;
    Ok(s)
}

impl<DB> Clone for EIP1559GasMarket<DB> {
    fn clone(&self) -> Self {
        Self {
            available_block_gas: AtomicU64::new(self.available_block_gas.load(Ordering::SeqCst)),
            _p: Default::default(),
        }
    }
}
