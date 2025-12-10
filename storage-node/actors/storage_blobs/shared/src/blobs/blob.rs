// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fvm_ipld_encoding::tuple::*;
use fvm_shared::clock::ChainEpoch;

use super::{BlobStatus, SubscriptionId};
use crate::bytes::B256;

/// The external (shared) view of a blob.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: B256,
    /// Active subscribers (accounts) that are paying for the blob to expiry.
    pub subscribers: HashMap<SubscriptionId, ChainEpoch>,
    /// Blob status.
    pub status: BlobStatus,
}
