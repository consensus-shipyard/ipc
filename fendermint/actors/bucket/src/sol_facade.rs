// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use fendermint_actor_blobs_shared::state::Hash;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::bucket as sol;
use std::collections::HashMap;

pub struct ObjectAdded<'a> {
    pub key: &'a Vec<u8>,
    pub blob_hash: &'a Hash,
    pub metadata: &'a HashMap<String, String>,
}
impl<'a> ObjectAdded<'a> {
    pub fn new(
        key: &'a Vec<u8>,
        blob_hash: &'a Hash,
        metadata: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            key,
            blob_hash,
            metadata,
        }
    }
}
impl TryIntoEVMEvent for ObjectAdded<'_> {
    type Target = sol::Events;

    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let metadata = fvm_ipld_encoding::to_vec(self.metadata)?;
        Ok(sol::Events::ObjectAdded(sol::ObjectAdded {
            key: self.key.clone().into(),
            blobHash: self.blob_hash.0.into(),
            metadata: metadata.into(),
        }))
    }
}

pub struct ObjectMetadataUpdated<'a> {
    pub key: &'a Vec<u8>,
    pub metadata: &'a HashMap<String, String>,
}
impl<'a> ObjectMetadataUpdated<'a> {
    pub fn new(key: &'a Vec<u8>, metadata: &'a HashMap<String, String>) -> Self {
        Self { key, metadata }
    }
}
impl<'a> TryIntoEVMEvent for ObjectMetadataUpdated<'a> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let metadata = fvm_ipld_encoding::to_vec(self.metadata)?;
        Ok(sol::Events::ObjectMetadataUpdated(
            sol::ObjectMetadataUpdated {
                key: self.key.clone().into(),
                metadata: metadata.into(),
            },
        ))
    }
}

pub struct ObjectDeleted<'a> {
    pub key: &'a Vec<u8>,
    pub blob_hash: &'a Hash,
}
impl<'a> ObjectDeleted<'a> {
    pub fn new(key: &'a Vec<u8>, blob_hash: &'a Hash) -> Self {
        Self { key, blob_hash }
    }
}
impl TryIntoEVMEvent for ObjectDeleted<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        Ok(sol::Events::ObjectDeleted(sol::ObjectDeleted {
            key: self.key.clone().into(),
            blobHash: self.blob_hash.0.into(),
        }))
    }
}
