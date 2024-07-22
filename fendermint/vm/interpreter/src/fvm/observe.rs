// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, Recordable, TraceLevel,
    Traceable,
};
use prometheus::{register_histogram, Histogram, Registry};

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
    }
}
