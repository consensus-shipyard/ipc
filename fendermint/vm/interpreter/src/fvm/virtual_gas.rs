// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{bail, Context};
use fendermint_actor_blobs_shared::params::{DebitCreditParams, GetGasAllowanceParams};
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;

use crate::fvm::FvmMessage;

/// Returns the virtual gas allowance for the sender.
pub(crate) fn get_vgas_allowance<E: Executor>(
    executor: &mut E,
    sender: Address,
) -> anyhow::Result<TokenAmount> {
    let params = fvm_ipld_encoding::RawBytes::serialize(GetGasAllowanceParams { sender })?;

    let msg = FvmMessage {
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0, // irrelevant for implicit executions.
        gas_limit: i64::MAX as u64,
        method_num: fendermint_actor_blobs_shared::Method::GetGasAllowance as u64,
        params,
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let apply_ret = apply_implicit_message(executor, msg)?;

    if let Some(err) = apply_ret.failure_info {
        bail!(
            "failed to acquire virtual gas allowance for {}: {}",
            sender,
            err
        );
    }

    fvm_ipld_encoding::from_slice::<TokenAmount>(&apply_ret.msg_receipt.return_data)
        .context("failed to parse gas allowance")
}

/// Deducts virtual gas from the sender.
pub(crate) fn deduct_vgas_allowance<E: Executor>(
    executor: &mut E,
    sender: Address,
    amount: TokenAmount,
) -> anyhow::Result<()> {
    let params = fvm_ipld_encoding::RawBytes::serialize(DebitCreditParams {
        from: sender,
        amount: amount.clone(),
    })?;

    let msg = FvmMessage {
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0, // irrelevant for implicit executions.
        gas_limit: i64::MAX as u64,
        method_num: fendermint_actor_blobs_shared::Method::DebitCredit as u64,
        params,
        value: Default::default(),
        version: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    let apply_ret = apply_implicit_message(executor, msg)?;

    if let Some(err) = apply_ret.failure_info {
        bail!(
            "failed to deduct virtual gas for {} (amount={}): {}",
            sender,
            amount,
            err
        );
    }

    Ok(())
}

fn apply_implicit_message<E: Executor>(
    executor: &mut E,
    msg: FvmMessage,
) -> anyhow::Result<ApplyRet> {
    let apply_ret = executor.execute_message(msg, ApplyKind::Implicit, 0)?;
    if let Some(err) = apply_ret.failure_info {
        bail!("failed to apply message: {}", err)
    }
    Ok(apply_ret)
}
// }
