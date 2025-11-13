// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt;

use fendermint_actor_blobs_shared::bytes::B256;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::State;

pub const BLOB_READER_ACTOR_NAME: &str = "blob_reader";
pub const BLOB_READER_ACTOR_ID: ActorID = 67;
pub const BLOB_READER_ACTOR_ADDR: Address = Address::new_id(BLOB_READER_ACTOR_ID);

/// The status of a read request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
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
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct ReadRequest {
    /// The hash of the blob to read data from.
    pub blob_hash: B256,
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

    // User methods
    OpenReadRequest = frc42_dispatch::method_hash!("OpenReadRequest"),

    // System methods
    GetReadRequestStatus = frc42_dispatch::method_hash!("GetReadRequestStatus"),
    GetOpenReadRequests = frc42_dispatch::method_hash!("GetOpenReadRequests"),
    GetPendingReadRequests = frc42_dispatch::method_hash!("GetPendingReadRequests"),
    SetReadRequestPending = frc42_dispatch::method_hash!("SetReadRequestPending"),
    CloseReadRequest = frc42_dispatch::method_hash!("CloseReadRequest"),
}

/// Params for adding a read request.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct OpenReadRequestParams {
    /// The hash of the blob to read.
    pub hash: B256,
    /// The offset to start reading from.
    pub offset: u32,
    /// The length of the read request.
    pub len: u32,
    /// The address to call back when the read is complete.
    pub callback_addr: Address,
    /// The method to call back when the read is complete.
    pub callback_method: MethodNum,
}

/// Params for getting read request status.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetReadRequestStatusParams(pub B256);

/// Params for getting open read requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetOpenReadRequestsParams(pub u32);

/// Params for getting pending read requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetPendingReadRequestsParams(pub u32);

/// Params for setting a read request to pending.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetReadRequestPendingParams(pub B256);

/// Params for closing a read request. The ID of the read request.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CloseReadRequestParams(pub B256);

/// Return type for request queues.
pub type ReadRequestTuple = (B256, B256, u32, u32, Address, u64);
