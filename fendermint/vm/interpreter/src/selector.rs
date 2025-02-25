// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Gas related message selection

use fendermint_vm_message::{chain::ChainMessage, ipc::IpcMessage};

use crate::verify::VerifiableMessage;

pub trait GasLimit {
    fn gas_limit(&self) -> u64;
}

impl GasLimit for ChainMessage {
    fn gas_limit(&self) -> u64 {
        match self {
            ChainMessage::Signed(s) => s.message.gas_limit,
            ChainMessage::Ipc(ipc) => match ipc {
                IpcMessage::BottomUpResolve(relayed) => relayed.message.gas_limit,
                other => {
                    // This should never happen as only messages above can be in the mempool.
                    // But if it does, let's not panic and just return 0 gas limit which should not temper
                    // with the block gas limit.
                    tracing::warn!(
                        error = "unexpected IpcMessage variant encountered",
                        message = ?other
                    );
                    0
                }
            },
        }
    }
}

/// Generic helper: select items until the accumulated weight exceeds `max`.
/// Returns a tuple of (selected items, accumulated weight).
pub fn select_until<T, F>(items: Vec<T>, max: u64, weight: F) -> (Vec<T>, u64)
where
    F: Fn(&T) -> u64,
{
    let mut total: u64 = 0;
    let mut out = Vec::new();
    for item in items {
        let w = weight(&item);
        if total.saturating_add(w) > max {
            break;
        }
        total += w;
        out.push(item);
    }
    (out, total)
}

/// Select messages by gas limit.
/// This function sorts the messages in descending order by gas limit and
/// then selects them until the accumulated gas limit would exceed `total_gas_limit`.
pub fn select_messages_by_gas_limit(
    mut msgs: Vec<VerifiableMessage>,
    total_gas_limit: u64,
) -> Vec<VerifiableMessage> {
    // Sort by gas limit descending.
    msgs.sort_by(|a, b| b.gas_limit().cmp(&a.gas_limit()));

    select_until(msgs, total_gas_limit, |msg| msg.gas_limit()).0
}

/// Select transactions until the total size (in bytes) exceeds `max_tx_bytes`.
pub fn select_messages_until_total_bytes<T: AsRef<[u8]>>(
    txs: Vec<T>,
    max_tx_bytes: usize,
) -> (Vec<T>, usize) {
    let (selected, total) = select_until(txs, max_tx_bytes as u64, |tx| tx.as_ref().len() as u64);
    (selected, total as usize)
}
