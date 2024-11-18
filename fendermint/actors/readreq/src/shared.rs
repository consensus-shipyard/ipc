// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use crate::state::State;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const READREQ_ACTOR_NAME: &str = "readreq";
pub const READREQ_ACTOR_ID: ActorID = 67;
pub const READREQ_ACTOR_ADDR: Address = Address::new_id(READREQ_ACTOR_ID);

/// Blob blake3 hash.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

/// Source https://github.com/n0-computer/iroh/blob/main/iroh-base/src/hash.rs
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // the result will be 52 bytes
        let mut res = [b'b'; 52];
        // write the encoded bytes
        data_encoding::BASE32_NOPAD.encode_mut(self.0.as_slice(), &mut res);
        // convert to string, this is guaranteed to succeed
        let t = std::str::from_utf8_mut(res.as_mut()).unwrap();
        // hack since data_encoding doesn't have BASE32LOWER_NOPAD as a const
        t.make_ascii_lowercase();
        // write the str, no allocations
        f.write_str(t)
    }
}

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
