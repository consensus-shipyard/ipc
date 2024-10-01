// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::Weight;
use crate::{BlockHeight, Bytes};
use anyhow::anyhow;
use arbitrary::Arbitrary;
use fendermint_crypto::secp::RecoverableECDSASignature;
use fendermint_crypto::SecretKey;
use fendermint_vm_genesis::ValidatorKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub type PowerTable = HashMap<ValidatorKey, Weight>;
pub type PowerUpdates = Vec<(ValidatorKey, Weight)>;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct VoteTallyState {
    pub last_finalized_height: BlockHeight,
    pub quorum_threshold: Weight,
    pub power_table: PowerTable,
}

/// The different versions of vote casted in topdown gossip pub-sub channel
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Vote {
    V1 {
        validator: ValidatorKey,
        payload: CertifiedObservation,
    },
}

/// The content that validators gossip among each other.
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Eq, PartialEq, Arbitrary)]
pub struct Observation {
    pub(crate) parent_height: u64,
    /// The hash of the chain unit at that height. Usually a block hash, but could
    /// be another entity (e.g. tipset CID), depending on the parent chain
    /// and our interface to it. For example, if the parent is a Filecoin network,
    /// this would be a tipset CID coerced into a block hash if queried through
    /// the Eth API, or the tipset CID as-is if accessed through the Filecoin API.
    pub(crate) parent_hash: Bytes,
    /// A rolling/cumulative commitment to topdown effects since the beginning of
    /// time, including the ones in this block.
    pub(crate) cumulative_effects_comm: Bytes,
}

/// A self-certified observation made by a validator.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CertifiedObservation {
    observation: Observation,
    /// The signature for the observation only
    observation_signature: RecoverableECDSASignature,
    /// The hash of the subnet's last committed block when this observation was made.
    /// Used to discard stale observations that are, e.g. replayed by an attacker
    /// at a later time. Also used to detect nodes that might be wrongly gossiping
    /// whilst being out of sync.
    certified_at: BlockHeight,
    /// A "recoverable" ECDSA signature with the validator's secp256k1 private key over the
    /// CID of the DAG-CBOR encoded observation using a BLAKE2b-256 multihash.
    signature: RecoverableECDSASignature,
}

impl Vote {
    pub fn v1(validator_key: ValidatorKey, obs: CertifiedObservation) -> Self {
        Self::V1 {
            validator: validator_key,
            payload: obs,
        }
    }

    pub fn v1_checked(obs: CertifiedObservation) -> anyhow::Result<Self> {
        Ok(Self::V1 {
            validator: obs.ensure_valid()?,
            payload: obs,
        })
    }

    pub fn voter(&self) -> ValidatorKey {
        match self {
            Self::V1 { validator, .. } => validator.clone(),
        }
    }

    pub fn observation(&self) -> &Observation {
        match self {
            Self::V1 { payload, .. } => &payload.observation,
        }
    }
}

impl TryFrom<&[u8]> for Vote {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let version = bytes[0];

        if version == 0 {
            return Self::v1_checked(CertifiedObservation::try_from(&bytes[1..])?);
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

impl CertifiedObservation {
    pub fn ensure_valid(&self) -> anyhow::Result<ValidatorKey> {
        let to_sign = fvm_ipld_encoding::to_vec(&self.observation)?;
        let (pk1, _) = self.observation_signature.recover(&to_sign)?;

        let p = Self::envelop_payload(&self.observation_signature, self.certified_at)?;
        let (pk2, _) = self.signature.recover(p.as_slice())?;

        if pk1 != pk2 {
            return Err(anyhow!("public keys not aligned"));
        }

        Ok(ValidatorKey::new(pk1))
    }

    fn envelop_payload(
        observation_sig: &RecoverableECDSASignature,
        certified_at: BlockHeight,
    ) -> anyhow::Result<Bytes> {
        Ok(fvm_ipld_encoding::to_vec(&(observation_sig, certified_at))?)
    }

    pub fn sign(
        ob: Observation,
        certified_at: BlockHeight,
        sk: &SecretKey,
    ) -> anyhow::Result<Self> {
        let obs_payload = fvm_ipld_encoding::to_vec(&ob)?;
        let obs_sig = RecoverableECDSASignature::sign(sk, obs_payload.as_slice())?;

        let p = Self::envelop_payload(&obs_sig, certified_at)?;
        let sig = RecoverableECDSASignature::sign(sk, p.as_slice())?;
        Ok(Self {
            observation: ob,
            observation_signature: obs_sig,
            certified_at,
            signature: sig,
        })
    }
}

impl Observation {
    pub fn new(parent_height: BlockHeight, parent_hash: Bytes, commitment: Bytes) -> Self {
        Self {
            parent_height,
            parent_hash,
            cumulative_effects_comm: commitment,
        }
    }
}

impl Display for Observation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Observation(parent_height={}, parent_hash={}, commitment={})",
            self.parent_height,
            hex::encode(&self.parent_hash),
            hex::encode(&self.cumulative_effects_comm),
        )
    }
}

impl Observation {
    pub fn parent_height(&self) -> BlockHeight {
        self.parent_height
    }
}
