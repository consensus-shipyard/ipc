// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper methods to convert between Ethereum and FVM data formats.

use anyhow::Context;
use ethers_core::types::H256;

pub use fendermint_vm_message::conv::from_eth::*;

pub fn to_tm_hash(value: &H256) -> anyhow::Result<tendermint::Hash> {
    tendermint::Hash::try_from(value.as_bytes().to_vec())
        .context("failed to convert to Tendermint Hash")
}
