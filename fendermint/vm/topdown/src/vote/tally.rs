// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::payload::{Ballot, PowerTable, Vote};
use crate::vote::store::VoteStore;
use crate::vote::{Error, Weight};
use crate::BlockHeight;
use fendermint_vm_genesis::ValidatorKey;
use std::collections::HashMap;

/// VoteTally aggregates different votes received from various validators in the network
pub(crate) struct VoteTally<S> {
    /// Current validator weights. These are the ones who will vote on the blocks,
    /// so these are the weights which need to form a quorum.
    power_table: PowerTable,

    /// Index votes received by height and hash, which makes it easy to look up
    /// all the votes for a given block hash and also to verify that a validator
    /// isn't equivocating by trying to vote for two different things at the
    /// same height.
    votes: S,

    /// The latest height that was voted to be finalized and committed to child blockchian
    last_finalized_height: BlockHeight,
}

impl<S: VoteStore> VoteTally<S> {
    /// Initialize the vote tally from the current power table
    /// and the last finalized block from the ledger.
    pub fn new(
        power_table: Vec<(ValidatorKey, Weight)>,
        last_finalized_height: BlockHeight,
        mut store: S,
    ) -> Result<Self, Error> {
        // purge votes that already committed, no need keep them
        if let Some(h) = store.earliest_vote_height()? {
            for i in h..=last_finalized_height {
                store.purge_votes_at_height(i)?;
            }
        }
        Ok(Self {
            power_table: HashMap::from_iter(power_table),
            votes: store,
            last_finalized_height,
        })
    }

    fn power_table(&self) -> &HashMap<ValidatorKey, Weight> {
        &self.power_table
    }

    /// Check that a validator key is currently part of the power table.
    fn has_power(&self, validator_key: &ValidatorKey) -> bool {
        // For consistency consider validators without power unknown.
        match self.power_table.get(validator_key) {
            None => false,
            Some(weight) => *weight > 0,
        }
    }

    /// Calculate the minimum weight needed for a proposal to pass with the current membership.
    ///
    /// This is inclusive, that is, if the sum of weight is greater or equal to this, it should pass.
    /// The equivalent formula can be found in CometBFT [here](https://github.com/cometbft/cometbft/blob/a8991d63e5aad8be82b90329b55413e3a4933dc0/types/vote_set.go#L307).
    fn quorum_threshold(&self) -> Weight {
        let total_weight: Weight = self.power_table.values().sum();
        total_weight * 2 / 3 + 1
    }

    /// Return the height of the first entry in the chain.
    ///
    /// This is the block that was finalized *in the ledger*.
    fn last_finalized_height(&self) -> BlockHeight {
        self.last_finalized_height
    }

    /// Add a vote we received.
    ///
    /// Returns `true` if this vote was added, `false` if it was ignored as a
    /// duplicate or a height we already finalized, and an error if it's an
    /// equivocation or from a validator we don't know.
    pub fn add_vote(&mut self, vote: Vote) -> Result<bool, Error> {
        let validator = vote.voter();
        let parent_height = vote.ballot().parent_height();

        if !self.has_power(&validator) {
            tracing::error!(
                validator = validator.to_string(),
                "validator unknown or has no power"
            );
            return Err(Error::UnpoweredValidator);
        }

        if parent_height < self.last_finalized_height() {
            tracing::debug!(
                parent_height,
                last_finalized_height = self.last_finalized_height(),
                validator = validator.to_string(),
                "reject vote as parent height finalized"
            );
            return Ok(false);
        }

        if self.votes.has_voted(&parent_height, &validator)? {
            tracing::error!(
                parent_height,
                validator = validator.to_string(),
                "equivocation by validator"
            );
            return Err(Error::Equivocation);
        }

        self.votes.store_vote(parent_height, vote)?;

        Ok(true)
    }

    /// Find a block on the (from our perspective) finalized chain that gathered enough votes from validators.
    pub fn find_quorum(&self) -> Result<Option<Ballot>, Error> {
        let quorum_threshold = self.quorum_threshold();
        let Some(max_height) = self.votes.latest_vote_height()? else {
            tracing::info!("vote store has no vote yet, skip finding quorum");
            return Ok(None);
        };

        for h in ((self.last_finalized_height + 1)..max_height).rev() {
            let votes = self.votes.get_votes_at_height(h)?;

            for (ballot, weight) in votes.ballot_weights(&self.power_table) {
                tracing::info!(
                    height = h,
                    ballot = ballot.to_string(),
                    weight,
                    quorum_threshold,
                    "ballot and weight"
                );

                if weight >= quorum_threshold {
                    return Ok(Some(ballot.clone()));
                }
            }

            tracing::info!(height = h, "no quorum found");
        }

        Ok(None)
    }

    /// Call when a new finalized block is added to the ledger, to clear out all preceding blocks.
    ///
    /// After this operation the minimum item in the chain will the new finalized block.
    pub fn set_finalized(&mut self, block_height: BlockHeight) -> Result<(), Error> {
        self.votes.purge_votes_at_height(block_height)?;
        self.last_finalized_height = block_height;
        Ok(())
    }

    /// Overwrite the power table after it has changed to a new snapshot.
    ///
    /// This method expects absolute values, it completely replaces the existing powers.
    pub fn set_power_table(&mut self, power_table: Vec<(ValidatorKey, Weight)>) {
        let power_table = HashMap::from_iter(power_table);
        // We don't actually have to remove the votes of anyone who is no longer a validator,
        // we just have to make sure to handle the case when they are not in the power table.
        self.power_table = power_table;
    }

