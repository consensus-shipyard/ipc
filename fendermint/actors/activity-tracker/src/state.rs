// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{ActorError, Map2, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidatorStats {
    pub blocks_committed: u64,
}

pub type ValidatorMap<BS> = Map2<BS, Address, ValidatorStats>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ValidatorData {
    pub validator: Address,
    pub stats: ValidatorStats,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    pub cycle_start: ChainEpoch,
    pub consensus: Cid, // BlockCommittedMap
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<State, ActorError> {
        let mut deployers_map = ValidatorMap::empty(store, DEFAULT_HAMT_CONFIG, "empty");
        Ok(State {
            cycle_start: 0,
            consensus: deployers_map.flush()?,
        })
    }

    pub fn reset_cycle_height(&mut self, rt: &impl Runtime) -> Result<(), ActorError> {
        self.cycle_start = rt.curr_epoch();
        Ok(())
    }

    pub fn purge_validator_block_committed(&mut self, rt: &impl Runtime) -> Result<(), ActorError> {
        let all_validators = self.validator_activities(rt)?;
        let mut validators = ValidatorMap::load(
            rt.store(),
            &self.consensus,
            DEFAULT_HAMT_CONFIG,
            "verifiers",
        )?;

        for v in all_validators {
            validators.delete(&v.validator)?;
        }

        self.consensus = validators.flush()?;

        Ok(())
    }

    pub fn incr_validator_block_committed(
        &mut self,
        rt: &impl Runtime,
        validator: &Address,
    ) -> Result<(), ActorError> {
        let mut validators = ValidatorMap::load(
            rt.store(),
            &self.consensus,
            DEFAULT_HAMT_CONFIG,
            "verifiers",
        )?;

        let mut v = validators.get(validator)?.cloned().unwrap_or_default();
        v.blocks_committed += 1;

        validators.set(validator, v)?;

        self.consensus = validators.flush()?;

        Ok(())
    }

    pub fn validator_activities(
        &self,
        rt: &impl Runtime,
    ) -> Result<Vec<ValidatorData>, ActorError> {
        let validators = ValidatorMap::load(
            rt.store(),
            &self.consensus,
            DEFAULT_HAMT_CONFIG,
            "verifiers",
        )?;

        let mut result = vec![];

        validators.for_each(|k, v| {
            let detail = ValidatorData {
                validator: k,
                stats: v.clone(),
            };
            result.push(detail);
            Ok(())
        })?;

        Ok(result)
    }
}
