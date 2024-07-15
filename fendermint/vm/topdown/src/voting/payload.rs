// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHash, BlockHeight, Bytes};
use anyhow::anyhow;
use ipc_ipld_resolver::ValidatorKey;
use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};

pub type Signature = Bytes;

/// The different versions of vote casted in topdown gossip pub-sub channel
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Eq, PartialEq)]
pub enum TopdownVote {
    V1(TopdownVoteV1),
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Eq, PartialEq)]
pub struct TopdownVoteV1 {
    // TODO: add subnet id as part of the vote
    // parent: SubnetID,
    block_height: BlockHeight,
    /// The block hash of the parent chain at the `block_height`.
    block_hash: BlockHash,
    /// The commitment or proof of the side effects (topdown messages and validator changes)
    commitment: Bytes,
}

impl TopdownVote {
    pub fn v1(block_height: BlockHeight, block_hash: BlockHash, commitment: Bytes) -> Self {
        Self::V1(TopdownVoteV1 {
            block_height,
            block_hash,
            commitment,
        })
    }

    pub fn block_height(&self) -> BlockHeight {
        match self {
            Self::V1(v) => v.block_height,
        }
    }
}

/// The vote submitted to the vote tally
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SignedVote {
    pub(crate) payload: Bytes,
    /// The signature of the signed content using the pubkey
    signature: Signature,
    pub(crate) pubkey: ValidatorKey,
    // TODO: might have to add timestamp against more attacks
}

impl SignedVote {
    /// Create a new [`SignedVoteRecord`] with the current timestamp
    /// and a signed envelope which can be shared with others.
    pub fn signed(key: &Keypair, vote: &TopdownVote) -> anyhow::Result<Self> {
        let payload = fvm_ipld_encoding::to_vec(vote)?;
        let signature = key.sign(&payload)?;
        let pubkey = ValidatorKey::from(key.public());
        Ok(Self {
            payload,
            signature,
            pubkey,
        })
    }

    pub fn into_validated_payload(self) -> anyhow::Result<(TopdownVote, Signature, ValidatorKey)> {
        if !self.pubkey.verify(&self.payload, &self.signature) {
            Err(anyhow!("invalid validator signature"))
        } else {
            Ok((
                fvm_ipld_encoding::from_slice(&self.payload)?,
                self.signature,
                self.pubkey,
            ))
        }
    }
}