    /// Update the power table after it has changed with changes.
    ///
    /// This method expects only the updated values, leaving everyone who isn't in it untouched
    pub fn update_power_table(&mut self, power_updates: Vec<(ValidatorKey, Weight)>) {
        if power_updates.is_empty() {
            return;
        }
        // We don't actually have to remove the votes of anyone who is no longer a validator,
        // we just have to make sure to handle the case when they are not in the power table.
        for (vk, w) in power_updates {
            if w == 0 {
                self.power_table.remove(&vk);
            } else {
                *self.power_table.entry(vk).or_default() = w;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BlockHeight;
    use ipc_ipld_resolver::ValidatorKey;
    use libp2p::identity::Keypair;
    use crate::vote::payload::Vote;

    fn convert_key(key: &Keypair) -> libp2p::identity::secp256k1::Keypair {
        let key = key.clone();
        key.try_into_secp256k1().unwrap()
    }

    fn random_validator_key() -> (Keypair, ValidatorKey) {
        let key_pair = Keypair::generate_secp256k1();
        let public_key = key_pair.public();
        (key_pair, ValidatorKey::from(public_key))
    }

    fn random_vote(height: BlockHeight) -> Vote {
        let rand_bytes = |u: usize| {
            let mut v = vec![];
            for _ in 0..u {
                v.push(rand::random::<u8>());
            }
            v
        };
        let hash = rand_bytes(32);
        let commitment = rand_bytes(64);


        Vote::v1(height, hash, commitment)
    }

    #[test]
    fn new_validators_joined_void_previous_quorum() {
        atomically_or_err(|| {
            let validators = (0..3)
                .map(|_| random_validator_key())
                .collect::<Vec<(Keypair, ValidatorKey)>>();
            let powers = validators
                .iter()
                .map(|v| (v.1.clone(), 1))
                .collect::<Vec<_>>();

            let vote_tally = VoteTally::new(powers.clone(), 0);

            let votes = (11..15).map(random_vote).collect::<Vec<_>>();

            for v in votes.clone() {
                vote_tally.add_block(v)?;
            }

            for validator in validators {
                for v in votes.iter() {
                    let signed = SignedVote::signed(&convert_key(&validator.0), v).unwrap();
                    assert!(vote_tally.add_vote(signed)?);
                }
            }

            let (vote, cert) = vote_tally.find_quorum()?.unwrap();
            assert_eq!(vote, votes[votes.len() - 1]);
            cert.validate_power_table::<3, 2>(&vote.ballot().unwrap(), im::HashMap::from(powers))
                .unwrap();

            let new_powers = (0..3)
                .map(|_| (random_validator_key().1.clone(), 1))
                .collect::<Vec<_>>();
            vote_tally.update_power_table(new_powers)?;
            assert_eq!(vote_tally.find_quorum()?, None);
            Ok(())
        })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn new_validators_left_formed_quorum() {
        atomically_or_err(|| {
            let validators = (0..3)
                .map(|_| random_validator_key())
                .collect::<Vec<(Keypair, ValidatorKey)>>();
            let mut powers = validators
                .iter()
                .map(|v| (v.1.clone(), 1))
                .collect::<Vec<_>>();
            let extra_validators = (0..3)
                .map(|_| random_validator_key())
                .collect::<Vec<(Keypair, ValidatorKey)>>();
            for v in extra_validators.iter() {
                powers.push((v.1.clone(), 1));
            }

            let vote_tally = VoteTally::new(powers.clone(), 0);

            let votes = (11..15).map(random_vote).collect::<Vec<_>>();

            for v in votes.clone() {
                vote_tally.add_block(v)?;
            }

            for validator in validators {
                for v in votes.iter() {
                    let signed = SignedVote::signed(&convert_key(&validator.0), v).unwrap();
                    assert!(vote_tally.add_vote(signed)?);
                }
            }

            assert_eq!(vote_tally.find_quorum()?, None);

            let new_powers = extra_validators
                .into_iter()
                .map(|v| (v.1.clone(), 0))
                .collect::<Vec<_>>();
            vote_tally.update_power_table(new_powers)?;
            let powers = vote_tally.power_table()?;
            let (vote, cert) = vote_tally.find_quorum()?.unwrap();
            assert_eq!(vote, votes[votes.len() - 1]);
            cert.validate_power_table::<3, 2>(&vote.ballot().unwrap(), powers)
                .unwrap();

            Ok(())
        })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn simple_3_validators_no_quorum() {
        atomically_or_err(|| {
            let validators = (0..3)
                .map(|_| random_validator_key())
                .collect::<Vec<(Keypair, ValidatorKey)>>();
            let powers = validators
                .iter()
                .map(|v| (v.1.clone(), 1))
                .collect::<Vec<_>>();

            let vote_tally = VoteTally::new(powers.clone(), 0);

            let votes = [random_vote(10), random_vote(10)];

            vote_tally.add_block(votes[0].clone())?;

            let signed = SignedVote::signed(&convert_key(&validators[0].0), &votes[0]).unwrap();
            assert!(vote_tally.add_vote(signed)?);
            let signed = SignedVote::signed(&convert_key(&validators[1].0), &votes[0]).unwrap();
            assert!(vote_tally.add_vote(signed)?);
            let signed = SignedVote::signed(&convert_key(&validators[2].0), &votes[1]).unwrap();
            assert!(vote_tally.add_vote(signed)?);

            assert!(vote_tally.find_quorum()?.is_none());

            Ok(())
        })
            .await
            .unwrap();
    }
}
