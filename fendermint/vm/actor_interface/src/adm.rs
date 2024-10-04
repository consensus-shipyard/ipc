// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_machine::WriteAccess;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::{address::Address, ActorID, METHOD_CONSTRUCTOR};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

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
    ListMetadata = 4,
}

/// The kinds of machines available.
#[derive(Debug, Serialize, Deserialize)]
pub enum Kind {
    /// An object store with S3-like key semantics.
    ObjectStore,
    /// An MMR accumulator.
    Accumulator,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::ObjectStore => "objectstore",
            Self::Accumulator => "accumulator",
        };
        write!(f, "{}", str)
    }
}

/// Machine metadata.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Metadata {
    /// Machine kind.
    pub kind: Kind,
    /// Machine robust address.
    pub address: Address,
    /// User-defined metadata.
    pub metadata: HashMap<String, String>,
}

/// Helper for machine creation.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CreateExternalParams {
    pub owner: Address,
    pub kind: Kind,
    pub write_access: WriteAccess,
    pub metadata: HashMap<String, String>,
}

/// Helper to read return value from machine creation.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct CreateExternalReturn {
    pub actor_id: ActorID,
    pub robust_address: Option<Address>,
}

/// Helper for listing machine metadata by owner.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListMetadataParams {
    pub owner: Address,
}
