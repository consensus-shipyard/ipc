// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Result;

use crate::fvm::{
    observe::{MsgExec, MsgExecPurpose},
    state::FvmQueryState,
};
use fendermint_vm_message::query::GasEstimate;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{self, RawBytes};
use fvm_shared::{
    bigint::BigInt, econ::TokenAmount, error::ExitCode, message::Message,
};

// BLOCK_GAS_LIMIT was removed in FVM 4.7, define locally for IPC
const BLOCK_GAS_LIMIT: u64 = 10_000_000_000;
use ipc_observability::emit;
use num_traits::Zero;
use std::time::Instant;

/// Estimates the gas for a given message.
pub async fn estimate_gassed_msg<DB: Blockstore + Clone + 'static + Send + Sync>(
    state: FvmQueryState<DB>,
    msg: &mut Message,
    gas_overestimation_rate: f64,
) -> Result<(FvmQueryState<DB>, Option<GasEstimate>)> {
    msg.gas_limit = BLOCK_GAS_LIMIT;
    let gas_premium = msg.gas_premium.clone();
    let gas_fee_cap = msg.gas_fee_cap.clone();
    msg.gas_premium = TokenAmount::zero();
    msg.gas_fee_cap = TokenAmount::zero();

    let start = Instant::now();
    let (state, (ret, _)) = state.call(msg.clone()).await?;
    let latency = start.elapsed().as_secs_f64();

    emit(MsgExec {
        purpose: MsgExecPurpose::Estimate,
        height: state.block_height(),
        message: msg.clone(),
        duration: latency,
        exit_code: ret.msg_receipt.exit_code.value(),
    });

    if !ret.msg_receipt.exit_code.is_success() {
        return Ok((
            state,
            Some(GasEstimate {
                exit_code: ret.msg_receipt.exit_code,
                info: ret.failure_info.map(|x| x.to_string()).unwrap_or_default(),
                return_data: ret.msg_receipt.return_data,
                gas_limit: 0,
            }),
        ));
    }

    msg.gas_limit = (ret.msg_receipt.gas_used as f64 * gas_overestimation_rate) as u64;

    msg.gas_premium = if gas_premium.is_zero() {
        TokenAmount::from_nano(BigInt::from(1))
    } else {
        gas_premium
    };

    msg.gas_fee_cap = if gas_fee_cap.is_zero() {
        msg.gas_premium.clone()
    } else {
        gas_fee_cap
    };

    Ok((state, None))
}

/// Searches for a valid gas limit for the message by iterative estimation.
pub async fn gas_search<DB: Blockstore + Clone + 'static + Send + Sync>(
    mut state: FvmQueryState<DB>,
    msg: &Message,
    gas_search_step: f64,
) -> Result<(FvmQueryState<DB>, GasEstimate)> {
    let mut curr_limit = msg.gas_limit;

    loop {
        let (st, est) = estimation_call_with_limit(state, msg.clone(), curr_limit).await?;

        if let Some(est) = est {
            return Ok((st, est));
        } else {
            state = st;
        }

        curr_limit = (curr_limit as f64 * gas_search_step) as u64;
        if curr_limit > BLOCK_GAS_LIMIT {
            let est = GasEstimate {
                exit_code: ExitCode::OK,
                info: String::new(),
                return_data: RawBytes::default(),
                gas_limit: BLOCK_GAS_LIMIT,
            };
            return Ok((state, est));
        }
    }
}

/// Helper for making an estimation call with a specific gas limit.
async fn estimation_call_with_limit<DB: Blockstore + Clone + 'static + Send + Sync>(
    state: FvmQueryState<DB>,
    mut msg: Message,
    limit: u64,
) -> Result<(FvmQueryState<DB>, Option<GasEstimate>)> {
    msg.gas_limit = limit;
    msg.sequence = 0; // Reset nonce

    let start = Instant::now();
    let (state, (apply_ret, _)) = state.call(msg.clone()).await?;
    let latency = start.elapsed().as_secs_f64();

    let ret = GasEstimate {
        exit_code: apply_ret.msg_receipt.exit_code,
        info: apply_ret
            .failure_info
            .map(|x| x.to_string())
            .unwrap_or_default(),
        return_data: apply_ret.msg_receipt.return_data,
        gas_limit: apply_ret.msg_receipt.gas_used,
    };

    emit(MsgExec {
        purpose: MsgExecPurpose::Estimate,
        height: state.block_height(),
        message: msg,
        duration: latency,
        exit_code: ret.exit_code.value(),
    });

    if ret.exit_code == ExitCode::OK || ret.exit_code != ExitCode::SYS_OUT_OF_GAS {
        return Ok((state, Some(ret)));
    }

    Ok((state, None))
}
