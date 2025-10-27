// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Observability and metrics for the F3 proof service

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, Recordable, TraceLevel,
    Traceable,
};
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, HistogramVec,
    IntCounterVec, IntGauge, Registry,
};

/// Operation status for metrics
#[derive(Debug, Clone, Copy)]
pub enum OperationStatus {
    Success,
    Failure,
}

impl OperationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationStatus::Success => "success",
            OperationStatus::Failure => "failure",
        }
    }
}

register_metrics! {
    // F3 Certificate Operations
    F3_CERT_FETCH_TOTAL: IntCounterVec
        = register_int_counter_vec!("f3_cert_fetch_total", "F3 certificate fetch attempts", &["status"]);
    F3_CERT_FETCH_LATENCY_SECS: HistogramVec
        = register_histogram_vec!("f3_cert_fetch_latency_secs", "F3 certificate fetch latency", &["status"]);
    F3_CERT_VALIDATION_TOTAL: IntCounterVec
        = register_int_counter_vec!("f3_cert_validation_total", "F3 certificate validations", &["status"]);
    F3_CERT_VALIDATION_LATENCY_SECS: HistogramVec
        = register_histogram_vec!("f3_cert_validation_latency_secs", "F3 certificate validation latency", &["status"]);
    F3_CURRENT_INSTANCE: IntGauge
        = register_int_gauge!("f3_current_instance", "Current F3 instance in light client state");

    // Proof Generation
    PROOF_GENERATION_TOTAL: IntCounterVec
        = register_int_counter_vec!("proof_generation_total", "Proof bundle generation attempts", &["status"]);
    PROOF_GENERATION_LATENCY_SECS: HistogramVec
        = register_histogram_vec!("proof_generation_latency_secs", "Proof bundle generation latency", &["status"]);
    PROOF_BUNDLE_SIZE_BYTES: HistogramVec
        = register_histogram_vec!("proof_bundle_size_bytes", "Proof bundle sizes", &["type"]);

    // Cache Operations
    CACHE_SIZE: IntGauge
        = register_int_gauge!("proof_cache_size", "Number of proofs in cache");
    CACHE_LAST_COMMITTED: IntGauge
        = register_int_gauge!("proof_cache_last_committed", "Last committed F3 instance");
    CACHE_HIGHEST_CACHED: IntGauge
        = register_int_gauge!("proof_cache_highest_cached", "Highest cached F3 instance");
    CACHE_HIT_TOTAL: IntCounterVec
        = register_int_counter_vec!("proof_cache_hit_total", "Cache hits/misses", &["result"]);
    CACHE_INSERT_TOTAL: IntCounterVec
        = register_int_counter_vec!("proof_cache_insert_total", "Cache insertions", &["status"]);
}

impl_traceables!(
    TraceLevel::Info,
    "F3ProofService",
    F3CertificateFetched,
    F3CertificateValidated,
    ProofBundleGenerated,
    ProofCached
);

#[derive(Debug)]
pub struct F3CertificateFetched {
    pub instance: u64,
    pub ec_chain_len: usize,
    pub status: OperationStatus,
    pub latency: f64,
}

impl Recordable for F3CertificateFetched {
    fn record_metrics(&self) {
        F3_CERT_FETCH_TOTAL
            .with_label_values(&[self.status.as_str()])
            .inc();
        F3_CERT_FETCH_LATENCY_SECS
            .with_label_values(&[self.status.as_str()])
            .observe(self.latency);
    }
}

#[derive(Debug)]
pub struct F3CertificateValidated {
    pub instance: u64,
    pub new_instance: u64,
    pub power_table_size: usize,
    pub status: OperationStatus,
    pub latency: f64,
}

impl Recordable for F3CertificateValidated {
    fn record_metrics(&self) {
        F3_CERT_VALIDATION_TOTAL
            .with_label_values(&[self.status.as_str()])
            .inc();
        F3_CERT_VALIDATION_LATENCY_SECS
            .with_label_values(&[self.status.as_str()])
            .observe(self.latency);
        if matches!(self.status, OperationStatus::Success) {
            F3_CURRENT_INSTANCE.set(self.new_instance as i64);
        }
    }
}

#[derive(Debug)]
pub struct ProofBundleGenerated {
    pub instance: u64,
    pub highest_epoch: i64,
    pub storage_proofs: usize,
    pub event_proofs: usize,
    pub witness_blocks: usize,
    pub bundle_size_bytes: usize,
    pub status: OperationStatus,
    pub latency: f64,
}

impl Recordable for ProofBundleGenerated {
    fn record_metrics(&self) {
        PROOF_GENERATION_TOTAL
            .with_label_values(&[self.status.as_str()])
            .inc();
        PROOF_GENERATION_LATENCY_SECS
            .with_label_values(&[self.status.as_str()])
            .observe(self.latency);
        if matches!(self.status, OperationStatus::Success) {
            PROOF_BUNDLE_SIZE_BYTES
                .with_label_values(&["total"])
                .observe(self.bundle_size_bytes as f64);
        }
    }
}

#[derive(Debug)]
pub struct ProofCached {
    pub instance: u64,
    pub cache_size: usize,
    pub highest_cached: u64,
}

impl Recordable for ProofCached {
    fn record_metrics(&self) {
        CACHE_SIZE.set(self.cache_size as i64);
        CACHE_HIGHEST_CACHED.set(self.highest_cached as i64);
        CACHE_INSERT_TOTAL.with_label_values(&["success"]).inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipc_observability::emit;

    #[test]
    fn test_metrics_registration() {
        let registry = Registry::new();
        register_metrics(&registry).unwrap();
    }

    #[test]
    fn test_emit_f3_metrics() {
        emit(F3CertificateFetched {
            instance: 100,
            ec_chain_len: 1,
            status: OperationStatus::Success,
            latency: 0.5,
        });

        emit(F3CertificateValidated {
            instance: 100,
            new_instance: 101,
            power_table_size: 13,
            status: OperationStatus::Success,
            latency: 0.1,
        });
    }

    #[test]
    fn test_emit_proof_metrics() {
        emit(ProofBundleGenerated {
            instance: 100,
            highest_epoch: 1000,
            storage_proofs: 1,
            event_proofs: 2,
            witness_blocks: 15,
            bundle_size_bytes: 15000,
            status: OperationStatus::Success,
            latency: 1.2,
        });

        emit(ProofCached {
            instance: 100,
            cache_size: 5,
            highest_cached: 104,
        });
    }
}
