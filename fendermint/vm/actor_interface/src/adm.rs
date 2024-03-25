// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::{address::Address, ActorID, METHOD_CONSTRUCTOR};

define_singleton!(ADM {
    id: 17,
    code_id: 17
});

pub const ADM_ACTOR_NAME: &str = "adm";

/// ADM actor methods available.
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    CreateExternal = 2,
    UpdateDeployers = 3,
}

/// Helper for machine creation.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CreateParams {
    pub machine_name: String,
}

/// Helper to read return value from machine creation.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct CreateReturn {
    pub actor_id: ActorID,
    pub robust_address: Option<Address>,
}
