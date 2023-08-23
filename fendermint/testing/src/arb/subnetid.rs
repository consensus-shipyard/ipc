// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use quickcheck::Arbitrary;

#[derive(Debug, Clone)]
pub struct ArbSubnetID(pub SubnetID);

impl Arbitrary for ArbSubnetID {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let child_count = usize::arbitrary(g) % 4;

        let children = (0..child_count)
            .map(|_| {
                if bool::arbitrary(g) {
                    Address::new_id(u64::arbitrary(g))
                } else {
                    // Only expecting EAM managed delegated addresses.
                    let subaddr: [u8; 20] = std::array::from_fn(|_| Arbitrary::arbitrary(g));
                    Address::new_delegated(10, &subaddr).unwrap()
                }
            })
            .collect::<Vec<_>>();

        Self(SubnetID::new(u64::arbitrary(g), children))
    }
}
