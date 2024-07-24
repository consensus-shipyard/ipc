// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::voting::{Signature, Weight};
use crate::Bytes;
use anyhow::anyhow;
use bitvec::vec::BitVec;
use ipc_ipld_resolver::ValidatorKey;
use libp2p::identity::PublicKey;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::iter::zip;

/// Note that currently IPC is using seckp private keys which makes BLS impossible.
/// Ecdsa is not going to be scalable for large set of public keys, still ok for small set of public keys
///
/// Most promising solution is using Schnorr which is already implemented in Bitcoin, rust implementation
/// is still new. Keep `Schnorr` variant as kiv and should definitely implement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AggregatedSignature {
    Ecdsa(Vec<Bytes>),
    Schnorr(Bytes),
}

impl AggregatedSignature {
    /// Add the signature to the aggregate. Returns true if the signature is added else false.
    pub fn add(&mut self, sig: Bytes) -> bool {
        match self {
            AggregatedSignature::Ecdsa(sigs) => {
                if !sigs.iter().any(|v| *v == sig) {
                    sigs.push(sig);
                    true
                } else {
                    false
                }
            }
            _ => todo!(),
        }
    }

    pub fn is_valid(&self, message: &[u8], pub_keys: &[PublicKey]) -> bool {
        match self {
            AggregatedSignature::Ecdsa(ref signatures) => {
                if signatures.len() != pub_keys.len() {
                    return false;
                }

                for (sig, key) in zip(signatures, pub_keys) {
                    if !key.verify(message, sig) {
                        return false;
                    }
                }

                true
            }
            _ => todo!(),
        }
    }
}

/// The ecdsa signature aggregation quorum cert for topdown proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiSigCert {
    signed_validator_bitmap: BitVec,
    agg_signatures: AggregatedSignature,
}

impl MultiSigCert {
    pub fn signed_validators<T: Clone>(&self, validators: &[T]) -> anyhow::Result<Vec<T>> {
        if validators.len() != self.signed_validator_bitmap.len() {
            return Err(anyhow!("validators length and bitmap length not match"));
        }

        let mut pks = Vec::new();
        self.signed_validator_bitmap
            .iter()
            .enumerate()
            .for_each(|(idx, bit)| {
                if bit == true {
                    pks.push(validators[idx].clone());
                }
            });

        Ok(pks)
    }

    pub fn ecdsa(capacity: usize) -> Self {
        Self {
            signed_validator_bitmap: BitVec::with_capacity(capacity),
            agg_signatures: AggregatedSignature::Ecdsa(Vec::with_capacity(capacity)),
        }
    }

    pub fn extend<I: Iterator<Item = Option<Bytes>>>(&mut self, iter: I) {
        for sig in iter {
            let Some(sig) = sig else {
                self.signed_validator_bitmap.push(false);
                continue;
            };

            if self.agg_signatures.add(sig) {
                self.signed_validator_bitmap.push(true);
            }
        }
    }

    /// Checks if the cert contains aggregated signature signed by the public keys against the message
    pub fn is_valid(&self, message: &[u8], validator_set: &[PublicKey]) -> anyhow::Result<bool> {
        let pub_keys = self.signed_validators(validator_set)?;
        Ok(self.agg_signatures.is_valid(message, &pub_keys))
    }

    /// Checks if the cert contains aggregated signature signed by the public keys against the message
    pub fn validate_power_table<const DEN: Weight, const NUM: Weight>(
        &self,
        message: &[u8],
        powers: im::HashMap<ValidatorKey, Weight>,
    ) -> anyhow::Result<()> {
        let keys = order_validators(&powers)
            .iter()
            .map(|(validator_key, _)| (*validator_key).clone())
            .collect::<Vec<_>>();

        let signed_validators = self.signed_validators(&keys)?;

        if !threshold_reached::<DEN, NUM>(&powers, &signed_validators) {
            return Err(anyhow!("quorum threshold not reached"));
        }

        let pub_keys = keys.into_iter().map(PublicKey::from).collect::<Vec<_>>();
        if !self.is_valid(message, &pub_keys)? {
            Err(anyhow!("cert contains invalid signature"))
        } else {
            Ok(())
        }
    }
}

/// A collection of validator public key that have signed the same content.
#[derive(Default, Clone, Debug)]
pub(crate) struct ValidatorSignatures {
    validators: im::HashMap<ValidatorKey, Signature>,
}

impl ValidatorSignatures {
    pub fn empty() -> Self {
        Self {
            validators: im::HashMap::new(),
        }
    }

    pub fn validators(&self) -> Vec<&ValidatorKey> {
        self.validators.keys().collect::<Vec<_>>()
    }

    pub fn has_voted(&self, k: &ValidatorKey) -> bool {
        self.validators.contains_key(k)
    }

