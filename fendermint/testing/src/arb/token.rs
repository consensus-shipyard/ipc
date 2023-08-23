// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use fvm_shared::{
    bigint::{BigInt, Integer, Sign, MAX_BIGINT_SIZE},
    econ::TokenAmount,
};
use quickcheck::{Arbitrary, Gen};

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
