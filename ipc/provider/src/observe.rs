// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt;

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, Recordable, TraceLevel,
    Traceable,
};
use prometheus::{register_int_gauge, IntGauge, Registry};

register_metrics! {
    BOTTOMUP_CHECKPOINT_FINALIZED_HEIGHT: IntGauge
        = register_int_gauge!("bottomup_checkpoint_finalized_height", "Height of the checkpoint finalized");
}

impl_traceables!(TraceLevel::Info, "Bottomup", CheckpointFinalized);

// Hex encodable block hash.
pub struct HexEncodableBlockHash(pub Vec<u8>);

impl fmt::Debug for HexEncodableBlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
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
        let hash = vec![0x01, 0x02, 0x03];

        emit(CheckpointFinalized {
            height: 1,
            hash: HexEncodableBlockHash(hash.clone()),
        });
    }
}
