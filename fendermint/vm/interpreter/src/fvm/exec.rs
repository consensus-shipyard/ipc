// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_trait::async_trait;

use fendermint_vm_actor_interface::{cron, system};
use fvm::executor::ApplyRet;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, MethodNum, BLOCK_GAS_LIMIT};

use crate::ExecInterpreter;

use super::{state::FvmExecState, FvmMessage, FvmMessageInterpreter};

/// The return value extended with some things from the message that
/// might not be available to the caller, because of the message lookups
/// and transformations that happen along the way, e.g. where we need
/// a field, we might just have a CID.
pub struct FvmApplyRet {
    pub apply_ret: ApplyRet,
    pub from: Address,
    pub to: Address,
    pub method_num: MethodNum,
    pub gas_limit: u64,
}

impl<DB> Default for FvmMessageInterpreter<DB> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<DB> ExecInterpreter for FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync,
{
    type State = FvmExecState<DB>;
    type Message = FvmMessage;
    type BeginOutput = FvmApplyRet;
    type DeliverOutput = FvmApplyRet;
    type EndOutput = ();

    async fn begin(
        &self,
        mut state: Self::State,
    ) -> anyhow::Result<(Self::State, Self::BeginOutput)> {
        // Block height (FVM epoch) as sequence is intentional
        let height = state.block_height();
        // Arbitrarily large gas limit for cron (matching how Forest does it, which matches Lotus).
        // XXX: Our blocks are not necessarily expected to be 30 seconds apart, so the gas limit might be wrong.
        let gas_limit = BLOCK_GAS_LIMIT * 10000;
        let from = system::SYSTEM_ACTOR_ADDR;
        let to = cron::CRON_ACTOR_ADDR;
        let method_num = cron::Method::EpochTick as u64;
        // Cron.
        let msg = FvmMessage {
            from,
            to,
            sequence: height as u64,
            gas_limit,
            method_num,
            params: Default::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = state.execute_implicit(msg)?;

        // Failing cron would be fatal.
        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to apply block cron message: {}", err);
        }

        let ret = FvmApplyRet {
            apply_ret,
            from,
            to,
            method_num,
            gas_limit,
        };

        Ok((state, ret))
    }

    async fn deliver(
        &self,
        mut state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        let from = msg.from;
        let to = msg.to;
        let method_num = msg.method_num;
        let gas_limit = msg.gas_limit;

        let apply_ret = state.execute_explicit(msg)?;

        let ret = FvmApplyRet {
            apply_ret,
            from,
            to,
            method_num,
            gas_limit,
        };

        Ok((state, ret))
    }

    async fn end(&self, state: Self::State) -> anyhow::Result<(Self::State, Self::EndOutput)> {
        // TODO: Epoch transitions for checkpointing.
        Ok((state, ()))
    }
}
