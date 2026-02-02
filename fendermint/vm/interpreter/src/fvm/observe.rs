// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, serde::HexEncodableBlockHash,
    Recordable, TraceLevel, Traceable,
};

use prometheus::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec, Registry,
};

use fvm_shared::message::Message;

register_metrics! {
    EXEC_FVM_CHECK_EXECUTION_TIME_SECS: Histogram
        = register_histogram!("exec_fvm_check_execution_time_secs", "Execution time of FVM check in seconds");
    EXEC_FVM_ESTIMATE_EXECUTION_TIME_SECS: Histogram
        = register_histogram!("exec_fvm_estimate_execution_time_secs", "Execution time of FVM estimate in seconds");
    EXEC_FVM_APPLY_EXECUTION_TIME_SECS: Histogram
        = register_histogram!("exec_fvm_apply_execution_time_secs", "Execution time of FVM apply in seconds");
    EXEC_FVM_CALL_EXECUTION_TIME_SECS: Histogram
        = register_histogram!("exec_fvm_call_execution_time_secs", "Execution time of FVM call in seconds");
    BOTTOMUP_CHECKPOINT_CREATED_TOTAL: IntCounter
        = register_int_counter!("bottomup_checkpoint_created_total", "Bottom-up checkpoint produced");
    BOTTOMUP_CHECKPOINT_CREATED_HEIGHT: IntGauge
        = register_int_gauge!("bottomup_checkpoint_created_height", "Height of the checkpoint created");
    BOTTOMUP_CHECKPOINT_CREATED_MSGCOUNT: IntGauge
        = register_int_gauge!("bottomup_checkpoint_created_msgcount", "Number of messages in the checkpoint created");
    BOTTOMUP_CHECKPOINT_CREATED_CONFIGNUM: IntGauge
        = register_int_gauge!("bottomup_checkpoint_created_confignum", "Configuration number of the checkpoint created");
    BOTTOMUP_CHECKPOINT_SIGNED_HEIGHT: IntGaugeVec = register_int_gauge_vec!(
        "bottomup_checkpoint_signed_height",
        "Height of the checkpoint signed",
        &["validator"]
    );
    BOTTOMUP_CHECKPOINT_FINALIZED_HEIGHT: IntGauge
        = register_int_gauge!("bottomup_checkpoint_finalized_height", "Height of the checkpoint finalized");

    F3_TOPDOWN_CACHE_WAIT_TOTAL: IntCounterVec = register_int_counter_vec!(
        "f3_topdown_cache_wait_total",
        "Number of times the node waited for the local F3 proof cache during top-down execution",
        &["status"]
    );
    F3_TOPDOWN_CACHE_WAIT_SECS: HistogramVec = register_histogram_vec!(
        "f3_topdown_cache_wait_secs",
        "Seconds spent waiting for the local F3 proof cache during top-down execution",
        &["status"]
    );
}

impl_traceables!(TraceLevel::Info, "Execution", MsgExec);

#[derive(Debug, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MsgExecPurpose {
    Check,
    Apply,
    Estimate,
    Call,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MsgExec {
    pub purpose: MsgExecPurpose,
    pub message: Message,
    pub height: i64,
    pub duration: f64,
    pub exit_code: u32,
}

impl Recordable for MsgExec {
    fn record_metrics(&self) {
        match self.purpose {
            MsgExecPurpose::Check => EXEC_FVM_CHECK_EXECUTION_TIME_SECS.observe(self.duration),
            MsgExecPurpose::Estimate => {
                EXEC_FVM_ESTIMATE_EXECUTION_TIME_SECS.observe(self.duration)
            }
            MsgExecPurpose::Apply => EXEC_FVM_APPLY_EXECUTION_TIME_SECS.observe(self.duration),
            MsgExecPurpose::Call => EXEC_FVM_CALL_EXECUTION_TIME_SECS.observe(self.duration),
        }
    }
}

impl_traceables!(
    TraceLevel::Info,
    "Bottomup",
    CheckpointCreated,
    CheckpointSigned,
    CheckpointFinalized
);

impl_traceables!(
    TraceLevel::Error,
    "Topdown",
    F3CacheWaitStuck
);
impl_traceables!(TraceLevel::Info, "Topdown", F3CacheWaitRecovered);

