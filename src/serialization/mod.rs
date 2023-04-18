// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2022-2023 Protocol Labs
//! Handles the serialization of different types between actor cbor tuple serialization and json rpc
//! json serialization.

mod checkpoint;

/// A helper struct to serialize struct to json.
///
/// Most of the types should have no need to use this struct. But some types that are shared between
/// actor, which are using cbor tuple serialization and json rpc response. We are using this wrapper
/// to handle convert to json instead.
#[derive(Debug)]
pub struct SerializeToJson<T>(pub T);
