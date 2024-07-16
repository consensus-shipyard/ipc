// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod payload;
pub mod quorum;

use crate::voting::quorum::{MultiSigCert, ValidatorSignatures};
use async_stm::{abort, atomically_or_err, retry, Stm, StmResult, TVar};
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

    #[error("failed to extend chain; expected block height {0}, got {1}")]
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
    pub fn add_block(
        &self,
        block_height: BlockHeight,
        payload: Option<TopdownVote>,
    ) -> StmResult<(), Error> {
        let mut chain = self.chain.read_clone()?;

        // Check that we are extending the chain. We could also ignore existing heights.
        match chain.get_max() {
            None => {
                return abort(Error::Uninitialized);
            }
            Some((parent_height, _)) => {
                if block_height != parent_height + 1 {
                    return abort(Error::UnexpectedBlock(parent_height + 1, block_height));
                }
            }
        }

        chain.insert(block_height, payload);

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

        if votes_for_block.add_vote(validator_key, signature) {
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

        let Some((finalized_height, _)) = chain.get_min() else {
            tracing::debug!("finalized height not found");
            return Ok(None);
        };

        let votes = self.votes.read()?;
        let power_table = self.power_table.read()?;

        let mut weight = 0;

        for (block_height, maybe_payload) in chain.iter().rev() {
            if block_height == finalized_height {
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
        self.chain.update(|chain| {
            let (_, chain) = chain.split(&block_height);
            chain
        })?;

        self.votes.update(|votes| votes.split(&block_height).1)?;

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
    key: libp2p::identity::Keypair,
    pubsub_topic: String,
    client: ipc_ipld_resolver::Client<SignedVote>,
) {
    let validator_key = ValidatorKey::from(key.public());

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
