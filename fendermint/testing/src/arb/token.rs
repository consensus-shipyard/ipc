// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use ethers::types::U256;
use fvm_shared::{
    bigint::{BigInt, Integer, Sign, MAX_BIGINT_SIZE},
    econ::TokenAmount,
};
use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen};
use std::str::FromStr;

lazy_static! {
    /// The max below is taken from https://github.com/filecoin-project/ref-fvm/blob/fvm%40v3.0.0-alpha.24/shared/src/bigint/bigint_ser.rs#L80-L81
    static ref MAX_BIGINT: BigInt =
        BigInt::new(Sign::Plus, vec![u32::MAX; MAX_BIGINT_SIZE / 4 - 1]);

    static ref MAX_U256: BigInt = BigInt::from_str(&U256::MAX.to_string()).unwrap();

    // Restrict maximum token value to what we can actually pass to Ethereum.
    static ref MAX_ATTO: BigInt = MAX_BIGINT.clone().min(MAX_U256.clone());
}

#[derive(Clone, Debug)]
/// Unfortunately an arbitrary `TokenAmount` is not serializable if it has more than 128 bytes, we get "BigInt too large" error.
///
/// The max below is taken from https://github.com/filecoin-project/ref-fvm/blob/fvm%40v3.0.0-alpha.24/shared/src/bigint/bigint_ser.rs#L80-L81
pub struct ArbTokenAmount(pub TokenAmount);

impl Arbitrary for ArbTokenAmount {
    fn arbitrary(g: &mut Gen) -> Self {
        let tokens = TokenAmount::arbitrary(g);
        let atto = tokens.atto();
        let atto = atto.mod_floor(&MAX_ATTO);
        Self(TokenAmount::from_atto(atto))
    }
}
