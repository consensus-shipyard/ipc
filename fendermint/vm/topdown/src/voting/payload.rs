// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHash, BlockHeight, Bytes};
use anyhow::anyhow;
use ipc_ipld_resolver::ValidatorKey;
use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type Signature = Bytes;

/// The different versions of vote casted in topdown gossip pub-sub channel
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Eq, PartialEq)]
pub struct TopdownVote {
    version: u8,
    block_height: BlockHeight,
    /// The content that represents the data to be voted on for the block height
    payload: Bytes,
}

impl TopdownVote {
    pub fn v1(block_height: BlockHeight, mut block_hash: BlockHash, commitment: Bytes) -> Self {
        block_hash.extend(commitment);
        Self {
            version: 1,
            block_height,
            payload: block_hash,
        }
    }

    /// The bytes that it will be signed and voted on
    pub fn ballot(&self) -> anyhow::Result<Bytes> {
        Ok(fvm_ipld_encoding::to_vec(self)?)
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }

    pub fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}

impl Display for TopdownVote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TopdownVote {{ payload = {}, version = {}, block_height = {} }}",
            hex::encode(self.payload.as_slice()),
            self.version,
            self.block_height,
        )
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
        let payload = vote.ballot()?;
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

impl Display for SignedVote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SignedVote {{ payload = {}, signature = {}, pubkey = {} }}",
            hex::encode(self.payload.as_slice()),
            hex::encode(self.signature.as_slice()),
            hex::encode(self.pubkey.to_bytes().as_slice()),
        )
    }
}
