// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHash, BlockHeight, Bytes};
use anyhow::anyhow;
use ipc_ipld_resolver::ValidatorKey;
use libp2p::identity::PublicKey;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
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
    signature: SignatureInner,
    // TODO: might have to add timestamp against more attacks
}

impl SignedVote {
    /// Create a new [`SignedVoteRecord`] with secp256k1 secret key that can be shared with others
    pub fn signed(
        key: &libp2p::identity::secp256k1::Keypair,
        vote: &TopdownVote,
    ) -> anyhow::Result<Self> {
        let payload = fvm_ipld_encoding::to_vec(vote)?;
        let signature = SignatureInner::from_secp(key, &payload)?;
        Ok(Self { payload, signature })
    }

    pub fn into_validated_payload(self) -> anyhow::Result<(TopdownVote, Signature, ValidatorKey)> {
        let (pubkey, sig) = self.signature.pubkey_with_signature(&self.payload)?;
        if !pubkey.verify(&self.payload, &sig) {
            Err(anyhow!("invalid validator signature"))
        } else {
            Ok((
                fvm_ipld_encoding::from_slice(&self.payload)?,
                sig,
                ValidatorKey::from(pubkey),
            ))
        }
    }
}

impl Display for SignedVote {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SignedVote {{ payload = {}, signature = {} }}",
            hex::encode(self.payload.as_slice()),
            hex::encode(self.signature.as_slice()),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum SignatureInner {
    Secpk1Recoverable { sig: Vec<u8>, rec: u8 },
}

impl SignatureInner {
    fn from_secp(
        keypair: &libp2p::identity::secp256k1::Keypair,
        payload: &[u8],
    ) -> anyhow::Result<Self> {
        let s = secp256k1::SecretKey::from_slice(&keypair.secret().to_bytes())
            .map_err(|_| anyhow!("cannot parse secret key, should not have happen"))?;

        let secp = secp256k1::Secp256k1::new();

        let (id, sig) = secp
            .sign_ecdsa_recoverable(
                &secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(payload),
                &s,
            )
            .serialize_compact();

        Ok(Self::Secpk1Recoverable {
            sig: sig.to_vec(),
            rec: id.to_i32() as u8,
        })
    }

    fn pubkey_with_signature(self, payload: &[u8]) -> anyhow::Result<(PublicKey, Signature)> {
        match self {
            SignatureInner::Secpk1Recoverable { sig, rec } => {
                let secp = secp256k1::Secp256k1::new();

                let pubkey = secp.recover_ecdsa(
                    &secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(
                        payload,
                    ),
                    &RecoverableSignature::from_compact(&sig, RecoveryId::from_i32(rec as i32)?)?,
                )?;
                Ok((
                    libp2p::identity::secp256k1::PublicKey::try_from_bytes(&pubkey.serialize())?
                        .into(),
                    sig,
                ))
            }
        }
    }

    fn as_slice(&self) -> &[u8] {
        match self {
            SignatureInner::Secpk1Recoverable { ref sig, .. } => sig.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::voting::payload::SignatureInner;
    use rand::random;

    #[test]
    fn test_signing_and_verification() {
        let payload = (0..1024).map(|_| random::<u8>()).collect::<Vec<_>>();
        let key = libp2p::identity::secp256k1::Keypair::generate();

        let sig = SignatureInner::from_secp(&key, &payload).unwrap();

        let verified = sig.pubkey_with_signature(&payload).unwrap();
        assert_eq!(verified.0, (key.public().clone()).into())
    }
}
