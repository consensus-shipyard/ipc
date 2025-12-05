// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Simple Externs implementation for testing and no-op module.

use fvm::externs::{Chain, Consensus, Externs, Rand};
use fvm_shared::clock::ChainEpoch;

/// A minimal no-op implementation of Externs.
///
/// This is used by the NoOpModuleBundle and for testing.
/// All methods return errors or empty values.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpExterns;

impl Rand for NoOpExterns {
    fn get_chain_randomness(&self, _round: ChainEpoch) -> anyhow::Result<[u8; 32]> {
        anyhow::bail!("randomness not implemented in NoOpExterns")
    }

    fn get_beacon_randomness(&self, _round: ChainEpoch) -> anyhow::Result<[u8; 32]> {
        anyhow::bail!("beacon randomness not implemented in NoOpExterns")
    }
}

impl Consensus for NoOpExterns {
    fn verify_consensus_fault(
        &self,
        _h1: &[u8],
        _h2: &[u8],
        _extra: &[u8],
    ) -> anyhow::Result<(Option<fvm_shared::consensus::ConsensusFault>, i64)> {
        anyhow::bail!("consensus fault verification not implemented in NoOpExterns")
    }
}

impl Chain for NoOpExterns {
    fn get_tipset_cid(&self, _epoch: ChainEpoch) -> anyhow::Result<cid::Cid> {
        anyhow::bail!("tipset CID not implemented in NoOpExterns")
    }
}

impl Externs for NoOpExterns {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_op_externs_default() {
        let _externs = NoOpExterns::default();
    }

    #[test]
    fn test_no_op_externs_clone() {
        let externs1 = NoOpExterns;
        let _externs2 = externs1;
        let _externs3 = externs1; // NoOpExterns is Copy
    }

    #[test]
    fn test_no_op_externs_randomness() {
        let externs = NoOpExterns;
        assert!(externs.get_chain_randomness(0).is_err());
        assert!(externs.get_beacon_randomness(0).is_err());
    }

    #[test]
    fn test_no_op_externs_consensus() {
        let externs = NoOpExterns;
        assert!(externs.verify_consensus_fault(&[], &[], &[]).is_err());
    }

    #[test]
    fn test_no_op_externs_chain() {
        let externs = NoOpExterns;
        assert!(externs.get_tipset_cid(0).is_err());
    }
}
