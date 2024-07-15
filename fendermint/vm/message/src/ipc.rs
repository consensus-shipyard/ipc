// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use bitvec::vec::BitVec;
use cid::multihash::Code;
use cid::multihash::MultihashDigest;
use cid::Cid;
use fvm_ipld_encoding::DAG_CBOR;
use fvm_shared::{
    address::Address, clock::ChainEpoch, crypto::signature::Signature, econ::TokenAmount,
};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::StakingChangeRequest;
use ipc_api::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};

/// Messages involved in InterPlanetary Consensus.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum IpcMessage {
    /// A bottom-up checkpoint coming from a child subnet "for resolution", relayed by a user of the parent subnet for a reward.
    ///
    /// The reward can be given immediately upon the validation of the quorum certificate in the checkpoint,
    /// or later during execution, once data availability has been confirmed.
    BottomUpResolve(SignedRelayedMessage<CertifiedMessage<BottomUpCheckpoint>>),

    /// A bottom-up checkpoint proposed "for execution" by the parent subnet validators, provided that the majority of them
    /// have the data available to them, already resolved.
    ///
    /// To prove that the data is available, we can either use the ABCI++ "process proposal" mechanism,
    /// or we can gossip votes using the _IPLD Resolver_ and attach them as a quorum certificate.
    BottomUpExec(CertifiedMessage<BottomUpCheckpoint>),

    /// A top-down checkpoint parent finality proposal. This proposal should contain the latest parent
    /// state that to be checked and voted by validators.
    TopDownExec(TopdownProposalWithCert),
}

/// A message relayed by a user on the current subnet.
///
/// The relayer pays for the inclusion of the message in the ledger,
/// but not necessarily for the execution of its contents.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayedMessage<T> {
    /// The relayed message.
    pub message: T,
    /// The address (public key) of the relayer in the current subnet.
    pub relayer: Address,
    /// The nonce of the relayer in the current subnet.
    pub sequence: u64,
    /// The gas the relayer is willing to spend on the verification of the relayed message.
    pub gas_limit: u64,
    pub gas_fee_cap: TokenAmount,
    pub gas_premium: TokenAmount,
}

/// Relayed messages are signed by the relayer, so we can rightfully charge them message inclusion costs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedRelayedMessage<T> {
    /// The relayed message with the relayer identity.
    pub message: RelayedMessage<T>,
    /// The signature of the relayer, for cost and reward attribution.
    pub signature: Signature,
}

/// A message with a quorum certificate from a group of validators.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CertifiedMessage<T> {
    /// The message the validators signed.
    pub message: T,
    /// The quorum certificate.
    pub certificate: MultiSig,
}

/// A quorum certificate consisting of a simple multi-sig.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MultiSig {
    pub signatures: Vec<ValidatorSignature>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Signature,
}

/// A periodic bottom-up checkpoints contains the source subnet ID (to protect against replay attacks),
/// a block height (for sequencing), any potential handover to the next validator set, and a pointer
/// to the messages that need to be resolved and executed by the parent validators.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BottomUpCheckpoint {
    /// Which subnet is the checkpoint coming from.
    pub subnet_id: SubnetID,
    /// Block height of this checkpoint.
    pub height: ChainEpoch,
    /// Which validator set is going to sign the *next* checkpoint.
    /// The parent subnet already expects the last validator set to sign this one.
    pub next_validator_set_id: u64,
    /// Pointer at all the bottom-up messages included in this checkpoint.
    pub bottom_up_messages: Cid, // TODO: Use TCid
}

/// The ecdsa signature aggregation quorum cert for topdown proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiSigCert {
    pub signed_validator_bitmap: BitVec,
    pub agg_signatures: AggregatedSignature,
}

