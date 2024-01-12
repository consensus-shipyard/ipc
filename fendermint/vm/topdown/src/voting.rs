// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_stm::{abort, Stm, StmResult, TVar};

use crate::{BlockHash, BlockHeight};

// Usign this type because it's `Hash`, unlike the normal `libsecp256k1::PublicKey`.
use ipc_ipld_resolver::ValidatorKey;

pub type Weight = u64;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to extend chain; expected block height {0}, got {1}")]
    UnexpectedBlock(BlockHeight, BlockHeight),

    #[error("unknown validator: {0:?}")]
    UnknownValidator(ValidatorKey),

    #[error(
        "equivocation by validator {0:?} at height {1}; {} != {}",
        hex::encode(.2),
        hex::encode(.3)
    )]
    Equivocation(ValidatorKey, BlockHeight, BlockHash, BlockHash),
}

/// Keep track of votes beging gossiped about parent chain finality
/// and tally up the weights of the validators on the child subnet,
/// so that we can ask for proposals that are not going to be voted
/// down.
pub struct VoteTally {
    /// Current validator weights. These are the ones who will vote on the blocks,
    /// so these are the weights which need to form a quorum.
    power_table: TVar<im::HashMap<ValidatorKey, Weight>>,

    /// The *finalized mainchain* of the parent as observed by this node.
    ///
    /// These are assumed to be final because IIRC that's how the syncer works,
    /// only fetching the info about blocks which are already sufficiently deep.
    ///
    /// When we want to propose, all we have to do is walk back this chain and
    /// tally the votes we collected for the block hashes until we reach a quorum.
    ///
    /// The block hash is optional to allow for null blocks on Filecoin rootnet.
    chain: TVar<im::OrdMap<BlockHeight, Option<BlockHash>>>,

    /// Index votes received by height and hash, which makes it easy to look up
    /// all the votes for a given block hash and also to verify that a validator
    /// isn't equivocating by trying to vote for two different things at the
    /// same height.
    votes: TVar<im::OrdMap<BlockHeight, im::HashMap<BlockHash, im::HashSet<ValidatorKey>>>>,
}

impl VoteTally {
    /// Initialize the vote tally from the current power table
    /// and the last finalized block from the ledger.
    pub fn new(
        power_table: Vec<(ValidatorKey, Weight)>,
        last_finalized_block: (BlockHeight, BlockHash),
    ) -> Self {
        Self {
            power_table: TVar::new(im::HashMap::from_iter(power_table.into_iter())),
            chain: TVar::new(im::OrdMap::from_iter([last_finalized_block])),
            votes: TVar::default(),
        }
    }

    /// Check that a validator key is currently part of the power table.
    pub fn known_validator(&self, validator_key: &ValidatorKey) -> Stm<bool> {
        self.power_table
            .read()
            .map(|pt| pt.contains_key(validator_key))
    }

    /// Calculate the minimum weight needed for a proposal to pass with the current membership.
    pub fn quorum_threshold(&self) -> Stm<Weight> {
        let total_weight: Weight = self.power_table.read().map(|pt| pt.values().sum())?;

        Ok(total_weight * 2 / 3)
    }

    /// Return the height of the first entry in the chain.
    pub fn last_finalized_height(&self) -> Stm<BlockHeight> {
        self.chain
            .read()
            .map(|c| c.get_min().map(|(h, _)| *h).unwrap_or_default())
    }

    /// Add the next final block observed on the parent blockchain.
    ///
    /// Returns an error unless it's exactly the next expected height.
    pub fn add_block(
        &self,
        block_height: BlockHeight,
        block_hash: Option<BlockHash>,
    ) -> StmResult<(), Error> {
        let mut chain = self.chain.read_clone()?;

        // Check that we are extending the chain. We could also ignore existing heights.
        if let Some((prev, _)) = chain.get_max() {
            if block_height != prev + 1 {
                return abort(Error::UnexpectedBlock(prev + 1, block_height));
            }
        }

        chain.insert(block_height, block_hash);

        self.chain.write(chain)?;

        Ok(())
    }

    /// Add a vote we received.
    ///
    /// Returns `true` if this vote was added, `false` if it was ignored as a
    /// duplicate or a height we already finalized, and an error if it's an
    /// equivocation or from a validator we don't know.
    pub fn add_vote(
        &self,
        validator_key: ValidatorKey,
        block_height: BlockHeight,
        block_hash: BlockHash,
    ) -> StmResult<bool, Error> {
        let min_height = self.last_finalized_height()?;

        if block_height < min_height {
            return Ok(false);
        }

        if !self.known_validator(&validator_key)? {
            return abort(Error::UnknownValidator(validator_key));
        }

        let mut votes = self.votes.read_clone()?;
        let votes_at_height = votes.entry(block_height).or_default();

        for (bh, vs) in votes_at_height.iter() {
            if *bh != block_hash && vs.contains(&validator_key) {
                return abort(Error::Equivocation(
                    validator_key,
                    block_height,
                    block_hash,
                    bh.clone(),
                ));
            }
        }

        let votes_for_block = votes_at_height.entry(block_hash).or_default();

        if votes_for_block.insert(validator_key).is_some() {
            return Ok(false);
        }

        self.votes.write(votes)?;

        Ok(true)
    }

    /// Find a block on the (from our perspective) finalized chain that gathered enough votes from validators.
    pub fn find_quorum(&self) -> Stm<Option<(BlockHeight, BlockHash)>> {
        let quorum_threshold = self.quorum_threshold()?;
        let chain = self.chain.read()?;

        let Some((finalized_height, _)) = chain.get_min() else {
            return Ok(None);
        };

        let votes = self.votes.read()?;
        let power_table = self.power_table.read()?;

        let mut weight = 0;
        let mut voters = im::HashSet::new();

        for (block_height, block_hash) in chain.iter().rev() {
            if block_height == finalized_height {
                break;
            }
            // Skipping null blocks
            if let Some(block_hash) = block_hash {
                if let Some(votes_at_height) = votes.get(block_height) {
                    if let Some(votes_for_block) = votes_at_height.get(block_hash) {
                        for vk in votes_for_block {
                            if voters.insert(vk.clone()).is_none() {
                                // New voter, get their current weight; it might be 0 if they have been removed.
                                weight += power_table.get(vk).cloned().unwrap_or_default();
                            }
                        }
                    }

                    if weight > quorum_threshold {
                        return Ok(Some((*block_height, block_hash.clone())));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Call when a new finalized block is added to the ledger, to clear out all preceding blocks.
    ///
    /// After this operation the minimum item in the chain will the new finalized block.
    pub fn set_finalized(&self, block_height: BlockHeight, block_hash: BlockHash) -> Stm<()> {
        self.chain.update(|chain| {
            let (_, mut chain) = chain.split(&block_height);
            chain.insert(block_height, Some(block_hash));
            chain
        })?;

        self.votes.update(|votes| votes.split(&block_height).1)?;

        Ok(())
    }

    /// Update the power table after it has changed to a new snapshot, removing the votes of anyone
    /// who is no longer a validator.
    pub fn set_power_table(&self, power_table: Vec<(ValidatorKey, Weight)>) -> Stm<()> {
        let power_table = im::HashMap::from_iter(power_table.into_iter());
        // We don't actually have to remove the votes of anyone who is no longer a validator,
        // we just have to make sure to handle the case when they are not in the power table.
        self.power_table.write(power_table)
    }
}
