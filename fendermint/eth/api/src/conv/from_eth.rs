// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper methods to convert between Ethereum and FVM data formats.

use anyhow::Context;
use ethers_core::types::{transaction::eip2718::TypedTransaction, H256};

pub use fendermint_vm_message::conv::from_eth::*;
use fvm_shared::{error::ExitCode, message::Message};

use crate::{error, JsonRpcResult};

pub fn to_tm_hash(value: &H256) -> anyhow::Result<tendermint::Hash> {
    tendermint::Hash::try_from(value.as_bytes().to_vec())
        .context("failed to convert to Tendermint Hash")
}

pub fn to_fvm_message(tx: TypedTransaction) -> JsonRpcResult<Message> {
    match tx {
        TypedTransaction::Eip1559(tx) => {
            Ok(fendermint_vm_message::conv::from_eth::to_fvm_message(&tx)?)
        }
        TypedTransaction::Legacy(_) | TypedTransaction::Eip2930(_) => error(
            ExitCode::USR_ILLEGAL_ARGUMENT,
            "unexpected transaction type",
        ),
    }
}
