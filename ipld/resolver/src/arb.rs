// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};
use libipld::{Cid, Multihash};
use quickcheck::Arbitrary;

/// Unfortunately an arbitrary `DelegatedAddress` can be inconsistent
/// with bytes that do not correspond to its length. This struct fixes
/// that so we can generate arbitrary addresses that don't fail equality
/// after a roundtrip.
#[derive(Clone, Debug)]
pub struct ArbAddress(pub Address);

impl Arbitrary for ArbAddress {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let addr = Address::arbitrary(g);
        let bz = addr.to_bytes();
        let addr = Address::from_bytes(&bz).expect("address roundtrip works");
        Self(addr)
    }
}

#[derive(Clone, Debug)]
pub struct ArbSubnetID(pub SubnetID);

impl Arbitrary for ArbSubnetID {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut parent = ROOTNET_ID.clone();
        for _ in 0..=u8::arbitrary(g) % 5 {
            let addr = ArbAddress::arbitrary(g).0;
            parent = SubnetID::new_from_parent(&parent, addr);
        }
        Self(parent)
    }
}

/// Unfortunately ref-fvm depends on cid:0.8.6, which depends on quickcheck:0.9
/// whereas here we use quickcheck:1.0. This causes conflicts and the `Arbitrary`
/// implementations for `Cid` are not usable to us, nor can we patch all `cid`
/// dependencies to use 0.9 because then the IPLD and other FVM traits don't work.
///
/// TODO: Remove this module when the `cid` dependency is updated.
///
/// NOTE: This is based on the [simpler version](https://github.com/ChainSafe/forest/blob/v0.6.0/blockchain/blocks/src/lib.rs) in Forest.
///       The original uses weighted distributions to generate more plausible CIDs.
#[derive(Clone)]
pub struct ArbCid(pub Cid);

impl Arbitrary for ArbCid {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(Cid::new_v1(
            u64::arbitrary(g),
            Multihash::wrap(u64::arbitrary(g), &[u8::arbitrary(g)]).unwrap(),
        ))
    }
}
