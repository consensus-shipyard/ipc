// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Storage-node actor interfaces.
//!
//! These define the actor IDs, method numbers, and data types for storage-node actors.
//! Moved from fendermint/vm/actor_interface to achieve true plugin isolation.

// Macro definitions needed for actor ID/code definitions
macro_rules! define_code {
    ($name:ident { code_id: $code_id:literal }) => {
        paste::paste! {
            /// Position of the actor in the builtin actor bundle manifest.
            pub const [<$name _ACTOR_CODE_ID>]: u32 = $code_id;
        }
    };
}

macro_rules! define_id {
    ($name:ident { id: $id:literal }) => {
        paste::paste! {
            pub const [<$name _ACTOR_ID>]: fvm_shared::ActorID = $id;
            pub const [<$name _ACTOR_ADDR>]: fvm_shared::address::Address = fvm_shared::address::Address::new_id([<$name _ACTOR_ID>]);
        }
    };
}

macro_rules! define_singleton {
    ($name:ident { id: $id:literal, code_id: $code_id:literal }) => {
        define_id!($name { id: $id });
        define_code!($name { code_id: $code_id });
    };
}

pub mod adm;
pub mod blob_reader;
pub mod blobs;
pub mod bucket;
pub mod recall_config;
