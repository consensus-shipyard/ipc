// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::borrow::Cow;

use fendermint_storage::{Codec, Decode, Encode, KVError, KVResult, KVStore};
use fvm_ipld_encoding::{de::DeserializeOwned, serde::Serialize};

/// [`KVStore`] type we use to store data in the database.
#[derive(Clone)]
pub struct AppStore;

impl KVStore for AppStore {
    type Repr = Vec<u8>;
    type Namespace = String;
}

impl<T> Codec<T> for AppStore where AppStore: Encode<T> + Decode<T> {}

/// CBOR serialization.
impl<T> Encode<T> for AppStore
where
    T: Serialize,
{
    fn to_repr(value: &T) -> KVResult<Cow<Self::Repr>> {
        fvm_ipld_encoding::to_vec(value)
            .map_err(|e| KVError::Codec(Box::new(e)))
            .map(Cow::Owned)
    }
}

/// CBOR deserialization.
impl<T> Decode<T> for AppStore
where
    T: DeserializeOwned,
{
    fn from_repr(repr: &Self::Repr) -> KVResult<T> {
        fvm_ipld_encoding::from_slice(repr).map_err(|e| KVError::Codec(Box::new(e)))
    }
}
