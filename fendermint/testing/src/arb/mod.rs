// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use fvm_shared::{
    address::Address,
    bigint::{BigInt, Integer, Sign, MAX_BIGINT_SIZE},
    econ::TokenAmount,
    message::Message,
};
use quickcheck::{Arbitrary, Gen};

mod cid;

pub use crate::arb::cid::ArbCid;

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ArbAddress(pub Address);

impl Arbitrary for ArbAddress {
    fn arbitrary(g: &mut Gen) -> Self {
        let addr = Address::arbitrary(g);
        let bz = addr.to_bytes();
        Self(Address::from_bytes(&bz).unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct ArbMessage(pub Message);

impl Arbitrary for ArbMessage {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut message = Message::arbitrary(g);
        message.gas_fee_cap = ArbTokenAmount::arbitrary(g).0;
        message.gas_premium = ArbTokenAmount::arbitrary(g).0;
        message.value = ArbTokenAmount::arbitrary(g).0;
        message.to = ArbAddress::arbitrary(g).0;
        message.from = ArbAddress::arbitrary(g).0;
        Self(message)
    }
}
