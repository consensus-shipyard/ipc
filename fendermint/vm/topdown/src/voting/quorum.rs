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
    pub fn signed_validators(&self, validators: &[PublicKey]) -> anyhow::Result<Vec<PublicKey>> {
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
}

/// A collection of validator public key that have signed the same content.
#[derive(Default, Clone)]
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
        let mut sorted_powers = power_table.iter().collect::<Vec<_>>();

        sorted_powers.sort_by(|a, b| {
            let cmp = b.1.cmp(a.1);
            if cmp != Ordering::Equal {
                cmp
            } else {
                b.0.cmp(a.0)
            }
        });

        let mut cert = MultiSigCert::ecdsa(sorted_powers.len());

        let iter = sorted_powers
            .into_iter()
            .map(|(validator, _)| self.validators.get(validator).cloned());

        cert.extend(iter);

        cert
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
