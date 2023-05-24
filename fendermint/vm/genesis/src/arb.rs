// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::{
    Account, Actor, ActorMeta, Genesis, Multisig, Power, SignerAddr, Validator, ValidatorKey,
};
use fendermint_testing::arb::{ArbAddress, ArbTokenAmount};
use fendermint_vm_core::Timestamp;
use fvm_shared::version::NetworkVersion;
use quickcheck::{Arbitrary, Gen};
use rand::{rngs::StdRng, SeedableRng};

impl Arbitrary for ActorMeta {
    fn arbitrary(g: &mut Gen) -> Self {
        // NOTE: Signer addresses are probably only valid with public keys, but here we don't care.
        if bool::arbitrary(g) {
            ActorMeta::Account(Account {
                owner: SignerAddr(ArbAddress::arbitrary(g).0),
            })
        } else {
            let n = u64::arbitrary(g) % 4 + 2;
            let signers = (0..n)
                .map(|_| SignerAddr(ArbAddress::arbitrary(g).0))
                .collect();
            let threshold = u64::arbitrary(g) % n + 1;
            ActorMeta::Multisig(Multisig {
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
            timestamp: Timestamp(u64::arbitrary(g)),
            chain_name: String::arbitrary(g),
            network_version: NetworkVersion::new(*g.choose(&[18u32]).unwrap()),
            base_fee: ArbTokenAmount::arbitrary(g).0,
            validators: (0..nv).map(|_| Arbitrary::arbitrary(g)).collect(),
            accounts: (0..na).map(|_| Arbitrary::arbitrary(g)).collect(),
        }
    }
}
