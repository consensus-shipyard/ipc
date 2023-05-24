// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A Genesis data structure similar to [genesis.Template](https://github.com/filecoin-project/lotus/blob/v1.20.4/genesis/types.go)
//! in Lotus, which is used to [initialize](https://github.com/filecoin-project/lotus/blob/v1.20.4/chain/gen/genesis/genesis.go) the state tree.

use fendermint_vm_core::Timestamp;
use fvm_shared::version::NetworkVersion;
use fvm_shared::{address::Address, econ::TokenAmount};
use libsecp256k1::curve::Affine;
use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use fendermint_vm_encoding::IsHumanReadable;

#[cfg(feature = "arb")]
mod arb;

/// Wrapper around [`Address`] to provide human readable serialization in JSON format.
///
/// An alternative would be the `serde_with` crate.
///
/// TODO: This is based on [Lotus](https://github.com/filecoin-project/lotus/blob/v1.20.4/genesis/types.go).
///       Not sure if anything but public key addresses make sense here. Consider using `PublicKey` instead of `Address`.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerAddr(#[serde_as(as = "IsHumanReadable")] pub Address);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub owner: SignerAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Multisig {
    pub signers: Vec<SignerAddr>,
    pub threshold: u64,
    pub vesting_duration: u64,
    pub vesting_start: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActorMeta {
    Account(Account),
    Multisig(Multisig),
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Actor {
    pub meta: ActorMeta,
    #[serde_as(as = "IsHumanReadable")]
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

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Genesis {
    /// The name of the blockchain.
    ///
    /// It will be used to derive a chain ID as well as being
    /// the network name in the `InitActor`
    pub chain_name: String,
    pub timestamp: Timestamp,
    pub network_version: NetworkVersion,
    #[serde_as(as = "IsHumanReadable")]
    pub base_fee: TokenAmount,
    pub validators: Vec<Validator>,
    pub accounts: Vec<Actor>,
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
