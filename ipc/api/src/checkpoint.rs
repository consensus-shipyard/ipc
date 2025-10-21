// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Cross network messages related struct and utility functions.

use crate::cross::IpcEnvelope;
use crate::subnet_id::SubnetID;
use crate::HumanReadable;
use cid::Cid;
use ethers::abi::AbiEncode;
use ethers::utils::hex;
use fvm_ipld_encoding::DAG_CBOR;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_actors_abis::checkpointing_facet;
use lazy_static::lazy_static;
use multihash_codetable::{Code, MultihashDigest};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
use serde_with::serde_as;
use std::fmt::{Display, Formatter};

lazy_static! {
    // Default CID used for the genesis checkpoint. Using
    // TCid::default() leads to corrupting the fvm datastore
    // for storing the cid of an inaccessible HAMT.
    pub static ref CHECKPOINT_GENESIS_CID: Cid =
        Cid::new_v1(DAG_CBOR, Code::Blake2b256.digest("genesis".as_bytes()));
    /// ABI types of the Merkle tree which contains validator addresses and their voting power.
    pub static ref VALIDATOR_REWARD_FIELDS: Vec<String> = vec!["address".to_owned(), "uint64".to_owned()];
}

pub type Signature = Vec<u8>;

/// The event emitted
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct QuorumReachedEvent {
    pub obj_kind: u8,
    pub height: ChainEpoch,
    /// The checkpoint hash
    pub obj_hash: Vec<u8>,
    pub quorum_weight: TokenAmount,
}

impl Display for QuorumReachedEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "QuorumReachedEvent<height: {}, checkpoint: {}, quorum_weight: {}>",
            self.height,
            hex::encode(&self.obj_hash),
            self.quorum_weight
        )
    }
}

/// The collection of items for the bottom up checkpoint submission
#[serde_as]
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct BottomUpCheckpointBundle {
    pub checkpoint: BottomUpCheckpoint,
    /// The list of signatures that have signed the checkpoint hash
    #[serde_as(as = "Vec<HumanReadable>")]
    pub signatures: Vec<Signature>,
    /// The list of addresses that have signed the checkpoint hash
    pub signatories: Vec<Address>,
}

/// The collection of items for the bottom up checkpoint submission
#[serde_as]
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct BottomUpMsgBatch {
    /// Child subnet ID, for replay protection from other subnets where the exact same validators operate
    #[serde_as(as = "HumanReadable")]
    pub subnet_id: SubnetID,
    /// The height of the child subnet at which the batch was cut
    pub block_height: ChainEpoch,
    /// Batch of messages to execute
    pub msgs: Vec<IpcEnvelope>,
}

#[serde_as]
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct BottomUpBatchCommitment {
    pub total_num_msgs: u64,
    pub msgs_root: Vec<u8>,
}

// Encode to human-readable format, expected by `ipc_api::merkle::MerkleGen`.
// TODO: since this is done only to later be encoded via `abi::encode`, while
// that can be done on `checkpointing_facet::IpcEnvelope` directly,
// this workflow doesn't make sense, and should be refactored.
pub fn abi_encode_envelope(msg: &checkpointing_facet::IpcEnvelope) -> Vec<String> {
    vec![
        hex::encode(msg.kind.encode()),
        hex::encode(msg.local_nonce.encode()),
        hex::encode(msg.original_nonce.encode()),
        hex::encode(msg.value.encode()),
        hex::encode(msg.to.clone().subnet_id.root.encode()),
        format!(
            "[{}]",
            msg.to
                .subnet_id
                .route
                .iter()
                .map(|addr| format!("{:#x}", addr))
                .collect::<Vec<_>>()
                .join(",")
        ),
        hex::encode(msg.to.clone().raw_address.addr_type.encode()),
        format!("{:#x}", msg.to.clone().raw_address.payload),
        hex::encode(msg.from.clone().subnet_id.root.encode()),
        format!(
            "[{}]",
            msg.from
                .subnet_id
                .route
                .iter()
                .map(|addr| format!("{:#x}", addr))
                .collect::<Vec<_>>()
                .join(",")
        ),
        hex::encode(msg.from.clone().raw_address.addr_type.encode()),
        format!("{:#x}", msg.from.clone().raw_address.payload),
        format!("{:#x}", msg.message),
    ]
}

pub fn abi_encode_envelope_fields() -> Vec<impl ToString> {
    vec![
        "uint8".to_owned(),
        "uint64".to_owned(),
        "uint64".to_owned(),
        "uint256".to_owned(),
        "uint64".to_owned(),
        "address[]".to_owned(),
        "uint8".to_owned(),
        "bytes".to_owned(),
        "uint64".to_owned(),
        "address[]".to_owned(),
        "uint8".to_owned(),
        "bytes".to_owned(),
        "bytes".to_owned(),
    ]
}

