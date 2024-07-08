// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, Recordable, TraceLevel,
    Traceable,
};
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, register_int_gauge_vec,
    HistogramVec, IntCounterVec, IntGauge, IntGaugeVec, Registry,
};

register_metrics! {
    TOPDOWN_PARENT_RPC_CALL_TOTAL: IntCounterVec
        = register_int_counter_vec!("topdown_parent_rpc_call_total", "Parent RPC calls", &["source", "method", "status"]);
    TOPDOWN_PARENT_RPC_CALL_LATENCY_SECS: HistogramVec
        = register_histogram_vec!("topdown_parent_rpc_call_latency_secs", "Parent RPC calls	latency", &["source", "method", "status"]);
    TOPDOWN_PARENT_FINALITY_LATEST_ACQUIRED_HEIGHT: IntGaugeVec
        = register_int_gauge_vec!("topdown_parent_finality_latest_acquired_height", "Latest locally acquired parent finality", &["source"]);
    TOPDOWN_PARENT_FINALITY_VOTING_LATEST_RECEIVED_HEIGHT: IntGaugeVec
        = register_int_gauge_vec!("topdown_parent_finality_voting_latest_received_height", "Parent finality gossip received", &["validator"]);
    TOPDOWN_PARENT_FINALITY_VOTING_LATEST_SENT_HEIGHT: IntGauge
        = register_int_gauge!("topdown_parent_finality_voting_latest_sent_height", "Parent finality peer");
    TOPDOWN_PARENT_FINALITY_VOTING_QUORUM_HEIGHT: IntGauge
        = register_int_gauge!(
            "topdown_parent_finality_voting_quorum_height",
            "Parent finality vote tally new agreement; recorded whenever the latest epoch with quorum"
        );
    TOPDOWN_PARENT_FINALITY_VOTING_QUORUM_WEIGHT: IntGauge
        = register_int_gauge!(
            "topdown_parent_finality_voting_quorum_weight",
            "Parent finality vote tally new agreement; recorded whenever the latest epoch with quorum"
        );
    TOPDOWN_PARENT_FINALITY_COMMITTED_HEIGHT: IntGauge
        = register_int_gauge!("topdown_parent_finality_committed_height", "Parent finality committed on chain");
}

impl_traceables!(
    TraceLevel::Info,
    "Topdown",
    ParentRpcCalled<'a>,
    ParentFinalityAcquired<'a>,
    ParentFinalityPeerVoteReceived<'a>,
    ParentFinalityPeerVoteSent<'a>,
    ParentFinalityPeerQuorumReached<'a>,
    ParentFinalityCommitted<'a>
);

#[derive(Debug)]
pub struct ParentRpcCalled<'a> {
    pub source: &'a str,
    pub json_rpc: &'a str,
    pub method: &'a str,
    pub status: &'a str,
    pub latency: f64,
}

impl Recordable for ParentRpcCalled<'_> {
    fn record_metrics(&self) {
        TOPDOWN_PARENT_RPC_CALL_TOTAL
            .with_label_values(&[self.source, self.method, self.status])
            .inc();

        TOPDOWN_PARENT_RPC_CALL_LATENCY_SECS
            .with_label_values(&[self.source, self.method, self.status])
            .observe(self.latency);
    }
}

pub type BlockHeight = u64;
/// Hex encoded block hash.
pub type BlockHashHex<'a> = &'a str;

#[derive(Debug)]
pub struct ParentFinalityAcquired<'a> {
    pub source: &'a str,
    pub is_null: bool,
    pub block_height: BlockHeight,
    pub block_hash: Option<BlockHashHex<'a>>,
    pub commitment_hash: Option<BlockHashHex<'a>>,
    pub num_msgs: usize,
    pub num_validator_changes: usize,
}

impl Recordable for ParentFinalityAcquired<'_> {
    fn record_metrics(&self) {
        TOPDOWN_PARENT_FINALITY_LATEST_ACQUIRED_HEIGHT
            .with_label_values(&[self.source])
            .set(self.block_height as i64);
    }
}

