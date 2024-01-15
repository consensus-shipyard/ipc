// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! State Machine Test for the finality voting tally component.
//!
//! The test simulates random events that the tally can receive, such as votes received
//! over gossip, power table updates, block being executed, and tests that the tally
//! correctly identifies the blocks which are agreeable to the majority of validator.
//!
//! It can be executed the following way:
//!
//! ```text
//! cargo test --release -p fendermint_vm_topdown --test smt_voting
//! ```

use std::{
    cmp::{max, min},
    collections::BTreeMap,
};

use arbitrary::Unstructured;
use async_stm::{atomically_or_err, StmError, StmResult};
use fendermint_testing::{smt, state_machine_test};
use fendermint_vm_topdown::{
    voting::{self, VoteTally, Weight},
    BlockHash, BlockHeight,
};
use ipc_ipld_resolver::ValidatorKey;
use rand::{rngs::StdRng, SeedableRng};

/// Size of window of voting relative to the last cast vote.
const MAX_VOTE_DELTA: BlockHeight = 10;
/// Maximum number of blocks to finalize at a time.
const MAX_FINALIZED_DELTA: BlockHeight = 5;

state_machine_test!(voting, 10000 ms, 65512 bytes, 200 steps, VotingMachine::new());

#[derive(Debug)]
pub enum VotingCommand {
    /// The tally observes the next block fo the chain.
    ExtendChain(BlockHeight, BlockHash),
    /// One of the validators voted on a block.
    AddVote(ValidatorKey, BlockHeight, BlockHash),
    /// Update the power table.
    UpdatePower(Vec<(ValidatorKey, Weight)>),
    /// A certain height was finalized in the ledger.
    BlockFinalized(BlockHeight, BlockHash),
    /// Ask the tally for the highest agreeable block.
    FindQuorum,
}

/// Model state of voting
#[derive(Clone)]
pub struct VotingState {
    /// We have a single parent chain that everybody observes, just at different heights.
    /// There is no forking in this test because we assume that the syncing component
    /// only downloads blocks which are final, and that reorgs don't happen.
    ///
    /// The tally is currently unable to handle reorgs and rejects equivocations anyway.
    ///
    /// TODO: Null blocks.
    chain: Vec<BlockHash>,
    /// All the validator keys to help pic random ones.
    validator_keys: Vec<ValidatorKey>,
    /// All the validators with varying weights (can be zero).
    validator_states: BTreeMap<ValidatorKey, ValidatorState>,

    last_finalized_block: BlockHeight,
    last_chain_block: BlockHeight,
}

impl VotingState {
    pub fn can_extend(&self) -> bool {
        self.last_chain_block < self.max_chain_height()
    }

    pub fn can_finalize(&self) -> bool {
        // We can finalize a block even if we haven't observed the votes,
        // if the majority of validators vote for an actual block that
        // proposed it for execution.
        self.last_finalized_block < self.max_chain_height()
    }

    pub fn next_chain_block(&self) -> Option<(BlockHeight, BlockHash)> {
        if self.can_extend() {
            let h = self.last_chain_block + 1;
            Some((h, self.block_hash(h)))
        } else {
            None
        }
    }

    pub fn max_chain_height(&self) -> BlockHeight {
        self.chain.len() as BlockHeight - 1
    }

    pub fn block_hash(&self, h: BlockHeight) -> BlockHash {
        self.chain[h as usize].clone()
    }
}

#[derive(Clone)]
pub struct ValidatorState {
    /// Current voting power (can be zero).
    weight: Weight,
    /// The highest vote casted by the validator.
    /// Initially zero, meaning everyone voted on the initial finalized block.
    highest_vote: BlockHeight,
}

pub struct VotingMachine {
    /// Runtime for executing async commands.
    runtime: tokio::runtime::Runtime,
}

impl VotingMachine {
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().expect("create tokio runtime"),
        }
    }

    fn run<F, T>(&self, f: F) -> Result<T, voting::Error>
    where
        F: Fn() -> StmResult<T, voting::Error>,
    {
        self.runtime.block_on(atomically_or_err(f))
    }
}

impl smt::StateMachine for VotingMachine {
    /// The System Under Test is the Vote Tally.
    type System = VoteTally;
    /// The model state is defined here in the test.
    type State = VotingState;
    /// Random commands we can apply in a step.
    type Command = VotingCommand;
    /// Result of command application on the system.
    ///
    /// The only return value we are interested in is the finality.
    type Result = Result<Option<(BlockHeight, BlockHash)>, voting::Error>;

