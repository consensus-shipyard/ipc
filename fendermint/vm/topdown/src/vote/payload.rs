// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::observation::{Ballot, ObservationCommitment};
use crate::vote::Weight;
use crate::BlockHeight;
use anyhow::anyhow;
use fendermint_crypto::secp::RecoverableECDSASignature;
use fendermint_crypto::SecretKey;
use fendermint_vm_genesis::ValidatorKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// A self-certified observation made by a validator.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CertifiedObservation {
    observed: ObservationCommitment,
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
        let to_sign = fvm_ipld_encoding::to_vec(&obs.observed)?;
        let (pk, _) = obs.signature.clone().recover(&to_sign)?;

        Ok(Self::V1 {
            validator: ValidatorKey::new(pk),
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
    pub fn sign(ob: ObservationCommitment, sk: &SecretKey) -> anyhow::Result<Self> {
        let to_sign = fvm_ipld_encoding::to_vec(&ob)?;
        let sig = RecoverableECDSASignature::sign(sk, to_sign.as_slice())?;
        Ok(Self {
            observed: ob,
            signature: sig,
        })
    }
}
