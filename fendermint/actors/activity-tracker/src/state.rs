// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{ActorError, Map2, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

pub type BlockCommittedMap<BS> = Map2<BS, Address, BlockCommitted>;
pub type BlockCommitted = u64;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ValidatorSummary {
    pub validator: Address,
    pub block_committed: BlockCommitted,
    pub metadata: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    pub start_height: ChainEpoch,
    pub blocks_committed: Cid, // BlockCommittedMap
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<State, ActorError> {
        let mut deployers_map = BlockCommittedMap::empty(store, DEFAULT_HAMT_CONFIG, "empty");
        Ok(State {
            start_height: 0,
            blocks_committed: deployers_map.flush()?,
        })
    }

    pub fn reset_start_height(&mut self, rt: &impl Runtime) -> Result<(), ActorError> {
        self.start_height = rt.curr_epoch();
        Ok(())
    }

    pub fn purge_validator_block_committed(&mut self, rt: &impl Runtime) -> Result<(), ActorError> {
        let all_validators = self.validator_activities(rt)?;
        let mut validators = BlockCommittedMap::load(
            rt.store(),
            &self.blocks_committed,
            DEFAULT_HAMT_CONFIG,
            "verifiers",
        )?;

        for v in all_validators {
            validators.delete(&v.validator)?;
        }

        self.blocks_committed = validators.flush()?;

        Ok(())
    }

    pub fn incr_validator_block_committed(
        &mut self,
        rt: &impl Runtime,
        validator: &Address,
    ) -> Result<(), ActorError> {
        let mut validators = BlockCommittedMap::load(
            rt.store(),
            &self.blocks_committed,
            DEFAULT_HAMT_CONFIG,
            "verifiers",
        )?;

        let v = if let Some(v) = validators.get(validator)? {
            *v + 1
        } else {
            1
        };

        validators.set(validator, v)?;

        self.blocks_committed = validators.flush()?;

        Ok(())
    }

    pub fn validator_activities(
        &self,
        rt: &impl Runtime,
    ) -> Result<Vec<ValidatorSummary>, ActorError> {
        let mut result = vec![];

        let validators = BlockCommittedMap::load(
            rt.store(),
            &self.blocks_committed,
            DEFAULT_HAMT_CONFIG,
            "verifiers",
        )?;
        validators.for_each(|k, v| {
            result.push(ValidatorSummary {
                validator: k,
                block_committed: *v,
                metadata: vec![],
            });
            Ok(())
        })?;

        Ok(result)
    }
}
