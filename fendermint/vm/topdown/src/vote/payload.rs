// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::Weight;
use crate::{BlockHash, BlockHeight, Bytes};
use anyhow::anyhow;
use fendermint_vm_genesis::ValidatorKey;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub type Signature = Bytes;
pub type RecoverableECDSASignature = (i32, Vec<u8>);

pub type PowerTable = HashMap<ValidatorKey, Weight>;

/// The different versions of vote casted in topdown gossip pub-sub channel
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Vote {
    V1 {
        validator: ValidatorKey,
        payload: CertifiedObservation,
    },
}

/// The actual content that validators should agree upon, or put in another way the content
/// that a quorum should be formed upon
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Eq, PartialEq)]
pub struct Ballot {
    parent_height: u64,
    /// The hash of the chain unit at that height. Usually a block hash, but could
    /// be another entity (e.g. tipset CID), depending on the parent chain
    /// and our interface to it. For example, if the parent is a Filecoin network,
    /// this would be a tipset CID coerced into a block hash if queried through
    /// the Eth API, or the tipset CID as-is if accessed through the Filecoin API.
    parent_hash: Bytes,
    /// A rolling/cumulative commitment to topdown effects since the beginning of
    /// time, including the ones in this block.
    cumulative_effects_comm: Bytes,
}

/// The content that validators gossip among each other
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Observation {
    /// The hash of the subnet's last committed block when this observation was made.
    /// Used to discard stale observations that are, e.g. replayed by an attacker
    /// at a later time. Also used to detect nodes that might be wrongly gossiping
    /// whilst being out of sync.
    local_hash: BlockHash,
    ballot: Ballot,
}

/// A self-certified observation made by a validator.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CertifiedObservation {
    observed: Observation,
    /// A "recoverable" ECDSA signature with the validator's secp256k1 private key over the
    /// CID of the DAG-CBOR encoded observation using a BLAKE2b-256 multihash.
    signature: RecoverableECDSASignature,
}

impl Vote {
    pub fn v1(obs: CertifiedObservation) -> anyhow::Result<Self> {
        let to_sign = fvm_ipld_encoding::to_vec(&obs.observed)?;
        let (validator, _) = recover_ecdsa_sig(&to_sign, obs.signature.0, &obs.signature.1)?;
        Ok(Self::V1 {
            validator,
            payload: obs,
        })
    }

    pub fn voter(&self) -> ValidatorKey {
        match self {
            Self::V1 { validator, .. } => validator.clone(),
        }
    }

    pub fn ballot(&self) -> &Ballot {
        match self {
            Self::V1 { payload, .. } => &payload.observed.ballot,
        }
    }
}

impl TryFrom<&[u8]> for Vote {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let version = bytes[0];

        if version == 0 {
            return Self::v1(CertifiedObservation::try_from(&bytes[1..])?);
        }

        Err(anyhow!("invalid vote version"))
    }
}

impl TryFrom<&[u8]> for CertifiedObservation {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(fvm_ipld_encoding::from_slice(bytes)?)
    }
}

fn sign_recoverable_ecdsa() -> anyhow::Result<RecoverableECDSASignature> {

}

fn recover_ecdsa_sig(
    payload: &[u8],
    rec_id: i32,
    sig: &[u8],
) -> anyhow::Result<(ValidatorKey, Signature)> {
    let secp = secp256k1::Secp256k1::new();

    let message = secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(payload);
    let pubkey = secp.recover_ecdsa(
        &message,
        &RecoverableSignature::from_compact(sig, RecoveryId::from_i32(rec_id)?)?,
    )?;
    let signature = secp256k1::ecdsa::Signature::from_compact(sig)?
        .serialize_der()
        .to_vec();

    Ok((
        ValidatorKey::from_compressed_pubkey(&pubkey.serialize())?,
        signature,
    ))
}

impl Display for Ballot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ballot(parent_height={}, parent_hash={}, commitment={})",
            self.parent_height,
            hex::encode(&self.parent_hash),
            hex::encode(&self.cumulative_effects_comm),
        )
    }
}

impl Ballot {
    pub fn parent_height(&self) -> BlockHeight {
        self.parent_height
    }
}
