use crate::fvm::gas::{Gas, GasMarket};
use crate::fvm::state::FvmExecState;
use crate::fvm::FvmMessage;
use anyhow::Context;

use fendermint_vm_actor_interface::gas::GAS_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::BytesDe;
use std::sync::atomic::{AtomicU64, Ordering};

type AtomicGas = AtomicU64;
type GasMarketState = fendermint_actor_gas_market::EIP1559GasState;

/// The gas market based on EIP1155
/// Due to the reference trait bound limit (`&self` instead of `&mut self`) in Interpreter, `Atomic`
/// is used. However, the calling pattern should be single threaded, so direct `store` could be used.
/// The usage of `Atomic` is purely to bypass the compilation issue without using unsafe.
/// TODO: remove this overhead when trait bound is updated.
#[derive(Default)]
pub struct ActorGasMarket {
    /// The block gas limit
    block_gas_limit: AtomicGas,
    /// The accumulated gas usage so far
    block_gas_used: AtomicGas,
}

impl GasMarket for ActorGasMarket {
    fn available_block_gas(&self) -> Gas {
        self.block_gas_limit.load(Ordering::SeqCst) - self.block_gas_used.load(Ordering::SeqCst)
    }

    fn consume_gas(&self, gas: Gas) -> anyhow::Result<()> {
        let block_gas_used = self.block_gas_used.load(Ordering::SeqCst);

        if block_gas_used + gas >= self.block_gas_limit.load(Ordering::SeqCst) {
            anyhow::bail!("out of block gas")
        }
        self.block_gas_used
            .store(block_gas_used + gas, Ordering::SeqCst);

        Ok(())
    }

    fn update_params<DB: Blockstore + Clone + 'static>(
        &self,
        chain_state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<()> {
        let block_gas_used = self.block_gas_used.load(Ordering::SeqCst);
        let params = fvm_ipld_encoding::RawBytes::serialize(block_gas_used)?;

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_ACTOR_ADDR,
            sequence: chain_state.block_height() as u64,
            // exclude this from gas restriction
            gas_limit: u64::MAX,
            method_num: fendermint_actor_gas_market::Method::UpdateBlockGasConsumption as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let (apply_ret, _) = chain_state.execute_implicit(msg)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to update EIP1559 gas state: {}", err)
        } else {
            Ok(())
        }
    }
}

impl ActorGasMarket {
    pub fn load<DB: Blockstore + Clone + 'static>(
        &self,
        chain_state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<()> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: GAS_ACTOR_ADDR,
            sequence: chain_state.block_height() as u64,
            // exclude this from gas restriction
            gas_limit: u64::MAX,
            method_num: fendermint_actor_gas_market::Method::GetState as u64,
            params: fvm_ipld_encoding::RawBytes::serialize(())?,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let (apply_ret, _) = chain_state.execute_implicit(msg)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to read gas market state: {}", err);
        }

        let output = apply_ret
            .msg_receipt
            .return_data
            .deserialize::<BytesDe>()
            .map(|bz| bz.0)
            .context("failed to deserialize error data")?;
        let state = fvm_ipld_encoding::from_slice::<GasMarketState>(&output)?;

        self.block_gas_used.store(0, Ordering::SeqCst);
        self.block_gas_limit
            .store(state.block_gas_limit, Ordering::SeqCst);
        Ok(())
    }
}

impl Clone for ActorGasMarket {
    fn clone(&self) -> Self {
        Self {
            block_gas_limit: AtomicGas::new(self.block_gas_limit.load(Ordering::SeqCst)),
            block_gas_used: AtomicGas::new(self.block_gas_used.load(Ordering::SeqCst)),
        }
    }
}
