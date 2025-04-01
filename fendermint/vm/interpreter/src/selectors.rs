// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_message::signed::SignedMessage;
use fvm_shared::econ::TokenAmount;

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

pub fn select_messages_above_base_fee(
    msgs: Vec<SignedMessage>,
    base_fee: &TokenAmount,
) -> Vec<SignedMessage> {
    msgs.into_iter()
        .filter(|f| f.message.gas_fee_cap > *base_fee)
        .collect()
}

/// Select messages by gas limit.
/// This function sorts the messages in descending order by gas limit and
/// then selects them until the accumulated gas limit would exceed `total_gas_limit`.
pub fn select_messages_by_gas_limit(
    mut msgs: Vec<SignedMessage>,
    total_gas_limit: u64,
) -> Vec<SignedMessage> {
    // Sort by gas limit descending.
    msgs.sort_by(|a, b| b.message.gas_limit.cmp(&a.message.gas_limit));

    select_until(msgs, total_gas_limit, |msg| msg.message.gas_limit).0
}

/// Select transactions until the total size (in bytes) exceeds `max_tx_bytes`.
pub fn select_messages_until_total_bytes<T: AsRef<[u8]>>(
    txs: Vec<T>,
    max_tx_bytes: usize,
) -> (Vec<T>, usize) {
    let (selected, total) = select_until(txs, max_tx_bytes as u64, |tx| tx.as_ref().len() as u64);
    (selected, total as usize)
}
