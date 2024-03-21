// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
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
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CreateExternalParams {
    pub code_cid: Cid,
}

/// Helper to read return value from machine creation.
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct CreateReturn {
    pub actor_id: ActorID,
    pub robust_address: Option<Address>,
}
