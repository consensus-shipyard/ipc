// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{actor_error, ActorError, Map2, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::{Address, Payload};
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidatorStats {
    pub blocks_committed: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    pub tracking_since: ChainEpoch,
    pub consensus: Cid, // ConsensusData
}

pub type ConsensusData<BS> = Map2<BS, Address, ValidatorStats>;

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<State, ActorError> {
        let state = State {
            tracking_since: 0,
            consensus: ConsensusData::flush_empty(store, DEFAULT_HAMT_CONFIG)?,
        };
        Ok(state)
    }

    /// Returns the pending activity rollup.
    pub fn pending_activity_rollup(
        &self,
        rt: &impl Runtime,
    ) -> Result<ipc_actors_abis::checkpointing_facet::FullActivityRollup, ActorError> {
        let consensus = {
            let cid = &rt.state::<State>()?.consensus;
            ConsensusData::load(rt.store(), cid, DEFAULT_HAMT_CONFIG, "consensus")
        }?;

        // Populate the rollup struct.
        let mut rollup = ipc_actors_abis::checkpointing_facet::FullActivityRollup::default();
        consensus.for_each(|validator_addr, validator_stats| {
            rollup.consensus.stats.total_active_validators += 1;
            rollup.consensus.stats.total_num_blocks_committed += validator_stats.blocks_committed;

            // All addresses in state are f410 addresses, so we can safely cast to ethers addresses.
            let addr = {
                let Payload::Delegated(delegated) = validator_addr.payload() else {
                    return Err(actor_error!(illegal_state; "illegal address in state"));
                };
                let slice = delegated.subaddress();
                ethers::types::Address::from_slice(&slice[0..20])
            };

            let item = ipc_actors_abis::checkpointing_facet::ValidatorData {
                validator: addr,
                blocks_committed: validator_stats.blocks_committed,
            };

            rollup.consensus.data.push(item);
            Ok(())
        })?;

        Ok(rollup)
    }
}
