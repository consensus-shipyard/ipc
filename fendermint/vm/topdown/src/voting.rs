// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_stm::{abort, retry, Stm, StmResult, TVar};
use std::fmt::Debug;
use std::hash::Hash;

use crate::{BlockHash, BlockHeight};

// Usign this type because it's `Hash`, unlike the normal `libsecp256k1::PublicKey`.
use ipc_ipld_resolver::ValidatorKey;

pub type Weight = u64;

#[derive(Debug, thiserror::Error)]
pub enum Error<K = ValidatorKey, V: AsRef<[u8]> = BlockHash> {
    #[error("the last finalized block has not been set")]
    Uninitialized,

    #[error("failed to extend chain; expected block height {0}, got {1}")]
    UnexpectedBlock(BlockHeight, BlockHeight),

    #[error("unknown validator: {0:?}")]
    UnknownValidator(K),

    #[error(
        "equivocation by validator {0:?} at height {1}; {} != {}",
        hex::encode(.2),
        hex::encode(.3)
    )]
    Equivocation(K, BlockHeight, V, V),
}

/// Keep track of votes being gossiped about parent chain finality
/// and tally up the weights of the validators on the child subnet,
/// so that we can ask for proposals that are not going to be voted
/// down.
#[derive(Clone)]
pub struct VoteTally<K = ValidatorKey, V = BlockHash> {
    /// Current validator weights. These are the ones who will vote on the blocks,
    /// so these are the weights which need to form a quorum.
    power_table: TVar<im::HashMap<K, Weight>>,

    /// The *finalized mainchain* of the parent as observed by this node.
    ///
    /// These are assumed to be final because IIRC that's how the syncer works,
    /// only fetching the info about blocks which are already sufficiently deep.
    ///
    /// When we want to propose, all we have to do is walk back this chain and
    /// tally the votes we collected for the block hashes until we reach a quorum.
    ///
    /// The block hash is optional to allow for null blocks on Filecoin rootnet.
    chain: TVar<im::OrdMap<BlockHeight, Option<V>>>,

    /// Index votes received by height and hash, which makes it easy to look up
    /// all the votes for a given block hash and also to verify that a validator
    /// isn't equivocating by trying to vote for two different things at the
    /// same height.
    votes: TVar<im::OrdMap<BlockHeight, im::HashMap<V, im::HashSet<K>>>>,

    /// Adding votes can be paused if we observe that looking for a quorum takes too long
    /// and is often retried due to votes being added.
    pause_votes: TVar<bool>,
}

impl<K, V> VoteTally<K, V>
where
    K: Clone + Hash + Eq + Sync + Send + 'static,
    V: AsRef<[u8]> + Clone + Hash + Eq + Sync + Send + 'static,
{
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
        }
    }

    /// Initialize the vote tally from the current power table
    /// and the last finalized block from the ledger.
    pub fn new(power_table: Vec<(K, Weight)>, last_finalized_block: (BlockHeight, V)) -> Self {
        let (height, hash) = last_finalized_block;
        Self {
            power_table: TVar::new(im::HashMap::from_iter(power_table)),
            chain: TVar::new(im::OrdMap::from_iter([(height, Some(hash))])),
            votes: TVar::default(),
            pause_votes: TVar::new(false),
        }
    }

    /// Check that a validator key is currently part of the power table.
    pub fn known_validator(&self, validator_key: &K) -> Stm<bool> {
        let pt = self.power_table.read()?;
        // For consistency consider validators without power unknown.
        match pt.get(validator_key) {
            None => Ok(false),
            Some(weight) => Ok(*weight > 0),
        }
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
        block_hash: Option<V>,
    ) -> StmResult<(), Error<K>> {
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
        validator_key: K,
        block_height: BlockHeight,
        block_hash: V,
    ) -> StmResult<bool, Error<K, V>> {
        if *self.pause_votes.read()? {
            retry()?;
        }

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

    /// Pause adding more votes until we are finished calling `find_quorum` which
    /// automatically re-enables them.
    pub fn pause_votes_until_find_quorum(&self) -> Stm<()> {
        self.pause_votes.write(true)
    }

    /// Find a block on the (from our perspective) finalized chain that gathered enough votes from validators.
    pub fn find_quorum(&self) -> Stm<Option<(BlockHeight, V)>> {
        self.pause_votes.write(false)?;

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
            let Some(block_hash) = block_hash else {
                continue; // Skip null blocks
            };
            let Some(votes_at_height) = votes.get(block_height) else {
                continue;
            };
            let Some(votes_for_block) = votes_at_height.get(block_hash) else {
                continue; // We could detect equovicating voters here.
            };

            for vk in votes_for_block {
                if voters.insert(vk.clone()).is_none() {
                    // New voter, get their current weight; it might be 0 if they have been removed.
                    weight += power_table.get(vk).cloned().unwrap_or_default();
                }
            }

            if weight > quorum_threshold {
                return Ok(Some((*block_height, block_hash.clone())));
            }
        }

        Ok(None)
    }

    /// Call when a new finalized block is added to the ledger, to clear out all preceding blocks.
    ///
    /// After this operation the minimum item in the chain will the new finalized block.
    pub fn set_finalized(&self, block_height: BlockHeight, block_hash: V) -> Stm<()> {
        self.chain.update(|chain| {
            let (_, mut chain) = chain.split(&block_height);
            chain.insert(block_height, Some(block_hash));
            chain
        })?;

        self.votes.update(|votes| votes.split(&block_height).1)?;

        Ok(())
    }

    /// Overwrite the power table after it has changed to a new snapshot.
    ///
    /// This method expects absolute values, it completely replaces the existing powers.
    pub fn set_power_table(&self, power_table: Vec<(K, Weight)>) -> Stm<()> {
        let power_table = im::HashMap::from_iter(power_table);
        // We don't actually have to remove the votes of anyone who is no longer a validator,
        // we just have to make sure to handle the case when they are not in the power table.
        self.power_table.write(power_table)
    }

    /// Update the power table after it has changed with changes.
    ///
    /// This method expects only the updated values, leaving everyone who isn't in it untouched
    pub fn update_power_table(&self, power_updates: Vec<(K, Weight)>) -> Stm<()> {
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
