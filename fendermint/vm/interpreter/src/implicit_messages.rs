use crate::fvm::state::FvmExecState;
use crate::types::*;
use anyhow::Context;
use fendermint_vm_actor_interface::{chainmetadata, cron, system};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::BLOCK_GAS_LIMIT;
use tendermint::consensus::params;

use crate::fvm::FvmMessage;

// Arbitrarily large gas limit for cron (matching how Forest does it, which matches Lotus).
// XXX: Our blocks are not necessarily expected to be 30 seconds apart, so the gas limit might be wrong.
const GAS_LIMIT: u64 = BLOCK_GAS_LIMIT * 10000;

/// Helper to build and execute an implicit system message.
/// It uses the default values for the other fields not passed.
fn execute_implicit_message<DB: Blockstore + Clone + 'static + Send + Sync>(
    state: &mut FvmExecState<DB>,
    from: Address,
    to: Address,
    sequence: u64,
    gas_limit: u64,
    method_num: u64,
    params: RawBytes,
) -> anyhow::Result<FvmApplyRet> {
    let msg = FvmMessage {
        from,
        to,
        sequence,
        gas_limit,
        method_num,
        params,
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let (apply_ret, emitters) = state.execute_implicit(msg)?;
    if let Some(err) = apply_ret.failure_info {
        anyhow::bail!("failed to apply system message: {}", err);
    }
    Ok(FvmApplyRet {
        apply_ret,
        emitters,
        from,
        to,
        method_num,
        gas_limit,
    })
}

/// Executes the cron message for the given block height.
pub fn execute_cron_message<DB: Blockstore + Clone + 'static + Send + Sync>(
    state: &mut FvmExecState<DB>,
    height: u64,
) -> anyhow::Result<FvmApplyRet> {
    let from = system::SYSTEM_ACTOR_ADDR;
    let to = cron::CRON_ACTOR_ADDR;
    let method_num = cron::Method::EpochTick as u64;
    let params = Default::default();

    execute_implicit_message(state, from, to, height, GAS_LIMIT, method_num, params)
        .context("failed to execute cron message")
}

/// Attempts to push chain metadata if a block hash is available.
pub fn maybe_push_chain_metadata<DB: Blockstore + Clone + 'static + Send + Sync>(
    state: &mut FvmExecState<DB>,
    height: u64,
) -> anyhow::Result<Option<FvmApplyRet>> {
    let from = system::SYSTEM_ACTOR_ADDR;
    let to = chainmetadata::CHAINMETADATA_ACTOR_ADDR;
    let method_num = fendermint_actor_chainmetadata::Method::PushBlockHash as u64;

    if let Some(block_hash) = state.block_hash() {
        let params = RawBytes::serialize(fendermint_actor_chainmetadata::PushBlockParams {
            // TODO Karel: this conversion from u64 to i64 should be revisited.
            epoch: height as i64,
            block: block_hash,
        })?;

        let fvm_apply_ret =
            execute_implicit_message(state, from, to, height, GAS_LIMIT, method_num, params)
                .context("failed to execute chainmetadata message")?;

        Ok(Some(fvm_apply_ret))
    } else {
        Ok(None)
    }
}
