// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Gas related message selection

use crate::fvm::gas::BlockGasTracker;
use crate::fvm::state::FvmExecState;
use fendermint_vm_message::signed::SignedMessage;
use fvm_ipld_blockstore::Blockstore;

/// Implement this trait to perform message selection
pub trait MessageSelector {
    fn select_messages<DB: Blockstore + Clone + 'static>(
        &self,
        state: &FvmExecState<DB>,
        msgs: Vec<SignedMessage>,
    ) -> Vec<SignedMessage>;
}

pub(crate) struct GasLimitSelector;

impl MessageSelector for GasLimitSelector {
    fn select_messages<DB: Blockstore + Clone + 'static>(
        &self,
        state: &FvmExecState<DB>,
        mut msgs: Vec<SignedMessage>,
    ) -> Vec<SignedMessage> {
        let total_gas_limit = state.gas_market().available();

        // sort by gas limit descending
        msgs.sort_by(|a, b| b.message.gas_limit.cmp(&a.message.gas_limit));

        let mut total_gas_limit_consumed = 0;
        let mut selected = vec![];
        for msg in msgs {
            if total_gas_limit_consumed + msg.message.gas_limit <= total_gas_limit {
                total_gas_limit_consumed += msg.message.gas_limit;
                selected.push(msg);
            } else {
                break;
            }
        }

        selected
    }
}
