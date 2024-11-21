// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the current blockchain block mining activities and propagates to the parent subnet if
//! needed.

pub mod actor;
mod merkle;

use crate::fvm::activity::merkle::MerkleProofGen;
use fendermint_actor_activity_tracker::ValidatorData;
use fendermint_crypto::PublicKey;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ActivityDetails<T> {
    details: Vec<T>,
    cycle_start: ChainEpoch,
}

/// Tracks the validator activities in the current blockchain
pub trait ValidatorActivityTracker {
    type ValidatorSummaryDetail: Clone + Debug;

    /// Mark the validator has mined the target block.
    fn record_block_committed(&mut self, validator: PublicKey) -> anyhow::Result<()>;

    /// Get the validators activities summary since the checkpoint height
    fn commit_activity(
        &mut self,
    ) -> anyhow::Result<ActivityDetails<Self::ValidatorSummaryDetail>>;

    /// Purge the current validator activities summary
    fn purge_activities(&mut self) -> anyhow::Result<()>;
}

impl ActivityDetails<ValidatorData> {
    pub fn new(mut details: Vec<ValidatorData>, cycle_start: ChainEpoch) -> Self {
        details.sort_by(|a, b| {
            let cmp = a.validator.cmp(&b.validator);
            if cmp.is_eq() {
                // Address will be unique, do this just in case equal
                a.stats.blocks_committed.cmp(&b.stats.blocks_committed)
            } else {
                cmp
            }
        });
        Self {
            details,
            cycle_start,
        }
    }

    pub fn commitment(&self) -> anyhow::Result<Vec<u8>> {
        let gen = MerkleProofGen::new(self.details.as_slice())?;
        Ok(gen.root().to_fixed_bytes().to_vec())
    }
}

impl<T> ActivityDetails<T> {
    pub fn elapsed(&self, height: ChainEpoch) -> ChainEpoch {
        height.saturating_sub(self.cycle_start)
    }

    pub fn active_validators(&self) -> usize {
        self.details.len()
    }

    pub fn details(&self) -> &[T] {
        self.details.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::fvm::activities::ActivityDetails;
    use fendermint_actor_activity_tracker::{ValidatorData, ValidatorStats};
    use fendermint_vm_actor_interface::eam::EthAddress;
    use fvm_shared::address::Address;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    use std::str::FromStr;

    #[test]
    fn test_commitment() {
        let mut v = vec![
            ValidatorData {
                validator: Address::from(EthAddress::from(
                    ethers::types::Address::from_str("0xB29C00299756135ec5d6A140CA54Ec77790a99d6")
                        .unwrap(),
                )),
                stats: ValidatorStats {
                    blocks_committed: 1,
                },
            },
            ValidatorData {
                validator: Address::from(EthAddress::from(
                    ethers::types::Address::from_str("0x28345a43c2fBae4412f0AbadFa06Bd8BA3f58867")
                        .unwrap(),
                )),
                stats: ValidatorStats {
                    blocks_committed: 2,
                },
            },
            ValidatorData {
                validator: Address::from(EthAddress::from(
                    ethers::types::Address::from_str("0x1A79385eAd0e873FE0C441C034636D3Edf7014cC")
                        .unwrap(),
                )),
                stats: ValidatorStats {
                    blocks_committed: 10,
                },
            },
            ValidatorData {
                validator: Address::from(EthAddress::from(
                    ethers::types::Address::from_str("0x76B9d5a35C46B1fFEb37aadf929f1CA63a26A829")
                        .unwrap(),
                )),
                stats: ValidatorStats {
                    blocks_committed: 4,
                },
            },
            ValidatorData {
                validator: Address::from(EthAddress::from(
                    ethers::types::Address::from_str("0x3c5cc76b07cb02a372e647887bD6780513659527")
                        .unwrap(),
                )),
                stats: ValidatorStats {
                    blocks_committed: 3,
                },
            },
        ];

        for _ in 0..10 {
            v.shuffle(&mut thread_rng());
            let details = ActivityDetails::new(v.clone(), 10);
            assert_eq!(
                hex::encode(details.commitment().unwrap()),
                "5519955f33109df3338490473cb14458640efdccd4df05998c4c439738280ab0"
            );
        }
    }
}
