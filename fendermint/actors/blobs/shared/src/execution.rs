// Copyright 2026 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch};
use serde::{Deserialize, Serialize};

use crate::bytes::B256;

// FEVM InvokeContract selectors used by blobs actor facade for execution methods.
pub const CREATE_JOB_SELECTOR: [u8; 4] = [0x6b, 0xa4, 0x8d, 0x87];
pub const CLAIM_JOB_SELECTOR: [u8; 4] = [0x9c, 0x7d, 0xd2, 0x19];
pub const COMPLETE_JOB_SELECTOR: [u8; 4] = [0x59, 0x2f, 0x72, 0xc4];
pub const FAIL_JOB_SELECTOR: [u8; 4] = [0xf5, 0xe2, 0x2c, 0x70];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Claimed,
    Running,
    Succeeded,
    Failed,
    TimedOut,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ExecutionJob {
    pub id: u64,
    pub creator: Address,
    pub claimed_by: Option<Address>,
    pub status: JobStatus,
    pub binary_ref: String,
    pub input_refs: Vec<String>,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub timeout_secs: u64,
    pub created_epoch: ChainEpoch,
    pub started_epoch: Option<ChainEpoch>,
    pub completed_epoch: Option<ChainEpoch>,
    pub output_refs: Vec<String>,
    pub output_commitment: Option<B256>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CreateJobParams {
    pub binary_ref: String,
    pub input_refs: Vec<String>,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub timeout_secs: u64,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ClaimJobParams {
    pub id: u64,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CompleteJobParams {
    pub id: u64,
    pub output_refs: Vec<String>,
    pub output_commitment: B256,
    pub exit_code: i32,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct FailJobParams {
    pub id: u64,
    pub reason: String,
    pub exit_code: i32,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetJobParams {
    pub id: u64,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListJobsParams {
    pub status: Option<JobStatus>,
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListJobsReturn {
    pub jobs: Vec<ExecutionJob>,
}
