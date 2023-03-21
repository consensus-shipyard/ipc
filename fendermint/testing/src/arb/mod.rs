// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use fvm_shared::{
    address::Address,
    bigint::{BigInt, Integer, Sign, MAX_BIGINT_SIZE},
    econ::TokenAmount,
};
use quickcheck::{Arbitrary, Gen};

mod cid;

pub use crate::arb::cid::ArbCid;

#[derive(Clone)]
/// Unfortunately an arbitrary `TokenAmount` is not serializable if it has more than 128 bytes, we get "BigInt too large" error.
///
/// The max below is taken from https://github.com/filecoin-project/ref-fvm/blob/fvm%40v3.0.0-alpha.24/shared/src/bigint/bigint_ser.rs#L80-L81
pub struct ArbTokenAmount(pub TokenAmount);

impl Arbitrary for ArbTokenAmount {
    fn arbitrary(g: &mut Gen) -> Self {
        let tokens = TokenAmount::arbitrary(g);
        let max_bigint = BigInt::new(Sign::Plus, vec![u32::MAX; MAX_BIGINT_SIZE / 4 - 1]);
        let atto = tokens.atto();
        let atto = atto.mod_floor(&max_bigint);
        Self(TokenAmount::from_atto(atto))
    }
}

/// Unfortunately an arbitrary `DelegatedAddress` can be inconsistent with bytes that do not correspond to its length.
#[derive(Clone)]
pub struct ArbAddress(pub Address);

impl Arbitrary for ArbAddress {
    fn arbitrary(g: &mut Gen) -> Self {
        let addr = Address::arbitrary(g);
        let bz = addr.to_bytes();
        Self(Address::from_bytes(&bz).unwrap())
    }
}
