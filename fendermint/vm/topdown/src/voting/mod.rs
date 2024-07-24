// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod payload;
pub mod quorum;

use crate::voting::quorum::{MultiSigCert, ValidatorSignatures};
use async_stm::{abort, atomically_or_err, retry, Stm, StmResult, TVar};
use im::OrdMap;
use std::borrow::Borrow;
use std::{fmt::Debug, time::Duration};

use crate::{BlockHeight, Bytes};

// Usign this type because it's `Hash`, unlike the normal `libsecp256k1::PublicKey`.
use crate::voting::payload::{SignedVote, TopdownVote};
pub use ipc_ipld_resolver::ValidatorKey;

pub type Weight = u64;
pub type Signature = Bytes;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the last finalized block has not been set")]
    Uninitialized,

    #[error("failed to extend chain; height going backwards, current height {0}, got {1}")]
    UnexpectedBlock(BlockHeight, BlockHeight),

    #[error("validator unknown or has no power")]
    UnpoweredValidator,

    #[error("equivocation by validator")]
    Equivocation,

    #[error("validator vote is invalidated")]
    VoteCannotBeValidated,

    #[error("validator cannot sign vote")]
    CannotSignVote,
}

/// Keep track of votes being gossiped about parent chain finality
/// and tally up the weights of the validators on the child subnet,
/// so that we can ask for proposals that are not going to be voted
/// down.
#[derive(Clone)]
pub struct VoteTally {
    /// Current validator weights. These are the ones who will vote on the blocks,
    /// so these are the weights which need to form a quorum.
    power_table: TVar<im::HashMap<ValidatorKey, Weight>>,

    /// The *finalized mainchain* of the parent as observed by this node and it's not
    /// considered finalized by the quorum and will be voted on.
    ///
    /// These are assumed to be final because IIRC that's how the syncer works,
    /// only fetching the info about blocks which are already sufficiently deep.
    ///
    /// When we want to propose, all we have to do is walk back this chain and
    /// tally the votes we collected for the block hashes until we reach a quorum.
    ///
    /// The block hash is optional to allow for null blocks on Filecoin rootnet.
    chain: TVar<im::OrdMap<BlockHeight, Option<TopdownVote>>>,

    /// Index votes received by height and hash, which makes it easy to look up
    /// all the votes for a given block hash and also to verify that a validator
    /// isn't equivocating by trying to vote for two different things at the
    /// same height.
    votes: TVar<im::OrdMap<BlockHeight, im::HashMap<TopdownVote, ValidatorSignatures>>>,

    /// Adding votes can be paused if we observe that looking for a quorum takes too long
    /// and is often retried due to votes being added.
    pause_votes: TVar<bool>,

    /// The latest height that was voted to be finalized and committed to child blockchian
    last_finalized_height: TVar<BlockHeight>,
}

impl VoteTally {
    /// Create an uninitialized instance. Before blocks can be added to it
    /// we will have to set the last finalized block.
    ///
    /// The reason this exists is so that we can delay initialization until
    /// after the genesis block has been executed.
    pub fn empty() -> Self {
        Self {
            power_table: TVar::default(),
            chain: TVar::default(),
            votes: TVar::default(),
            pause_votes: TVar::new(false),
            last_finalized_height: Default::default(),
        }
    }

    /// Initialize the vote tally from the current power table
    /// and the last finalized block from the ledger.
    pub fn new(
        power_table: Vec<(ValidatorKey, Weight)>,
        last_finalized_height: BlockHeight,
    ) -> Self {
        Self {
            power_table: TVar::new(im::HashMap::from_iter(power_table)),
            chain: TVar::default(),
            votes: TVar::default(),
            pause_votes: TVar::new(false),
            last_finalized_height: TVar::new(last_finalized_height),
        }
    }

    pub fn power_table(&self) -> Stm<im::HashMap<ValidatorKey, Weight>> {
        self.power_table.read_clone()
    }

    /// Check that a validator key is currently part of the power table.
    pub fn has_power(&self, validator_key: &ValidatorKey) -> Stm<bool> {
        let pt = self.power_table.read()?;
        // For consistency consider validators without power unknown.
        match pt.get(validator_key) {
            None => Ok(false),
            Some(weight) => Ok(*weight > 0),
        }
    }

