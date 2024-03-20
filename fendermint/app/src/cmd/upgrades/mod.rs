// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::chainid::ChainID;
use lazy_static::lazy_static;

pub mod patch_actor_state;

lazy_static! {
    pub static ref EXAMPLE_CHAIN_ID: ChainID = ChainID::from(1942764459484029);
}
