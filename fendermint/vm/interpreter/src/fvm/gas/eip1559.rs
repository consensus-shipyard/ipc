use crate::fvm::gas::{Gas, GasMarket};
use crate::fvm::state::{read_actor_state, FvmExecState};
use crate::fvm::FvmMessage;
use anyhow::anyhow;
use fendermint_actor_gas_market::EIP1559GasState;
use fendermint_vm_actor_interface::gas::GAS_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm_ipld_blockstore::Blockstore;
use std::sync::atomic::{AtomicU64, Ordering};

type AtomicGas = AtomicU64;

/// The gas market based on EIP1155
#[derive(Default)]
pub struct EIP1559GasMarket {
    /// The block gas limit
    block_gas_limit: AtomicGas,
    /// The accumulated gas usage so far
    block_gas_used: AtomicGas,
}

impl GasMarket for EIP1559GasMarket {
    type State = EIP1559GasState;

    fn reload_from_chain<DB: Blockstore + Clone + 'static>(
        &self,
        chain_state: &FvmExecState<DB>,
    ) -> anyhow::Result<()> {
        let state = get_state(chain_state)?;
        self.block_gas_used.store(0, Ordering::SeqCst);
        self.block_gas_limit
            .store(state.block_gas_limit, Ordering::SeqCst);
        Ok(())
    }

    fn available_block_gas(&self) -> Gas {
        self.block_gas_limit.load(Ordering::SeqCst) - self.block_gas_used.load(Ordering::SeqCst)
    }

    fn consume_gas(&self, gas: Gas) -> anyhow::Result<()> {
        let block_gas_used = self.block_gas_used.load(Ordering::SeqCst);

        if block_gas_used + gas >= self.block_gas_limit.load(Ordering::SeqCst) {
            return Err(anyhow!("out of block gas"));
        }

        let new_gas_used = block_gas_used + gas;
        self.update_block_gas_used(block_gas_used, new_gas_used)
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

impl EIP1559GasMarket {
    fn update_block_gas_used(&self, old_used: Gas, new_used: Gas) -> anyhow::Result<()> {
        self.block_gas_used
            .compare_exchange(old_used, new_used, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| anyhow!("concurrent update in block gas used, should not happen"))?;
        Ok(())
    }
}

impl Clone for EIP1559GasMarket {
    fn clone(&self) -> Self {
        Self {
            block_gas_limit: AtomicGas::new(self.block_gas_limit.load(Ordering::SeqCst)),
            block_gas_used: AtomicGas::new(self.block_gas_used.load(Ordering::SeqCst)),
        }
    }
}

#[inline]
fn get_state<DB: Blockstore + Clone + 'static>(
    chain_state: &FvmExecState<DB>,
) -> anyhow::Result<EIP1559GasState> {
    read_actor_state::<EIP1559GasState, DB>(
        chain_state,
        fendermint_vm_actor_interface::gas::GAS_ACTOR_ID,
    )
}