    /// Returns `true` if the vote is successfully added, `false` is the validator has already
    /// voted.
    pub fn add_vote(&mut self, k: ValidatorKey, sig: Signature) -> bool {
        if self.validators.contains_key(&k) {
            return false;
        }
        self.validators.insert(k, sig);
        true
    }

    pub fn to_cert(&self, power_table: &im::HashMap<ValidatorKey, Weight>) -> MultiSigCert {
        let sorted_powers = order_validators(power_table);

        let mut cert = MultiSigCert::ecdsa(sorted_powers.len());

        let iter = sorted_powers
            .into_iter()
            .map(|(validator, _)| self.validators.get(validator).cloned());

        cert.extend(iter);

        cert
    }
}

fn threshold_reached<const DEN: Weight, const NUM: Weight>(
    all: &im::HashMap<ValidatorKey, Weight>,
    signed: &[ValidatorKey],
) -> bool {
    let threshold = all.values().sum::<Weight>() * NUM / DEN;

    let mut accumulated = 0;
    for k in signed {
        accumulated += all.get(k).unwrap_or(&0);
    }

    accumulated > threshold
}

fn order_validators(
    power_table: &im::HashMap<ValidatorKey, Weight>,
) -> Vec<(&ValidatorKey, &Weight)> {
    let mut sorted_powers = power_table.iter().collect::<Vec<_>>();

    sorted_powers.sort_by(|a, b| {
        let cmp = b.1.cmp(a.1);
        if cmp != Ordering::Equal {
            cmp
        } else {
            b.0.cmp(a.0)
        }
    });

    sorted_powers
}

// Because chain interpreter is importing from fendermint_vm_message, no choice but to define the
// type twice.
impl From<fendermint_vm_message::ipc::MultiSigCert> for MultiSigCert {
    fn from(value: fendermint_vm_message::ipc::MultiSigCert) -> Self {
        Self {
            signed_validator_bitmap: value.signed_validator_bitmap,
            agg_signatures: AggregatedSignature::from(value.agg_signatures),
        }
    }
}

impl From<fendermint_vm_message::ipc::AggregatedSignature> for AggregatedSignature {
    fn from(value: fendermint_vm_message::ipc::AggregatedSignature) -> Self {
        match value {
            fendermint_vm_message::ipc::AggregatedSignature::Ecdsa(v) => Self::Ecdsa(v),
            fendermint_vm_message::ipc::AggregatedSignature::Schnorr(v) => Self::Schnorr(v),
        }
    }
}

impl From<MultiSigCert> for fendermint_vm_message::ipc::MultiSigCert {
    fn from(value: MultiSigCert) -> fendermint_vm_message::ipc::MultiSigCert {
        fendermint_vm_message::ipc::MultiSigCert {
            signed_validator_bitmap: value.signed_validator_bitmap,
            agg_signatures: fendermint_vm_message::ipc::AggregatedSignature::from(
                value.agg_signatures,
            ),
        }
    }
}

impl From<AggregatedSignature> for fendermint_vm_message::ipc::AggregatedSignature {
    fn from(value: AggregatedSignature) -> fendermint_vm_message::ipc::AggregatedSignature {
        match value {
            AggregatedSignature::Ecdsa(v) => {
                fendermint_vm_message::ipc::AggregatedSignature::Ecdsa(v)
            }
            AggregatedSignature::Schnorr(v) => {
                fendermint_vm_message::ipc::AggregatedSignature::Schnorr(v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::voting::quorum::{AggregatedSignature, MultiSigCert};
    use libp2p::identity::Keypair;

    #[test]
    fn test_cert() {
        let num_validators = 100;
        let key_pairs: Vec<Keypair> = (0..num_validators)
            .map(|_| Keypair::generate_secp256k1())
            .collect();

        let message = vec![1, 2, 3];
        let signatures = key_pairs
            .iter()
            .map(|pair| {
                if rand::random::<bool>() {
                    Some(pair.sign(&message).unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut cert = MultiSigCert::ecdsa(20);
        cert.extend(signatures.clone().into_iter());
        // add again with duplicates, should have no effect
        cert.extend(signatures.clone().into_iter().filter(|v| v.is_some()));
        match cert.agg_signatures {
            AggregatedSignature::Ecdsa(ref v) => assert_eq!(
                *v,
                signatures.clone().into_iter().flatten().collect::<Vec<_>>()
            ),
            AggregatedSignature::Schnorr(_) => unreachable!(),
        }
        assert_eq!(cert.signed_validator_bitmap.len(), key_pairs.len());

        let pub_keys = key_pairs.iter().map(|p| p.public()).collect::<Vec<_>>();
        assert!(
            cert.is_valid(&message, &pub_keys).unwrap(),
            "should validate cert"
        );

        assert!(
            !cert.is_valid(&[1, 2], &pub_keys).unwrap(),
            "should invalidate cert"
        );
    }
}
