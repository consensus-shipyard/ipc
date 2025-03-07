// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::FvmMessage;
use anyhow::{bail, Context};
use fendermint_actor_blobs_shared::state::TokenCreditRate;
use fendermint_actor_recall_config_shared::Method::GetConfig;
use fendermint_actor_recall_config_shared::RecallConfig;
use fendermint_vm_actor_interface::recall_config::RECALL_CONFIG_ACTOR_ADDR;
use fendermint_vm_actor_interface::system;
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::clock::ChainEpoch;
use num_traits::Zero;

/// Makes the current Recall network configuration available to execution state.
#[derive(Debug, Clone)]
pub struct RecallConfigTracker {
    /// The total storage capacity of the subnet.
    pub blob_capacity: u64,
    /// The token to credit rate.
    pub token_credit_rate: TokenCreditRate,
    /// Epoch interval at which to debit all credit accounts.
    pub blob_credit_debit_interval: ChainEpoch,
    /// The minimum epoch duration a blob can be stored.
    pub blob_min_ttl: ChainEpoch,
    /// The default epoch duration a blob is stored.
    pub blob_default_ttl: ChainEpoch,
    /// The number of blobs to delete in a single batch.
    pub blob_delete_batch_size: u64,
    /// The number of accounts to debit in a single batch.
    pub account_debit_batch_size: u64,
}

impl RecallConfigTracker {
    pub fn create<E: Executor>(executor: &mut E) -> anyhow::Result<RecallConfigTracker> {
        let mut ret = Self {
            blob_capacity: Zero::zero(),
            token_credit_rate: TokenCreditRate::from(0usize),
            blob_credit_debit_interval: Zero::zero(),
            blob_min_ttl: Zero::zero(),
            blob_default_ttl: Zero::zero(),
            blob_delete_batch_size: Zero::zero(),
            account_debit_batch_size: Zero::zero(),
        };

        let reading = Self::read_recall_config(executor)?;

        ret.blob_capacity = reading.blob_capacity;
        ret.token_credit_rate = reading.token_credit_rate;
        ret.blob_credit_debit_interval = reading.blob_credit_debit_interval;
        ret.blob_min_ttl = reading.blob_min_ttl;
        ret.blob_default_ttl = reading.blob_default_ttl;
        ret.blob_delete_batch_size = reading.blob_delete_batch_size;
        ret.account_debit_batch_size = reading.account_debit_batch_size;

        Ok(ret)
    }

    pub fn read_recall_config<E: Executor>(executor: &mut E) -> anyhow::Result<RecallConfig> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: RECALL_CONFIG_ACTOR_ADDR,
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
            bail!("failed to acquire recall config: {}", err);
        }

        fvm_ipld_encoding::from_slice::<RecallConfig>(&apply_ret.msg_receipt.return_data)
            .context("failed to parse recall config")
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
