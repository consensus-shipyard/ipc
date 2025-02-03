// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, Recordable, TraceLevel,
    Traceable,
};
use prometheus::{register_int_counter_vec, register_int_gauge, IntCounterVec, IntGauge, Registry};

register_metrics! {
    BLOBS_FINALITY_VOTING_SUCCESS: IntCounterVec
        = register_int_counter_vec!(
            "blobs_finality_voting_success",
            "Blobs finality: number of votes for successful blob resolution",
            &["blob_hash"]
        );
    BLOBS_FINALITY_VOTING_FAILURE: IntCounterVec
        = register_int_counter_vec!(
            "blobs_finality_voting_failure",
            "Blobs finality: number of votes for failed blob resolution",
            &["blob_hash"]
        );
    BLOBS_FINALITY_PENDING_BLOBS: IntGauge
        = register_int_gauge!(
            "blobs_finality_pending_blobs",
            "Blobs finality: current count of pending blobs"
        );
    BLOBS_FINALITY_PENDING_BYTES: IntGauge
        = register_int_gauge!("blobs_finality_pending_bytes", "Blobs finality: current count of pending bytes");

    BLOBS_FINALITY_ADDED_BLOBS: IntGauge
        = register_int_gauge!("blobs_finality_added_blobs", "Blobs finality: current count of added blobs");

    BLOBS_FINALITY_ADDED_BYTES: IntGauge
        = register_int_gauge!("blobs_finality_added_bytes", "Blobs finality: current count of added bytes");

    READ_REQUESTS_VOTING_CLOSE: IntCounterVec
        = register_int_counter_vec!(
            "read_requests_voting_close",
            "Read requests finality: number of votes for closing read request",
            &["read_request_id"]
        );
}

impl_traceables!(
    TraceLevel::Info,
    "IrohResolver",
    BlobsFinalityVotingFailure,
    BlobsFinalityVotingSuccess,
    BlobsFinalityPendingBlobs,
    BlobsFinalityPendingBytes,
    BlobsFinalityAddedBlobs,
    BlobsFinalityAddedBytes,
    ReadRequestsCloseVoting
);

#[derive(Debug)]
pub struct BlobsFinalityVotingSuccess {
    pub blob_hash: Option<String>,
}

impl Recordable for BlobsFinalityVotingSuccess {
    fn record_metrics(&self) {
        let hash = self.blob_hash.as_deref().unwrap_or("");
        BLOBS_FINALITY_VOTING_SUCCESS
            .with_label_values(&[hash])
            .inc();
    }
}

#[derive(Debug)]
pub struct BlobsFinalityVotingFailure {
    pub blob_hash: Option<String>,
}

impl Recordable for BlobsFinalityVotingFailure {
    fn record_metrics(&self) {
        let hash = self.blob_hash.as_deref().unwrap_or("");
        BLOBS_FINALITY_VOTING_FAILURE
            .with_label_values(&[hash])
            .inc();
    }
}

#[derive(Debug)]
pub struct BlobsFinalityPendingBlobs(pub u64);

impl Recordable for BlobsFinalityPendingBlobs {
    fn record_metrics(&self) {
        BLOBS_FINALITY_PENDING_BLOBS.set(self.0 as i64);
    }
}

#[derive(Debug)]
pub struct BlobsFinalityPendingBytes(pub u64);

impl Recordable for BlobsFinalityPendingBytes {
    fn record_metrics(&self) {
        BLOBS_FINALITY_PENDING_BYTES.set(self.0 as i64);
    }
}

#[derive(Debug)]
pub struct BlobsFinalityAddedBlobs(pub u64);

impl Recordable for BlobsFinalityAddedBlobs {
    fn record_metrics(&self) {
        BLOBS_FINALITY_ADDED_BLOBS.set(self.0 as i64);
    }
}

#[derive(Debug)]
pub struct BlobsFinalityAddedBytes(pub u64);

impl Recordable for BlobsFinalityAddedBytes {
    fn record_metrics(&self) {
        BLOBS_FINALITY_ADDED_BYTES.set(self.0 as i64);
    }
}

#[derive(Debug)]
pub struct ReadRequestsCloseVoting {
    pub read_request_id: Option<String>,
}

impl Recordable for ReadRequestsCloseVoting {
    fn record_metrics(&self) {
        let id = self.read_request_id.as_deref().unwrap_or("");
        READ_REQUESTS_VOTING_CLOSE.with_label_values(&[id]).inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipc_observability::emit;

    #[test]
    fn test_metrics() {
        let registry = Registry::new();
        register_metrics(&registry).unwrap();
    }

    #[test]
    fn test_metric_increase() {
        let registry = Registry::new();
        register_metrics(&registry).unwrap();

        emit(BlobsFinalityPendingBlobs(1));
        emit(BlobsFinalityPendingBytes(1));
        emit(BlobsFinalityAddedBlobs(1));
        emit(BlobsFinalityAddedBytes(1));
        emit(ReadRequestsCloseVoting {
            read_request_id: Some(String::from("id")),
        });
    }

    #[test]
    fn test_emit() {
        emit(BlobsFinalityVotingSuccess {
            blob_hash: Some(String::from("hash")),
        });
        emit(BlobsFinalityVotingFailure {
            blob_hash: Some(String::from("hash")),
        });
        emit(BlobsFinalityPendingBlobs(1));
        emit(BlobsFinalityPendingBytes(1));
        emit(BlobsFinalityAddedBlobs(1));
        emit(BlobsFinalityAddedBytes(1));
    }
}
