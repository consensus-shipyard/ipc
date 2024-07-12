// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, Recordable, TraceLevel,
    Traceable,
};
use prometheus::{register_histogram, Histogram, Registry};

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

impl_traceables!(
    TraceLevel::Info,
    "Execution",
    MsgExecCheck<'a>,
    MsgExecEstimate<'a>,
    MsgExecApply<'a>,
    MsgExecCall<'a>
);

macro_rules! message_exec_struct {
    ($($name:ident),*) => {
        $(
            #[warn(dead_code)]
            #[derive(std::fmt::Debug)]
            pub struct $name<'a> {
                pub height: i64,
                pub from: &'a str,
                pub to: &'a str,
                pub value: &'a str,
                pub method_num: u64,
                pub gas_limit: u64,
                pub gas_price: &'a str,
                pub params: &'a [u8],
                pub nonce: u64,
                pub duration: f64,
                pub exit_code: u32,
            }
        )*
    };
}

impl Recordable for MsgExecCheck<'_> {
    fn record_metrics(&self) {
        EXEC_FVM_CHECK_EXECUTION_TIME_SECS.observe(self.duration);
    }
}

impl Recordable for MsgExecEstimate<'_> {
    fn record_metrics(&self) {
        EXEC_FVM_ESTIMATE_EXECUTION_TIME_SECS.observe(self.duration);
    }
}

impl Recordable for MsgExecApply<'_> {
    fn record_metrics(&self) {
        EXEC_FVM_APPLY_EXECUTION_TIME_SECS.observe(self.duration);
    }
}

impl Recordable for MsgExecCall<'_> {
    fn record_metrics(&self) {
        EXEC_FVM_CALL_EXECUTION_TIME_SECS.observe(self.duration);
    }
}

message_exec_struct!(MsgExecCheck, MsgExecEstimate, MsgExecApply, MsgExecCall);

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
        emit(MsgExecCheck {
            height: 1,
            from: "from",
            to: "to",
            value: "value",
            method_num: 1,
            gas_limit: 1,
            gas_price: "gas_price",
            params: &[1, 2, 3],
            nonce: 1,
            duration: 1.0,
            exit_code: 1,
        });

        emit(MsgExecEstimate {
            height: 1,
            from: "from",
            to: "to",
            value: "value",
            method_num: 1,
            gas_limit: 1,
            gas_price: "gas_price",
            params: &[1, 2, 3],
            nonce: 1,
            duration: 1.0,
            exit_code: 1,
        });

        emit(MsgExecApply {
            height: 1,
            from: "from",
            to: "to",
            value: "value",
            method_num: 1,
            gas_limit: 1,
            gas_price: "gas_price",
            params: &[1, 2, 3],
            nonce: 1,
            duration: 1.0,
            exit_code: 1,
        });

        emit(MsgExecCall {
            height: 1,
            from: "from",
            to: "to",
            value: "value",
            method_num: 1,
            gas_limit: 1,
            gas_price: "gas_price",
            params: &[1, 2, 3],
            nonce: 1,
            duration: 1.0,
            exit_code: 1,
        })
    }
}