    /// Calculate the minimum weight needed for a proposal to pass with the current membership.
    ///
    /// This is inclusive, that is, if the sum of weight is greater or equal to this, it should pass.
    /// The equivalent formula can be found in CometBFT [here](https://github.com/cometbft/cometbft/blob/a8991d63e5aad8be82b90329b55413e3a4933dc0/types/vote_set.go#L307).
    pub fn quorum_threshold(&self) -> Stm<Weight> {
        let total_weight: Weight = self.power_table.read().map(|pt| pt.values().sum())?;

        Ok(total_weight * 2 / 3 + 1)
    }

    /// Return the height of the first entry in the chain.
    ///
    /// This is the block that was finalized *in the ledger*.
    pub fn last_finalized_height(&self) -> Stm<BlockHeight> {
        self.last_finalized_height.read_clone()
    }

    /// Return the height of the last entry in the chain.
    ///
    /// This is the block that we can cast our vote on as final.
    pub fn latest_height(&self) -> Stm<BlockHeight> {
        self.chain
            .read()
            .map(|c| c.get_max().map(|(h, _)| *h).unwrap_or_default())
    }

    /// Get the hash of a block at the given height, if known.
    pub fn payload(&self, height: BlockHeight) -> Stm<Option<TopdownVote>> {
        self.chain.read().map(|c| c.get(&height).cloned().flatten())
    }

    /// Add the next final block observed on the parent blockchain.
    ///
    /// Returns an error unless it's exactly the next expected height,
    /// so the caller has to call this in every epoch. If the parent
    /// chain produced no blocks in that epoch then pass `None` to
    /// represent that null-round in the tally.
    pub fn add_block(&self, payload: TopdownVote) -> StmResult<(), Error> {
        let mut chain = self.chain.read_clone()?;

        let block_height = payload.block_height();

        // Check that we are extending the chain. We could also ignore existing heights.
        match chain.get_max() {
            None => {
                let last_finalized_height = self.last_finalized_height.read_clone()?;
                if block_height <= last_finalized_height {
                    tracing::error!(
                        height = block_height,
                        last_finalized_height,
                        "block data for vote tally went backwards"
                    );
                    return abort(Error::UnexpectedBlock(last_finalized_height, block_height));
                }
            }
            Some((parent_height, vote)) => {
                if block_height == *parent_height {
                    debug_assert!(*vote == Some(payload.clone()), "inconsistent block data");
                }

                if block_height <= *parent_height {
                    tracing::warn!(
                        height = block_height,
                        parent_height,
                        "past block data added, this should not have happened, ignore"
                    );
                    return Ok(());
                }
            }
        };

        chain.insert(block_height, Some(payload));

        self.chain.write(chain)?;

        Ok(())
    }

    /// Add a vote we received.
    ///
    /// Returns `true` if this vote was added, `false` if it was ignored as a
    /// duplicate or a height we already finalized, and an error if it's an
    /// equivocation or from a validator we don't know.
    pub fn add_vote(&self, vote: SignedVote) -> StmResult<bool, Error> {
        if *self.pause_votes.read()? {
            retry()?;
        }

        let (payload, signature, validator_key) = match vote.into_validated_payload() {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("error when validating vote payload: {e}");
                return abort(Error::VoteCannotBeValidated);
            }
        };

        let block_height = payload.block_height();
        if block_height < self.last_finalized_height()? {
            return Ok(false);
        }

        if !self.has_power(&validator_key)? {
            tracing::error!(validator = ?validator_key, "validator unknown or has no power");
            return abort(Error::UnpoweredValidator);
        }

        let mut votes = self.votes.read_clone()?;
        let votes_at_height = votes.entry(block_height).or_default();

        for (bh, vs) in votes_at_height.iter() {
            if *bh != payload && vs.has_voted(&validator_key) {
                tracing::error!(block_height, validator = ?validator_key, "equivocation by validator");
                return abort(Error::Equivocation);
            }
        }

        let votes_for_block = votes_at_height
            .entry(payload)
            .or_insert(ValidatorSignatures::empty());

        if !votes_for_block.add_vote(validator_key, signature) {
            return Ok(false);
        }

        self.votes.write(votes)?;

