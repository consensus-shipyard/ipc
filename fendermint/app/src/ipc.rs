// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! IPC related execution

use crate::app::{AppStoreKey, SubnetAppState};
use crate::{App, BlockHeight};
use ethers::utils::keccak256;
use fendermint_storage::{Codec, Encode, KVReadable, KVStore, KVWritable};
use fendermint_vm_genesis::{Power, Validator};
use fendermint_vm_interpreter::fvm::end_block_hook::LightClientCommitments;
use fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller;
use fendermint_vm_interpreter::fvm::state::{FvmExecState, FvmStateParams};
use fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore;
use fendermint_vm_interpreter::MessagesInterpreter;
use fendermint_vm_topdown::sync::ParentFinalityStateQuery;
use fendermint_vm_topdown::{IPCBlobFinality, IPCParentFinality, IPCReadRequestClosed};
use fvm_ipld_blockstore::Blockstore;
use ipc_actors_abis::subnet_actor_checkpointing_facet::{
    AppHashBreakdown, Commitment, CompressedActivityRollup,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn derive_subnet_app_hash_from_components(
    state: &FvmStateParams,
    maybe_light: Option<&LightClientCommitments>,
) -> tendermint::hash::AppHash {
    let state_params_cid = state.state_root.to_bytes();
    let mut submission = AppHashBreakdown {
        state_root: state_params_cid.into(),
        msg_batch_commitment: Commitment::default(),
        validator_next_configuration_number: 0,
        activity_commitment: CompressedActivityRollup::default(),
    };

    if let Some(commitment) = maybe_light {
        // safe to unwrap as it's internal conversion
        submission.activity_commitment = commitment.activity_commitment.clone().try_into().unwrap();
        submission.msg_batch_commitment = Commitment {
            total_num_msgs: commitment.msg_batch_commitment.total_num_msgs,
            msgs_root: commitment.msg_batch_commitment.msgs_root,
        };
        submission.validator_next_configuration_number =
            commitment.validator_next_configuration_number;
    }

    let app_hash_bytes = abi_encode_tuple_manual_hash(&submission);
    tendermint::hash::AppHash::try_from(app_hash_bytes.to_vec()).expect("hash can be wrapped")
}

pub fn derive_subnet_app_hash(state: &SubnetAppState) -> tendermint::hash::AppHash {
    derive_subnet_app_hash_from_components(state.state_params(), state.light_client_commitments())
}

/// All the things that can be voted on in a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppVote {
    /// The validator considers a certain block final on the parent chain.
    ParentFinality(IPCParentFinality),
    /// The validator considers a certain blob final.
    BlobFinality(IPCBlobFinality),
    /// The validator considers a certain read request completed.
    ReadRequestClosed(IPCReadRequestClosed),
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

fn abi_encode_tuple_manual(b: &AppHashBreakdown) -> Vec<u8> {
    use ethers::abi::{encode, Token};
    use ethers::types::U256;
    let commitment = Token::Tuple(vec![
        Token::Uint(U256::from(b.msg_batch_commitment.total_num_msgs)), // uint64 (ABI word)
        Token::FixedBytes(b.msg_batch_commitment.msgs_root.to_vec()),   // bytes32
    ]);

    let stats = Token::Tuple(vec![
        Token::Uint(U256::from(
            b.activity_commitment
                .consensus
                .stats
                .total_active_validators,
        )), // uint64
        Token::Uint(U256::from(
            b.activity_commitment
                .consensus
                .stats
                .total_num_blocks_committed,
        )), // uint64
    ]);

    let consensus = Token::Tuple(vec![
        stats, // (uint64,uint64)
        Token::FixedBytes(
            b.activity_commitment
                .consensus
                .data_root_commitment
                .to_vec(),
        ), // bytes32
    ]);

    let top = Token::Tuple(vec![
        Token::Bytes(b.state_root.clone().to_vec()), // bytes (dynamic)
        commitment,                                  // (uint64,bytes32)
        Token::Uint(U256::from(b.validator_next_configuration_number)), // uint64
        consensus,                                   // ((uint64,uint64),bytes32)
    ]);

    encode(&[top])
}

fn abi_encode_tuple_manual_hash(b: &AppHashBreakdown) -> [u8; 32] {
    keccak256(abi_encode_tuple_manual(b))
}

#[cfg(test)]
mod tests {
    use crate::ipc::abi_encode_tuple_manual_hash;
    use ethers::types::Bytes;
    use ipc_actors_abis::subnet_actor_checkpointing_facet::{
        AggregatedStats, AppHashBreakdown, Commitment, CompressedActivityRollup, CompressedSummary,
    };

    #[test]
    fn test_app_hash() {
        let breakdown = AppHashBreakdown {
            state_root: Bytes::from(
                hex::decode(
                    "0171a0e40220a776144e88bd56b306ffda96b79dabfdbec8f019603e7bb649667279f5600882",
                )
                .unwrap(),
            ),
            msg_batch_commitment: Commitment {
                total_num_msgs: 0,
                msgs_root: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            },
            validator_next_configuration_number: 0,
            activity_commitment: CompressedActivityRollup {
                consensus: CompressedSummary {
                    stats: AggregatedStats {
                        total_active_validators: 1,
                        total_num_blocks_committed: 10,
                    },
                    data_root_commitment: [
                        196, 56, 51, 81, 126, 189, 173, 223, 65, 33, 72, 220, 198, 251, 248, 103,
                        68, 100, 186, 130, 111, 244, 207, 168, 7, 7, 150, 100, 68, 197, 242, 53,
                    ],
                },
            },
        };

        let bytes = abi_encode_tuple_manual_hash(&breakdown);
        assert_eq!(
            hex::encode(bytes),
            "5904603f8b5b1844a80498361e2a2c92938529bd09be6ba7eefcd9b679d5a0e3"
        );

        let another = AppHashBreakdown {
            state_root: Bytes::from(
                hex::decode(
                    "0171a0e40220d324f2815da59e84482e6103a5eb1b6a7674918c55ebad378685ec908fac2e39",
                )
                .unwrap(),
            ),
            msg_batch_commitment: Commitment {
                total_num_msgs: 0,
                msgs_root: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            },
            validator_next_configuration_number: 0,
            activity_commitment: CompressedActivityRollup {
                consensus: CompressedSummary {
                    stats: AggregatedStats {
                        total_active_validators: 1,
                        total_num_blocks_committed: 10,
                    },
                    data_root_commitment: [
                        196, 56, 51, 81, 126, 189, 173, 223, 65, 33, 72, 220, 198, 251, 248, 103,
                        68, 100, 186, 130, 111, 244, 207, 168, 7, 7, 150, 100, 68, 197, 242, 53,
                    ],
                },
            },
        };

        let bytes = abi_encode_tuple_manual_hash(&another);
        assert_eq!(
            hex::encode(bytes),
            "8d26ca04a9eb3b9140457445abd7ab774c98b6b6a1a4ee187fb8dc8ee99f9f55"
        );
    }
}
