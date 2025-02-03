// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt;

use fendermint_actor_blobs_shared::state::Hash;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::State;

pub const BLOB_READER_ACTOR_NAME: &str = "blob_reader";
pub const BLOB_READER_ACTOR_ID: ActorID = 67;
pub const BLOB_READER_ACTOR_ADDR: Address = Address::new_id(BLOB_READER_ACTOR_ID);

/// The status of a read request.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum ReadRequestStatus {
    /// Read request is open and waiting to be processed
    #[default]
    Open,
    /// Read request is being processed
    Pending,
}

impl fmt::Display for ReadRequestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadRequestStatus::Open => write!(f, "open"),
            ReadRequestStatus::Pending => write!(f, "pending"),
        }
    }
}

/// A request to read blob data.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ReadRequest {
    /// The hash of the blob to read data from.
    pub blob_hash: Hash,
    /// The offset to start reading from.
    pub offset: u32,
    /// The length of data to read.
    pub len: u32,
    /// The address to call back when the read is complete.
    pub callback_addr: Address,
    /// The method to call back when the read is complete.
    pub callback_method: MethodNum,
    /// Status of the read request
    pub status: ReadRequestStatus,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    GetReadRequestStatus = frc42_dispatch::method_hash!("GetReadRequestStatus"),
    CloseReadRequest = frc42_dispatch::method_hash!("CloseReadRequest"),
    GetOpenReadRequests = frc42_dispatch::method_hash!("GetOpenReadRequests"),
    OpenReadRequest = frc42_dispatch::method_hash!("OpenReadRequest"),
    SetReadRequestPending = frc42_dispatch::method_hash!("SetReadRequestPending"),
}

/// Params for adding a read request.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct OpenReadRequestParams {
    /// The hash of the blob to read.
    pub hash: Hash,
    /// The offset to start reading from.
    pub offset: u32,
    /// The length of the read request.
    pub len: u32,
    /// The address to call back when the read is complete.
    pub callback_addr: Address,
    /// The method to call back when the read is complete.
    pub callback_method: MethodNum,
}

/// Params for closing a read request. The ID of the read request.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CloseReadRequestParams(pub Hash);

/// Params for getting pending read requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetOpenReadRequestsParams(pub u32);

/// Params for setting a read request to pending.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetReadRequestPendingParams(pub Hash);

/// Params for getting read request status.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetReadRequestStatusParams(pub Hash);