        Ok(true)
    }

    /// Pause adding more votes until we are finished calling `find_quorum` which
    /// automatically re-enables them.
    pub fn pause_votes_until_find_quorum(&self) -> Stm<()> {
        self.pause_votes.write(true)
    }

    /// Find a block on the (from our perspective) finalized chain that gathered enough votes from validators.
    pub fn find_quorum(&self) -> Stm<Option<(TopdownVote, MultiSigCert)>> {
        self.pause_votes.write(false)?;

        let quorum_threshold = self.quorum_threshold()?;
        let chain = self.chain.read()?;

        let finalized_height = self.last_finalized_height.read_clone()?;

        let votes = self.votes.read()?;
        let power_table = self.power_table.read()?;

        for (block_height, maybe_payload) in chain.iter().rev() {
            if *block_height == finalized_height {
                tracing::debug!(
                    block_height,
                    finalized_height,
                    "finalized height and block height equal, no new proposals"
                );
                break; // This block is already finalized in the ledger, no need to propose it again.
            }
            let Some(payload) = maybe_payload else {
                tracing::debug!(block_height, "null block found in vote proposal");
                continue; // Skip null blocks
            };
            let Some(votes_at_height) = votes.get(block_height) else {
                tracing::debug!(block_height, "no votes");
                continue;
            };
            let Some(votes_for_block) = votes_at_height.get(payload) else {
                tracing::debug!(block_height, "no votes for block");
                continue; // We could detect equovicating voters here.
            };

            let mut weight = 0;

            for vk in votes_for_block.validators() {
                weight += power_table.get(vk).cloned().unwrap_or_default();
                tracing::debug!(weight, key = ?vk, "aggregating vote power");
            }

            tracing::debug!(weight, quorum_threshold, "showdown");

            if weight >= quorum_threshold {
                return Ok(Some((
                    payload.clone(),
                    votes_for_block.to_cert(power_table.borrow()),
                )));
            }
        }

        Ok(None)
    }

    /// Call when a new finalized block is added to the ledger, to clear out all preceding blocks.
    ///
    /// After this operation the minimum item in the chain will the new finalized block.
    pub fn set_finalized(&self, block_height: BlockHeight) -> Stm<()> {
        // Clears all the previous `chain` and `votes`. Because now the proposals are accumulative
        // because of side effects. For example, if votes contains proposals from height 10, the
        // commitment includes the cross messages till height 10. If the vote tally receives a
        // vote for height 11, then the commitment includes cross messages till height 10 and
        // height 11. If height 10 is finalized, the proposal for height 11 should be void because
        // a new proposal contains commitment of only height 11 should be included.
        self.chain.write(OrdMap::new())?;
        self.votes.write(OrdMap::new())?;
        self.last_finalized_height.write(block_height)?;

        Ok(())
    }

    /// Overwrite the power table after it has changed to a new snapshot.
    ///
    /// This method expects absolute values, it completely replaces the existing powers.
    pub fn set_power_table(&self, power_table: Vec<(ValidatorKey, Weight)>) -> Stm<()> {
        let power_table = im::HashMap::from_iter(power_table);
        // We don't actually have to remove the votes of anyone who is no longer a validator,
        // we just have to make sure to handle the case when they are not in the power table.
        self.power_table.write(power_table)
    }

    /// Update the power table after it has changed with changes.
    ///
    /// This method expects only the updated values, leaving everyone who isn't in it untouched
    pub fn update_power_table(&self, power_updates: Vec<(ValidatorKey, Weight)>) -> Stm<()> {
        if power_updates.is_empty() {
            return Ok(());
        }
        // We don't actually have to remove the votes of anyone who is no longer a validator,
        // we just have to make sure to handle the case when they are not in the power table.
        self.power_table.update_mut(|pt| {
            for (vk, w) in power_updates {
                if w == 0 {
                    pt.remove(&vk);
                } else {
                    *pt.entry(vk).or_default() = w;
                }
            }
        })
    }
}

