// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Topdown finality related util functions

use crate::chain::TopDownFinalityProvider;
use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::FvmApplyRet;
use anyhow::Context;
use fendermint_vm_topdown::{BlockHeight, IPCParentFinality, ParentViewProvider};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::cross::CrossMsg;

/// Commit the parent finality. Returns the height that the previous parent finality is committed and
/// the committed finality itself. If there is no parent finality committed, genesis epoch is returned.
pub async fn commit_finality<DB>(
    gateway_caller: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    finality: IPCParentFinality,
    provider: &TopDownFinalityProvider,
) -> anyhow::Result<(BlockHeight, Option<IPCParentFinality>)>
where
    DB: Blockstore + Sync + Send + 'static,
{
    let (prev_height, prev_finality) =
        if let Some(prev_finality) = gateway_caller.commit_parent_finality(state, finality)? {
            (prev_finality.height, Some(prev_finality))
        } else {
            (provider.genesis_epoch()?, None)
        };

    tracing::debug!(
        "commit finality parsed: prev_height {prev_height}, prev_finality: {prev_finality:?}"
    );

    Ok((prev_height, prev_finality))
}

/// Execute the top down messages implicitly. Before the execution, mint to the gateway of the funds
/// transferred in the messages, and increase the circulating supply with the incoming value.
pub async fn execute_topdown_msgs<DB>(
    gateway_caller: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    messages: Vec<CrossMsg>,
) -> anyhow::Result<FvmApplyRet>
where
    DB: Blockstore + Sync + Send + 'static,
{
    let total_value: TokenAmount = messages.iter().map(|a| a.msg.value.clone()).sum();

    gateway_caller
        .mint_to_gateway(state, total_value.clone())
        .context("failed to mint to gateway")?;

    state.update_circ_supply(|circ_supply| {
        *circ_supply += total_value;
    });

    gateway_caller.apply_cross_messages(state, messages)
}