    /// New random state.
    fn gen_state(&self, u: &mut Unstructured) -> arbitrary::Result<Self::State> {
        let chain_length = u.int_in_range(40..=60)?;
        let mut chain = Vec::new();
        for _ in 0..chain_length {
            let block_hash = u.bytes(32)?;
            chain.push(Vec::from(block_hash));
        }

        let validator_count = u.int_in_range(1..=5)?;
        let mut rng = StdRng::seed_from_u64(u.arbitrary()?);
        let mut validator_states = BTreeMap::new();

        for i in 0..validator_count {
            let min_weight = if i == 0 { 1u64 } else { 0u64 };
            let weight = u.int_in_range(min_weight..=100)?;

            // A ValidatorKey is has a lot of wrapping...
            let secret_key = fendermint_crypto::SecretKey::random(&mut rng);
            let public_key = secret_key.public_key();
            let public_key = libp2p::identity::secp256k1::PublicKey::try_from_bytes(
                &public_key.serialize_compressed(),
            )
            .expect("secp256k1 public key");
            let public_key = libp2p::identity::PublicKey::from(public_key);
            let validator_key = ValidatorKey::from(public_key);

            validator_states.insert(
                validator_key,
                ValidatorState {
                    weight,
                    highest_vote: 0,
                },
            );
        }

        Ok(VotingState {
            chain,
            validator_keys: validator_states.keys().cloned().collect(),
            validator_states,
            last_chain_block: 0,
            last_finalized_block: 0,
        })
    }

    /// New System Under Test.
    fn new_system(&self, state: &Self::State) -> Self::System {
        let power_table = state
            .validator_states
            .iter()
            .filter(|(_, vs)| vs.weight > 0)
            .map(|(vk, vs)| (vk.clone(), vs.weight))
            .collect();

        let last_finalized_block = (0, state.chain[0].clone());

        VoteTally::new(power_table, last_finalized_block)
    }

    /// New random command.
    fn gen_command(
        &self,
        u: &mut Unstructured,
        state: &Self::State,
    ) -> arbitrary::Result<Self::Command> {
        let cmd = match u.int_in_range(0..=100)? {
            // Add a block to the observed chain
            i if i < 20 && state.can_extend() => {
                let (height, hash) = state.next_chain_block().unwrap();
                VotingCommand::ExtendChain(height, hash)
            }
            // Add a new (or repeated) vote by a validator, extending its chain
            i if i < 60 => {
                let vk = u.choose(&state.validator_keys)?;
                let high_vote = state.validator_states[vk].highest_vote;
                let max_vote: BlockHeight =
                    min(state.max_chain_height(), high_vote + MAX_VOTE_DELTA);
                let min_vote: BlockHeight = max(0, high_vote - MAX_VOTE_DELTA);
                let vote_height = u.int_in_range(min_vote..=max_vote)?;
                let vote_hash = state.block_hash(vote_height);
                VotingCommand::AddVote(vk.clone(), vote_height, vote_hash)
            }
            // Update the power table
            i if i < 80 => {
                // Move power from one validator to another (so we never have everyone be zero).
                let vk1 = u.choose(&state.validator_keys)?;
                let vk2 = u.choose(&state.validator_keys)?;
                let weight = state.validator_states[vk1].weight;
                let delta = u.int_in_range(0..=weight)?;

                let mut validator_states = state.validator_states.clone();
                validator_states.get_mut(vk1).unwrap().weight -= delta;
                validator_states.get_mut(vk2).unwrap().weight += delta;

                let power_table = validator_states
                    .into_iter()
                    .map(|(vk, vs)| (vk, vs.weight))
                    .collect();

                VotingCommand::UpdatePower(power_table)
            }
            // Finalize a block
            i if i < 90 && state.can_finalize() => {
                let min_fin = state.last_finalized_block + 1;
                let max_fin = min(
                    state.max_chain_height(),
                    state.last_finalized_block + MAX_FINALIZED_DELTA,
                );
                let fin_height = u.int_in_range(min_fin..=max_fin)?;
                let fin_hash = state.block_hash(fin_height);
                VotingCommand::BlockFinalized(fin_height, fin_hash)
            }
            _ => VotingCommand::FindQuorum,
        };
        Ok(cmd)
    }

    /// Apply the command on the System Under Test.
    fn run_command(&self, system: &mut Self::System, cmd: &Self::Command) -> Self::Result {
        self.run(|| match cmd {
            VotingCommand::ExtendChain(block_height, block_hash) => system
                .add_block(*block_height, Some(block_hash.clone()))
                .map(|_| None),
            VotingCommand::AddVote(vk, block_height, block_hash) => system
                .add_vote(vk.clone(), *block_height, block_hash.clone())
                .map(|_| None),
            VotingCommand::UpdatePower(power_table) => system
                .set_power_table(power_table.clone())
                .map_err(|c| c.into())
                .map(|_| None),
            VotingCommand::BlockFinalized(block_height, block_hash) => system
                .set_finalized(*block_height, block_hash.clone())
                .map_err(|c| c.into())
                .map(|_| None),
            VotingCommand::FindQuorum => system.find_quorum().map_err(|c| c.into()),
        })
    }

    fn check_result(&self, cmd: &Self::Command, pre_state: &Self::State, result: Self::Result) {
        todo!()
    }

    fn next_state(&self, cmd: &Self::Command, state: Self::State) -> Self::State {
        todo!()
    }

    fn check_system(
        &self,
        cmd: &Self::Command,
        post_state: &Self::State,
        post_system: &Self::System,
    ) -> bool {
        todo!()
    }
}
