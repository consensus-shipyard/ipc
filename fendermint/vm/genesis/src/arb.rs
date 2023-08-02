// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::{
    ipc, Account, Actor, ActorMeta, Genesis, Multisig, Power, SignerAddr, Validator, ValidatorKey,
};
use cid::multihash::MultihashDigest;
use fendermint_testing::arb::ArbTokenAmount;
use fendermint_vm_core::Timestamp;
use fvm_shared::{address::Address, version::NetworkVersion};
use ipc_sdk::subnet_id::SubnetID;
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
            ipc: if bool::arbitrary(g) {
                Some(ipc::IpcParams::arbitrary(g))
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbSubnetID(SubnetID);

impl Arbitrary for ArbSubnetID {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let child_count = usize::arbitrary(g) % 4;

        let children = (0..child_count)
            .map(|_| {
                if bool::arbitrary(g) {
                    Address::new_id(u64::arbitrary(g))
                } else {
                    // Only expectign EAM managed delegated addresses.
                    let subaddr: [u8; 20] = std::array::from_fn(|_| Arbitrary::arbitrary(g));
                    Address::new_delegated(10, &subaddr).unwrap()
                }
            })
            .collect::<Vec<_>>();

        Self(SubnetID::new(u64::arbitrary(g), children))
    }
}

impl Arbitrary for ipc::GatewayParams {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            subnet_id: ArbSubnetID::arbitrary(g).0,
            bottom_up_check_period: u64::arbitrary(g),
            top_down_check_period: u64::arbitrary(g),
            msg_fee: ArbTokenAmount::arbitrary(g).0,
            majority_percentage: u8::arbitrary(g) % 101,
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