#[derive(Debug)]
pub struct ParentFinalityPeerVoteReceived<'a> {
    validator: &'a str,
    block_height: u64,
    block_hash: &'a str,
    commitment_hash: &'a str,
}

impl Recordable for ParentFinalityPeerVoteReceived<'_> {
    fn record_metrics(&self) {
        TOPDOWN_PARENT_FINALITY_VOTING_LATEST_RECEIVED_HEIGHT
            .with_label_values(&[self.validator])
            .set(self.block_height as i64);
    }
}

#[derive(Debug)]
pub struct ParentFinalityPeerVoteSent<'a> {
    block_height: u64,
    block_hash: &'a str,
    commitment_hash: &'a str,
}

impl Recordable for ParentFinalityPeerVoteSent<'_> {
    fn record_metrics(&self) {
        TOPDOWN_PARENT_FINALITY_VOTING_LATEST_SENT_HEIGHT.set(self.block_height as i64);
    }
}

#[derive(Debug)]
pub struct ParentFinalityPeerQuorumReached<'a> {
    block_height: u64,
    block_hash: &'a str,
    commitment_hash: &'a str,
    weight: u32,
}

impl Recordable for ParentFinalityPeerQuorumReached<'_> {
    fn record_metrics(&self) {
        TOPDOWN_PARENT_FINALITY_VOTING_QUORUM_HEIGHT.set(self.block_height as i64);

        // TODO Karel - this should be sum of weights of all validators that voted? Ask Raul
        TOPDOWN_PARENT_FINALITY_VOTING_QUORUM_WEIGHT.set(self.weight as i64);
    }
}

#[derive(Debug)]
pub struct ParentFinalityCommitted<'a> {
    local_height: u64,
    parent_height: u64,
    block_hash: &'a str,
    proposer: &'a str,
}

impl Recordable for ParentFinalityCommitted<'_> {
    fn record_metrics(&self) {
        TOPDOWN_PARENT_FINALITY_COMMITTED_HEIGHT.set(self.parent_height as i64);
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

        // Initialize the metric values
        let source = "source";
        let method = "method";
        let status = "status";
        let initial_value = TOPDOWN_PARENT_RPC_CALL_TOTAL
            .with_label_values(&[source, method, status])
            .get();

        // Emit a record to increase the metric
        emit(ParentRpcCalled {
            source,
            json_rpc: "json_rpc",
            method,
            status,
            latency: 0.0,
        });

        // Check that the metric value has increased by 1
        let new_value = TOPDOWN_PARENT_RPC_CALL_TOTAL
            .with_label_values(&[source, method, status])
            .get();
        assert_eq!(new_value, initial_value + 1);
    }

    #[test]
    fn test_emit() {
        emit(ParentRpcCalled {
            source: "source",
            json_rpc: "json_rpc",
            method: "method",
            status: "status",
            latency: 0.0,
        });

        emit(ParentFinalityAcquired {
            source: "source",
            is_null: false,
            block_height: 0,
            block_hash: Some("block_hash"),
            commitment_hash: Some("commitment_hash"),
            num_msgs: 0,
            num_validator_changes: 0,
        });

        emit(ParentFinalityPeerVoteReceived {
            validator: "validator",
            block_height: 0,
            block_hash: "block_hash",
            commitment_hash: "commitment_hash",
        });

        emit(ParentFinalityPeerVoteSent {
            block_height: 0,
            block_hash: "block_hash",
            commitment_hash: "commitment_hash",
        });

        emit(ParentFinalityPeerQuorumReached {
            block_height: 0,
            block_hash: "block_hash",
            commitment_hash: "commitment_hash",
            weight: 0,
        });

        emit(ParentFinalityCommitted {
            local_height: 0,
            parent_height: 0,
            block_hash: "block_hash",
            proposer: "proposer",
        });
    }
}
