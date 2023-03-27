// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A Genesis data structure similar to [genesis.Template](https://github.com/filecoin-project/lotus/blob/v1.20.4/genesis/types.go)
//! in Lotus, which is used to [initialize](https://github.com/filecoin-project/lotus/blob/v1.20.4/chain/gen/genesis/genesis.go) the state tree.

use std::str::FromStr;

use fvm_shared::bigint::BigInt;
use fvm_shared::version::NetworkVersion;
use fvm_shared::{address::Address, econ::TokenAmount};
use libsecp256k1::curve::Affine;
use libsecp256k1::PublicKey;
use num_traits::Num;
use serde::de::Error;
use serde::{de, Deserialize, Serialize, Serializer};

/// Wrapper around [`Address`] to provide human readable serialization in JSON format.
///
/// An alternative would be the `serde_with` crate.
///
/// TODO: This is based on [Lotus](https://github.com/filecoin-project/lotus/blob/v1.20.4/genesis/types.go).
///       Not sure if anything but public key addresses make sense here. Consider using `PublicKey` instead of `Address`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorAddr(pub Address);

impl Serialize for ActorAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.0.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for ActorAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            match Address::from_str(&s) {
                Ok(a) => Ok(Self(a)),
                Err(e) => Err(D::Error::custom(format!(
                    "error deserializing address: {}",
                    e
                ))),
            }
        } else {
            Address::deserialize(deserializer).map(Self)
        }
    }
}

/// Serialize tokens as human readable string.
fn serialize_tokens<S>(tokens: &TokenAmount, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        tokens.atto().to_str_radix(10).serialize(serializer)
    } else {
        tokens.serialize(serializer)
    }
}

fn deserialize_tokens<'de, D>(deserializer: D) -> Result<TokenAmount, D::Error>
where
    D: de::Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        let s = String::deserialize(deserializer)?;
        match BigInt::from_str_radix(&s, 10) {
            Ok(a) => Ok(TokenAmount::from_atto(a)),
            Err(e) => Err(D::Error::custom(format!(
                "error deserializing tokens: {}",
                e
            ))),
        }
    } else {
        TokenAmount::deserialize(deserializer)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub owner: ActorAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Multisig {
    pub signers: Vec<ActorAddr>,
    pub threshold: u64,
    pub vesting_duration: u64,
    pub vesting_start: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActorMeta {
    Account(Account),
    MultiSig(Multisig),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Actor {
    pub meta: ActorMeta,
    #[serde(
        serialize_with = "serialize_tokens",
        deserialize_with = "deserialize_tokens"
    )]
    pub balance: TokenAmount,
}

/// Total stake delegated to this validator.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Power(pub u64);

/// Secp256k1 public key of the validators.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorKey(pub PublicKey);

impl ValidatorKey {
    /// Create a new key and make sure the wrapped public key is normalized,
    /// which is to ensure the results look the same after a serialization roundtrip.
    pub fn new(key: PublicKey) -> Self {
        let mut aff: Affine = key.into();
        aff.x.normalize();
        aff.y.normalize();
        let key = PublicKey::try_from(aff).unwrap();
        Self(key)
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.0
    }
}

/// A genesis validator with their initial power.
///
/// An [`Address`] would be enough to validate signatures, however
/// we will always need the public key to return updates in the
/// power distribution to Tendermint; it is easiest to ask for
/// the full public key.
///
/// Note that we could get the validators from `InitChain` through
/// the ABCI, but then we'd have to handle the case of a key we
/// don't know how to turn into an [`Address`]. This way leaves
/// less room for error, and we can pass all the data to the FVM
/// in one go.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Validator {
    pub public_key: ValidatorKey,
    pub power: Power,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Genesis {
    pub network_name: String,
    pub network_version: NetworkVersion,
    #[serde(
        serialize_with = "serialize_tokens",
        deserialize_with = "deserialize_tokens"
    )]
    pub base_fee: TokenAmount,
    pub validators: Vec<Validator>,
    pub accounts: Vec<Actor>,
}

#[cfg(feature = "arb")]
mod arb {
    use crate::{
        Account, Actor, ActorAddr, ActorMeta, Genesis, Multisig, Power, Validator, ValidatorKey,
    };
    use fendermint_testing::arb::{ArbAddress, ArbTokenAmount};
    use fvm_shared::version::NetworkVersion;
    use quickcheck::{Arbitrary, Gen};
    use rand::{rngs::StdRng, SeedableRng};

    impl Arbitrary for ActorMeta {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                ActorMeta::Account(Account {
                    owner: ActorAddr(ArbAddress::arbitrary(g).0),
                })
            } else {
                let n = u64::arbitrary(g) % 4 + 2;
                let signers = (0..n)
                    .map(|_| ActorAddr(ArbAddress::arbitrary(g).0))
                    .collect();
                let threshold = u64::arbitrary(g) % n + 1;
                ActorMeta::MultiSig(Multisig {
                    signers,
                    threshold,
                    vesting_duration: u64::arbitrary(g),
                    vesting_start: u64::arbitrary(g),
                })
            }
        }
    }

    impl Arbitrary for Actor {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                meta: ActorMeta::arbitrary(g),
                balance: ArbTokenAmount::arbitrary(g).0,
            }
        }
    }

    impl Arbitrary for ValidatorKey {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let sk = libsecp256k1::SecretKey::random(&mut rng);
            let pk = libsecp256k1::PublicKey::from_secret_key(&sk);
            Self::new(pk)
        }
    }

    impl Arbitrary for Validator {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                public_key: ValidatorKey::arbitrary(g),
                power: Power(u64::arbitrary(g)),
            }
        }
    }

    impl Arbitrary for Genesis {
        fn arbitrary(g: &mut Gen) -> Self {
            let nv = usize::arbitrary(g) % 10 + 1;
            let na = usize::arbitrary(g) % 10;
            Self {
                network_name: String::arbitrary(g),
                network_version: NetworkVersion::new(*g.choose(&[18u32]).unwrap()),
                base_fee: ArbTokenAmount::arbitrary(g).0,
                validators: (0..nv).map(|_| Arbitrary::arbitrary(g)).collect(),
                accounts: (0..na).map(|_| Arbitrary::arbitrary(g)).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use crate::Genesis;

    #[quickcheck]
    fn genesis_json(value0: Genesis) {
        let repr = serde_json::to_string(&value0).expect("failed to encode");
        let value1: Genesis = serde_json::from_str(&repr).expect("failed to decode");

        assert_eq!(value1, value0)
    }

    #[quickcheck]
    fn genesis_cbor(value0: Genesis) {
        let repr = fvm_ipld_encoding::to_vec(&value0).expect("failed to encode");
        let value1: Genesis = fvm_ipld_encoding::from_slice(&repr).expect("failed to decode");

        assert_eq!(value1, value0)
    }
}
