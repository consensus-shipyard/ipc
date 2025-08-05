// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! IPC related execution

use crate::app::{AppStoreKey, SubnetAppState};
use crate::{App, BlockHeight};
use ethers::abi::AbiEncode;
use fendermint_storage::{Codec, Encode, KVReadable, KVStore, KVWritable};
use fendermint_vm_genesis::{Power, Validator};
use fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller;
use fendermint_vm_interpreter::fvm::state::{FvmExecState, FvmStateParams};
use fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore;
use fendermint_vm_interpreter::MessagesInterpreter;
use fendermint_vm_topdown::sync::ParentFinalityStateQuery;
use fendermint_vm_topdown::IPCParentFinality;
use fvm_ipld_blockstore::Blockstore;
use ipc_actors_abis::subnet_actor_checkpoint_facet::{Commitment, StateCommitmentBreakDown};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub fn derive_subnet_app_hash(state: &SubnetAppState) -> tendermint::hash::AppHash {
    let state_params_cid = fendermint_vm_message::cid(state.state_params())
        .expect("state params have a CID")
        .to_bytes();

    let mut submission = StateCommitmentBreakDown {
        state_root: state_params_cid.into(),
        msg_batch_commitment: Commitment::default(),
        validator_next_configuration_number: 0,
        activity_commitment: [0; 32],
    };

    if let Some(commitment) = state.light_client_commitments() {
        submission.activity_commitment = commitment.activity_commitment;
        submission.msg_batch_commitment = Commitment {
            total_num_msgs: commitment.msg_batch_commitment.total_num_msgs,
            msgs_root: commitment.msg_batch_commitment.msgs_root,
        };
        submission.validator_next_configuration_number =
            commitment.validator_next_configuration_number;
    }

    let app_hash_bytes = ethers::utils::keccak256(submission.encode());
    tendermint::hash::AppHash::try_from(app_hash_bytes.to_vec()).expect("hash can be wrapped")
}

/// All the things that can be voted on in a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppVote {
    /// The validator considers a certain block final on the parent chain.
    ParentFinality(IPCParentFinality),
}

/// Queries the LATEST COMMITTED parent finality from the storage
pub struct AppParentFinalityQuery<DB, SS, S, I>
where
    SS: Blockstore + Clone + 'static + Send + Sync,
    S: KVStore,
    I: MessagesInterpreter<SS> + Send + Sync,
{
    /// The app to get state
    app: App<DB, SS, S, I>,
    gateway_caller: GatewayCaller<ReadOnlyBlockstore<Arc<SS>>>,
}

impl<DB, SS, S, I> AppParentFinalityQuery<DB, SS, S, I>
where
    S: KVStore
        + Codec<SubnetAppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + Clone + 'static + Send + Sync,
    I: MessagesInterpreter<SS> + Send + Sync,
{
    pub fn new(app: App<DB, SS, S, I>) -> Self {
        Self {
            app,
            gateway_caller: GatewayCaller::default(),
        }
    }

    fn with_exec_state<F, T>(&self, f: F) -> anyhow::Result<Option<T>>
    where
        F: FnOnce(FvmExecState<ReadOnlyBlockstore<Arc<SS>>>) -> anyhow::Result<T>,
    {
        match self.app.read_only_view(None)? {
            Some(s) => f(s).map(Some),
            None => Ok(None),
        }
    }
}

impl<DB, SS, S, I> ParentFinalityStateQuery for AppParentFinalityQuery<DB, SS, S, I>
where
    S: KVStore
        + Codec<SubnetAppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + Clone + 'static + Send + Sync,
    I: MessagesInterpreter<SS> + Send + Sync,
{
    fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>> {
        self.with_exec_state(|mut exec_state| {
            self.gateway_caller
                .get_latest_parent_finality(&mut exec_state)
        })
    }

    fn get_power_table(&self) -> anyhow::Result<Option<Vec<Validator<Power>>>> {
        self.with_exec_state(|mut exec_state| {
            self.gateway_caller
                .current_power_table(&mut exec_state)
                .map(|(_, pt)| pt)
        })
    }
}