/// Poll the vote tally for new finalized blocks and publish a vote about them if the validator is part of the power table.
pub async fn publish_vote_loop<V>(
    vote_tally: VoteTally,
    // Throttle votes to maximum 1/interval
    vote_interval: Duration,
    // Publish a vote after a timeout even if it's the same as before.
    vote_timeout: Duration,
    key: libp2p::identity::secp256k1::Keypair,
    pubsub_topic: String,
    client: ipc_ipld_resolver::Client<SignedVote>,
) {
    let pubkey = libp2p::identity::PublicKey::from(key.public().clone());
    let validator_key = ValidatorKey::from(pubkey);

    let mut vote_interval = tokio::time::interval(vote_interval);
    vote_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut prev = None;

    loop {
        let prev_height = prev
            .as_ref()
            .map(|(height, _, _)| *height)
            .unwrap_or_default();

        let result = tokio::time::timeout(
            vote_timeout,
            atomically_or_err(|| {
                let next_height = vote_tally.latest_height()?;

                if next_height == prev_height {
                    retry()?;
                }

                let vote = match vote_tally.payload(next_height)? {
                    Some(vote) => vote,
                    None => retry()?,
                };

                let has_power = vote_tally.has_power(&validator_key)?;

                if has_power {
                    // Add our own vote to the tally directly rather than expecting a message from the gossip channel.
                    // TODO (ENG-622): I'm not sure gossip messages published by this node would be delivered to it, so this might be the only way.
                    // NOTE: We should not see any other error from this as we just checked that the validator had power,
                    //       but for piece of mind let's return and log any potential errors, rather than ignore them.

                    let vote = match SignedVote::signed(&key, &vote) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("cannot sign topdown vote payload: {e}");
                            return abort(Error::CannotSignVote);
                        }
                    };
                    vote_tally.add_vote(vote)?;
                }

                Ok((next_height, vote, has_power))
            }),
        )
        .await;

        let (next_height, next_vote, has_power) = match result {
            Ok(Ok(vs)) => vs,
            Err(_) => {
                if let Some(ref vs) = prev {
                    tracing::debug!("vote timeout; re-publishing previous vote");
                    vs.clone()
                } else {
                    tracing::debug!("vote timeout, but no previous vote to re-publish");
                    continue;
                }
            }
            Ok(Err(e)) => {
                tracing::error!(
                    error = e.to_string(),
                    "failed to get next height to vote on"
                );
                continue;
            }
        };

        if has_power && prev_height > 0 {
            tracing::debug!(block_height = next_height, "publishing finality vote");

            match SignedVote::signed(&key, &next_vote) {
                Ok(signed) => {
                    if let Err(e) = client.publish_vote(pubsub_topic.clone(), signed) {
                        tracing::error!(error = e.to_string(), "failed to publish vote");
                    }
                }
                Err(e) => {
                    tracing::error!(error = e.to_string(), "failed to sign vote");
                }
            }

            // Throttle vote gossiping at periods of fast syncing. For example if we create a subnet contract on Friday
            // and bring up a local testnet on Monday, all nodes would be ~7000 blocks behind a Lotus parent. CometBFT
            // would be in-sync, and they could rapidly try to gossip votes on previous heights. GossipSub might not like
            // that, and we can just cast our votes every now and then to finalize multiple blocks.
            vote_interval.tick().await;
        }

        prev = Some((next_height, next_vote, has_power));
    }
}

#[cfg(test)]
mod tests {
    use crate::voting::payload::{SignedVote, TopdownVote};
    use crate::voting::VoteTally;
    use crate::BlockHeight;
    use async_stm::atomically_or_err;
    use ipc_ipld_resolver::ValidatorKey;
    use libp2p::identity::Keypair;

    fn convert_key(key: &Keypair) -> libp2p::identity::secp256k1::Keypair {
        let key = key.clone();
        key.try_into_secp256k1().unwrap()
    }

    fn random_validator_key() -> (Keypair, ValidatorKey) {
        let key_pair = Keypair::generate_secp256k1();
        let public_key = key_pair.public();
        (key_pair, ValidatorKey::from(public_key))
    }

    fn random_vote(height: BlockHeight) -> TopdownVote {
        let rand_bytes = |u: usize| {
            let mut v = vec![];
            for _ in 0..u {
                v.push(rand::random::<u8>());
            }
            v
        };
        let hash = rand_bytes(32);
        let commitment = rand_bytes(64);

        TopdownVote::v1(height, hash, commitment)
    }

    #[tokio::test]
    async fn simple_3_validators_vote() {
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

            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn new_validators_joined_void_previous_quorum() {
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
