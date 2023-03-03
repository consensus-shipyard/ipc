// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};
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
