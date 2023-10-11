// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::{
    ipc, Account, Actor, ActorMeta, Collateral, Genesis, Multisig, Power, SignerAddr, Validator,
    ValidatorKey,
};
use cid::multihash::MultihashDigest;
use fendermint_crypto::SecretKey;
use fendermint_testing::arb::{ArbSubnetID, ArbTokenAmount};
use fendermint_vm_core::Timestamp;
use fvm_shared::{
    address::Address,
    bigint::{BigInt, Integer},
    econ::TokenAmount,
    version::NetworkVersion,
};
use quickcheck::{Arbitrary, Gen};
use rand::{rngs::StdRng, SeedableRng};

impl Arbitrary for ActorMeta {
    fn arbitrary(g: &mut Gen) -> Self {
        // Generate keys which the loader knows how to initialize.
        if bool::arbitrary(g) {
            let pk = ValidatorKey::arbitrary(g).0;
            let pk = pk.serialize();
            let addr = if bool::arbitrary(g) {
                Address::new_secp256k1(&pk).unwrap()
            } else {
                // NOTE: Not using `EthAddress` because it would be circular dependency.
                let mut hash20 = [0u8; 20];
                let hash32 = cid::multihash::Code::Keccak256.digest(&pk[1..]);
                hash20.copy_from_slice(&hash32.digest()[12..]);
                Address::new_delegated(10, &hash20).unwrap()
            };
            ActorMeta::Account(Account {
                owner: SignerAddr(addr),
            })
        } else {
            let n = u64::arbitrary(g) % 4 + 2;
            let signers = (0..n)
                .map(|_| {
                    let pk = ValidatorKey::arbitrary(g).0;
                    let addr = Address::new_secp256k1(&pk.serialize()).unwrap();
                    SignerAddr(addr)
                })
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
        let sk = SecretKey::random(&mut rng);
        let pk = sk.public_key();
        Self::new(pk)
    }
}

impl Arbitrary for Collateral {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(ArbTokenAmount::arbitrary(g).0)
    }
}

impl Arbitrary for Power {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(u64::arbitrary(g))
    }
}

impl<P: Arbitrary> Arbitrary for Validator<P> {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            public_key: ValidatorKey::arbitrary(g),
            power: P::arbitrary(g),
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
            power_scale: *g.choose(&[-1, 0, 3]).unwrap(),
            validators: (0..nv).map(|_| Arbitrary::arbitrary(g)).collect(),
            accounts: (0..na).map(|_| Arbitrary::arbitrary(g)).collect(),
            ipc: if bool::arbitrary(g) {
                Some(ipc::IpcParams::arbitrary(g))
            } else {
                None
            },
        }
    }
}

/// `TokenAmount` well within the limits of U256
#[derive(Debug, Clone)]
struct ArbFee(TokenAmount);

impl Arbitrary for ArbFee {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let t = ArbTokenAmount::arbitrary(g).0;
        let (_, t) = t.atto().div_mod_floor(&BigInt::from(u64::MAX));
        Self(TokenAmount::from_atto(t))
    }
}

impl Arbitrary for ipc::GatewayParams {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            subnet_id: ArbSubnetID::arbitrary(g).0,
            // Gateway constructor would reject 0.
            bottom_up_check_period: u64::arbitrary(g).max(1),
            top_down_check_period: u64::arbitrary(g),
            // Gateway constructor would reject 0.
            min_collateral: ArbFee::arbitrary(g).0.max(TokenAmount::from_atto(1)),
            msg_fee: ArbFee::arbitrary(g).0,
            majority_percentage: u8::arbitrary(g) % 50 + 51,
            active_validators_limit: u16::arbitrary(g) % 100 + 1,
        }
    }
}

impl Arbitrary for ipc::IpcParams {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            gateway: ipc::GatewayParams::arbitrary(g),
        }
    }
}