#[derive(Debug)]
pub struct CheckpointCreated {
    pub height: u64,
    pub hash: HexEncodableBlockHash,
    pub msg_count: usize,
    pub config_number: u64,
}

impl Recordable for CheckpointCreated {
    fn record_metrics(&self) {
        BOTTOMUP_CHECKPOINT_CREATED_TOTAL.inc();
        BOTTOMUP_CHECKPOINT_CREATED_HEIGHT.set(self.height as i64);
        BOTTOMUP_CHECKPOINT_CREATED_MSGCOUNT.set(self.msg_count as i64);
        BOTTOMUP_CHECKPOINT_CREATED_CONFIGNUM.set(self.config_number as i64);
    }
}

#[derive(Debug)]
pub enum CheckpointSignedRole {
    Own,
    Peer,
}

#[derive(Debug)]
pub struct CheckpointSigned {
    pub role: CheckpointSignedRole,
    pub height: u64,
    pub hash: HexEncodableBlockHash,
    pub validator: Address,
}

impl Recordable for CheckpointSigned {
    fn record_metrics(&self) {
        BOTTOMUP_CHECKPOINT_SIGNED_HEIGHT
            .with_label_values(&[format!("{}", self.validator).as_str()])
            .set(self.height as i64);
    }
}

#[derive(Debug)]
pub struct CheckpointFinalized {
    pub height: i64,
    pub hash: HexEncodableBlockHash,
}

impl Recordable for CheckpointFinalized {
    fn record_metrics(&self) {
        BOTTOMUP_CHECKPOINT_FINALIZED_HEIGHT.set(self.height);
    }
}

#[derive(Debug)]
pub struct F3CacheWaitStuck {
    pub epoch: u64,
    pub waited_secs: f64,
}

impl Recordable for F3CacheWaitStuck {
    fn record_metrics(&self) {
        F3_TOPDOWN_CACHE_WAIT_TOTAL
            .with_label_values(&["stuck"])
            .inc();
        F3_TOPDOWN_CACHE_WAIT_SECS
            .with_label_values(&["stuck"])
            .observe(self.waited_secs);
    }
}

// NOTE: We intentionally do not have a one-shot "timeout" event. Execution waits indefinitely.

#[derive(Debug)]
pub struct F3CacheWaitRecovered {
    pub epoch: u64,
    pub waited_secs: f64,
}

impl Recordable for F3CacheWaitRecovered {
    fn record_metrics(&self) {
        F3_TOPDOWN_CACHE_WAIT_TOTAL
            .with_label_values(&["recovered"])
            .inc();
        F3_TOPDOWN_CACHE_WAIT_SECS
            .with_label_values(&["recovered"])
            .observe(self.waited_secs);
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
    fn test_emit() {
        use fvm_ipld_encoding::RawBytes;
        use fvm_shared::address::Address;
        use fvm_shared::econ::TokenAmount;

        let message = Message {
            version: 1,
            from: Address::new_id(1),
            to: Address::new_id(2),
            sequence: 1,
            value: TokenAmount::from_atto(1),
            method_num: 1,
            params: RawBytes::default(),
            gas_limit: 1,
            gas_fee_cap: TokenAmount::from_atto(1),
            gas_premium: TokenAmount::from_atto(1),
        };

        emit(MsgExec {
            purpose: MsgExecPurpose::Check,
            height: 1,
            duration: 1.0,
            exit_code: 1,
            message: message.clone(),
        });
        let hash = vec![0x01, 0x02, 0x03];

        emit(CheckpointCreated {
            height: 1,
            hash: HexEncodableBlockHash(hash.clone()),
            msg_count: 2,
            config_number: 3,
        });

        emit(CheckpointSigned {
            role: CheckpointSignedRole::Own,
            height: 1,
            hash: HexEncodableBlockHash(hash.clone()),
            validator: Address::new_id(1),
        });

        emit(F3CacheWaitStuck {
            epoch: 1,
            waited_secs: 120.0,
        });
        emit(F3CacheWaitRecovered {
            epoch: 1,
            waited_secs: 2.0,
        });
    }
}
