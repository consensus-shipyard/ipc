// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod null;

use crate::error::Error;
use crate::BlockHash;
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;

pub use null::FinalityWithNull;

pub(crate) type ParentViewPayload = (BlockHash, Vec<PowerChangeRequest>, Vec<IpcEnvelope>);

fn ensure_sequential<T, F: Fn(&T) -> u64>(msgs: &[T], f: F) -> Result<(), Error> {
    if msgs.is_empty() {
        return Ok(());
    }

    let first = msgs.first().unwrap();
    let mut nonce = f(first);
    for msg in msgs.iter().skip(1) {
        if nonce + 1 != f(msg) {
            return Err(Error::NotSequential);
        }
        nonce += 1;
    }

    Ok(())
}