/// Compressed representation of the activity summary that can be embedded in checkpoints to propagate up the hierarchy.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct CompressedActivityRollup {
    pub consensus: consensus::CompressedSummary,
}

/// Namespace for consensus-level activity summaries.
/// XYZ(raulk) move to activity module
pub mod consensus {
    use fvm_shared::address::Address;
    use serde::{Deserialize, Serialize};

    /// Aggregated stats for consensus-level activity.
    #[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
    pub struct AggregatedStats {
        /// The total number of unique validators that have mined within this period.
        pub total_active_validators: u64,
        /// The total number of blocks committed by all validators during this period.
        pub total_num_blocks_committed: u64,
    }

    /// The commitments for the child subnet activities that should be submitted to the parent subnet
    /// together with a bottom up checkpoint
    #[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
    pub struct ValidatorData {
        /// The validator address
        pub validator: Address,
        /// The number of blocks mined
        pub blocks_committed: u64,
    }

    // The full activity summary for consensus-level activity.
    #[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
    pub struct FullSummary {
        pub stats: AggregatedStats,
        /// The breakdown of activity per validator.
        pub data: Vec<ValidatorData>,
    }

    /// The compresed representation of the activity summary for consensus-level activity suitable for embedding in a checkpoint.
    #[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
    pub struct CompressedSummary {
        pub stats: AggregatedStats,
        /// The commitment for the validator details, so that we don't have to transmit them in full.
        pub data_root_commitment: Vec<u8>,
    }
}

#[serde_as]
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct BottomUpCheckpoint {
    /// Child subnet ID, for replay protection from other subnets where the exact same validators operate.
    /// Alternatively it can be appended to the hash before signing, similar to how we use the chain ID.
    pub subnet_id: SubnetID,
    /// The height of the child subnet at which this checkpoint was cut.
    /// Has to follow the previous checkpoint by checkpoint period.
    pub block_height: ChainEpoch,
    /// The hash of the block.
    #[serde_as(as = "HumanReadable")]
    pub block_hash: Vec<u8>,
    /// The number of the membership (validator set) which is going to sign the next checkpoint.
    /// This one expected to be signed by the validators from the membership reported in the previous checkpoint.
    /// 0 could mean "no change".
    pub next_configuration_number: u64,
    /// The list of messages for execution
    pub msgs: BottomUpBatchCommitment,
    /// The activity commitment from child subnet to parent subnet
    pub activity_rollup: CompressedActivityRollup,
}

pub fn serialize_vec_bytes_to_vec_hex<T: AsRef<[u8]>, S>(
    data: &[T],
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(data.len()))?;
    for element in data {
        seq.serialize_element(&hex::encode(element))?;
    }
    seq.end()
}

#[cfg(test)]
mod tests {
    use crate::address::IPCAddress;
    use crate::checkpoint::Signature;
    use crate::subnet_id::SubnetID;
    use crate::HumanReadable;
    use fvm_shared::address::Address;
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;
    use std::str::FromStr;

    #[test]
    fn test_serialization_vec_vec_u8() {
        #[serde_as]
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct T {
            #[serde_as(as = "Vec<HumanReadable>")]
            d: Vec<Signature>,
            #[serde_as(as = "HumanReadable")]
            subnet_id: SubnetID,
            #[serde_as(as = "HumanReadable")]
            ipc_address: IPCAddress,
        }

        let subnet_id =
            SubnetID::from_str("/r31415926/f2xwzbdu7z5sam6hc57xxwkctciuaz7oe5omipwbq").unwrap();
        let ipc_address = IPCAddress::new(&subnet_id, &Address::new_id(101)).unwrap();

        let t = T {
            d: vec![vec![1; 30], vec![2; 30]],
            subnet_id,
            ipc_address,
        };
        let s = serde_json::to_string(&t).unwrap();
        assert_eq!(
            s,
            r#"{"d":["010101010101010101010101010101010101010101010101010101010101","020202020202020202020202020202020202020202020202020202020202"],"subnet_id":"/r31415926/f2xwzbdu7z5sam6hc57xxwkctciuaz7oe5omipwbq","ipc_address":"/r31415926/f2xwzbdu7z5sam6hc57xxwkctciuaz7oe5omipwbq:f0101"}"#
        );

        let r: T = serde_json::from_str(&s).unwrap();

        assert_eq!(r, t);
    }
}
