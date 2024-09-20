// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::payload::{Ballot, PowerTable, Vote};
use crate::vote::{Error, Weight};
use crate::BlockHeight;
use fendermint_vm_genesis::ValidatorKey;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};

pub(crate) trait VoteStore {
    /// Get the earliest block height of the votes stored
    fn earliest_vote_height(&self) -> Result<Option<BlockHeight>, Error>;

    /// Get the latest block height of the votes stored
    fn latest_vote_height(&self) -> Result<Option<BlockHeight>, Error>;

    /// Store the vote at the target height
    fn store_vote(&mut self, height: BlockHeight, vote: Vote) -> Result<(), Error>;

    /// Checks if the validator has voted at the target block height
    fn has_voted(&self, height: &BlockHeight, validator: &ValidatorKey) -> Result<bool, Error>;

    /// Get the votes at the height.
    fn get_votes_at_height(&self, height: BlockHeight) -> Result<VoteAgg, Error>;

    /// Purge all votes at the specific height. This could be the target height has reached a
    /// quorum and the history is not needed.
    fn purge_votes_at_height(&mut self, height: BlockHeight) -> Result<(), Error>;
}

pub(crate) struct InMemoryVoteStore {
    votes: BTreeMap<BlockHeight, HashMap<ValidatorKey, Vote>>,
}

impl VoteStore for InMemoryVoteStore {
    fn earliest_vote_height(&self) -> Result<Option<BlockHeight>, Error> {
        Ok(self.votes.first_key_value().map(|(k, _)| *k))
    }

    fn latest_vote_height(&self) -> Result<Option<BlockHeight>, Error> {
        Ok(self.votes.last_key_value().map(|(k, _)| *k))
    }

    fn store_vote(&mut self, height: BlockHeight, vote: Vote) -> Result<(), Error> {
        match self.votes.entry(height) {
            Entry::Vacant(_) => {
                let mut map = HashMap::new();
                map.insert(vote.voter(), vote);
                self.votes.insert(height, map);
            }
            Entry::Occupied(mut v) => {
                let key = vote.voter();
                v.get_mut().insert(key, vote);
            }
        }
        Ok(())
    }

    fn has_voted(&self, height: &BlockHeight, validator: &ValidatorKey) -> Result<bool, Error> {
        let Some(votes) = self.votes.get(height) else {
            return Ok(false);
        };
        Ok(votes.contains_key(validator))
    }

    fn get_votes_at_height(&self, height: BlockHeight) -> Result<VoteAgg, Error> {
        let votes = self
            .votes
            .get(&height)
            .map(|v| v.values().collect())
            .unwrap_or_default();
        Ok(VoteAgg::new(votes))
    }

    fn purge_votes_at_height(&mut self, height: BlockHeight) -> Result<(), Error> {
        self.votes.remove(&height);
        Ok(())
    }
}

/// The aggregated votes  from different validators.
pub(crate) struct VoteAgg<'a>(Vec<&'a Vote>);

impl<'a> VoteAgg<'a> {
    pub fn new(votes: Vec<&'a Vote>) -> Self {
        Self(votes)
    }

    pub fn ballot_weights(&self, power_table: &PowerTable) -> Vec<(&Ballot, Weight)> {
        let mut votes: Vec<(&Ballot, Weight)> = Vec::new();

        for v in self.0.iter() {
            let validator = v.voter();

            let power = power_table.get(&validator).cloned().unwrap_or(0);
            if power == 0 {
                continue;
            }

            if let Some(w) = votes.iter_mut().find(|w| w.0 == v.ballot()) {
                w.1 += power;
            } else {
                votes.push((v.ballot(), power))
            }
        }

        votes
    }
}