/// Note that currently IPC is using seckp private keys which makes BLS impossible.
/// Ecdsa is not going to be scalable for large set of public keys, still ok for small set of public keys
///
/// Most promising solution is using Schnorr which is already implemented in Bitcoin, rust implementation
/// is still new. Keep `Schnorr` variant as kiv and should definitely implement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AggregatedSignature {
    Ecdsa(Vec<Vec<u8>>),
    Schnorr(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TopdownProposal {
    V1(ParentFinalityPayload),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopdownProposalWithCert {
    pub proposal: TopdownProposal,
    pub cert: MultiSigCert,
}

/// A proposal of the parent view that validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParentFinalityPayload {
    /// Block height of this proposal.
    pub height: ChainEpoch,
    /// The block hash of the parent, expressed as bytes
    pub block_hash: Vec<u8>,
    /// The topdown messages to be executed.
    ///
    /// Note that this is not the cross messages at the `height`,
    /// but instead the cross messages since the last topdown finality to the current `height`.
    pub cross_messages: Vec<IpcEnvelope>,
    /// The validator changes to be applied
    ///
    /// Note that this is not the validator changes at the `height`,
    /// but instead the validator changes since the last topdown finality to the current `height`.
    pub validator_changes: Vec<StakingChangeRequest>,
}

impl ParentFinalityPayload {
    pub fn new(
        height: ChainEpoch,
        hash: Vec<u8>,
        cross_messages: Vec<IpcEnvelope>,
        validator_changes: Vec<StakingChangeRequest>,
    ) -> Self {
        Self {
            height,
            block_hash: hash,
            cross_messages,
            validator_changes,
        }
    }

    pub fn side_effect_cid(&self) -> Cid {
        let bytes = fvm_ipld_encoding::to_vec(&(&self.cross_messages, &self.validator_changes))
            .expect("should not have failed");
        Cid::new_v1(DAG_CBOR, Code::Blake2b256.digest(&bytes))
    }
}

#[cfg(feature = "arb")]
mod arb {

    use crate::ipc::ParentFinalityPayload;
    use fendermint_testing::arb::{ArbAddress, ArbCid, ArbSubnetID, ArbTokenAmount};
    use fvm_shared::crypto::signature::Signature;
    use quickcheck::{Arbitrary, Gen};

    use super::{
        BottomUpCheckpoint, CertifiedMessage, IpcMessage, MultiSig, RelayedMessage,
        SignedRelayedMessage, ValidatorSignature,
    };

    impl Arbitrary for IpcMessage {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 3 {
                0 => IpcMessage::BottomUpResolve(Arbitrary::arbitrary(g)),
                1 => IpcMessage::BottomUpExec(Arbitrary::arbitrary(g)),
                _ => IpcMessage::TopDownExec(Arbitrary::arbitrary(g)),
            }
        }
    }

    impl<T: Arbitrary> Arbitrary for SignedRelayedMessage<T> {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                message: RelayedMessage::arbitrary(g),
                signature: Signature::arbitrary(g),
            }
        }
    }

    impl<T: Arbitrary> Arbitrary for RelayedMessage<T> {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                message: T::arbitrary(g),
                relayer: ArbAddress::arbitrary(g).0,
                sequence: u64::arbitrary(g),
                gas_limit: u64::arbitrary(g),
                gas_fee_cap: ArbTokenAmount::arbitrary(g).0,
                gas_premium: ArbTokenAmount::arbitrary(g).0,
            }
        }
    }

    impl<T: Arbitrary> Arbitrary for CertifiedMessage<T> {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                message: T::arbitrary(g),
                certificate: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for ValidatorSignature {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                validator: ArbAddress::arbitrary(g).0,
                signature: Signature::arbitrary(g),
            }
        }
    }

    impl Arbitrary for MultiSig {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut signatures = Vec::new();
            for _ in 0..*g.choose(&[1, 3, 5]).unwrap() {
                signatures.push(ValidatorSignature::arbitrary(g));
            }
            Self { signatures }
        }
    }

    impl Arbitrary for BottomUpCheckpoint {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                subnet_id: ArbSubnetID::arbitrary(g).0,
                height: u32::arbitrary(g).into(),
                next_validator_set_id: Arbitrary::arbitrary(g),
                bottom_up_messages: ArbCid::arbitrary(g).0,
            }
        }
    }

    impl Arbitrary for ParentFinalityPayload {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                height: u32::arbitrary(g).into(),
                block_hash: Vec::arbitrary(g),
                cross_messages: vec![],
                validator_changes: vec![],
            }
        }
    }
}
