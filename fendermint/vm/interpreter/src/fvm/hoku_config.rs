// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::FvmMessage;
use anyhow::{bail, Context};
use fendermint_actor_hoku_config_shared::HokuConfig;
use fendermint_actor_hoku_config_shared::Method::GetConfig;
use fendermint_vm_actor_interface::hoku_config::HOKU_CONFIG_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use num_traits::Zero;

/// Makes the current Hoku network configuration available to execution state.
#[derive(Debug, Clone)]
pub struct HokuConfigTracker {
    /// The total storage capacity of the subnet.
    pub blob_capacity: u64,
    /// The token to credit rate. The amount of atto credits that 1 atto buys.
    pub token_credit_rate: BigInt,
    /// Block interval at which to debit all credit accounts.
    pub blob_credit_debit_interval: ChainEpoch,
}

impl HokuConfigTracker {
    pub fn create<E: Executor>(executor: &mut E) -> anyhow::Result<HokuConfigTracker> {
        let mut ret = Self {
            blob_capacity: Zero::zero(),
            token_credit_rate: Zero::zero(),
            blob_credit_debit_interval: Zero::zero(),
        };

        let reading = Self::read_hoku_config(executor)?;

        ret.blob_capacity = reading.blob_capacity;
        ret.token_credit_rate = reading.token_credit_rate;
        ret.blob_credit_debit_interval = reading.blob_credit_debit_interval;

        Ok(ret)
    }

    pub fn read_hoku_config<E: Executor>(executor: &mut E) -> anyhow::Result<HokuConfig> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: HOKU_CONFIG_ACTOR_ADDR,
            sequence: 0, // irrelevant for implicit executions.
            gas_limit: i64::MAX as u64,
            method_num: GetConfig as u64,
            params: fvm_ipld_encoding::RawBytes::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = Self::apply_implicit_message(executor, msg)?;

        if let Some(err) = apply_ret.failure_info {
            bail!("failed to acquire hoku config: {}", err);
        }

        fvm_ipld_encoding::from_slice::<HokuConfig>(&apply_ret.msg_receipt.return_data)
            .context("failed to parse hoku config")
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
}
