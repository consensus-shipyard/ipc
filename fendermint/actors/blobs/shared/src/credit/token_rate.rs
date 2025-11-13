// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::ops::{Div, Mul};

use fvm_shared::{
    bigint::{BigInt, BigUint},
    econ::TokenAmount,
};
use serde::{Deserialize, Serialize};

use super::Credit;

/// TokenCreditRate determines how much atto credits can be bought by a certain amount of RECALL.
#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TokenCreditRate {
    rate: BigUint,
}

impl TokenCreditRate {
    pub const RATIO: u128 = 10u128.pow(18);

    pub fn from(rate: impl Into<BigUint>) -> Self {
        Self { rate: rate.into() }
    }

    pub fn rate(&self) -> &BigUint {
        &self.rate
    }
}

impl std::fmt::Display for TokenCreditRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rate)
    }
}

impl Mul<&TokenCreditRate> for TokenAmount {
    type Output = Credit;

    fn mul(self, rate: &TokenCreditRate) -> Self::Output {
        let rate = BigInt::from(rate.rate.clone());
        (self * rate).div_floor(TokenCreditRate::RATIO)
    }
}

impl Div<&TokenCreditRate> for &Credit {
    type Output = TokenAmount;

    fn div(self, rate: &TokenCreditRate) -> Self::Output {
        #[allow(clippy::suspicious_arithmetic_impl)]
        (self * TokenCreditRate::RATIO).div_floor(rate.rate.clone())
    }
}

impl PartialOrd for TokenCreditRate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TokenCreditRate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rate.cmp(&other.rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_credit_rate() {
        struct TestCase {
            tokens: TokenAmount,
            rate: TokenCreditRate,
            expected: &'static str,
            description: &'static str,
        }

        let test_cases = vec![
            TestCase {
                tokens: TokenAmount::from_whole(1),
                rate: TokenCreditRate::from(1usize),
                expected: "0.000000000000000001",
                description: "lower bound: 1 RECALL buys 1 atto credit",
            },
            TestCase {
                tokens: TokenAmount::from_nano(500000000), // 0.5 RECALL
                rate: TokenCreditRate::from(1usize),
                expected: "0.0",
                description: "crossing lower bound. 0.5 RECALL cannot buy 1 atto credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(1),
                rate: TokenCreditRate::from(2usize),
                expected: "0.000000000000000002",
                description: "1 RECALL buys 2 atto credits",
            },
            TestCase {
                tokens: TokenAmount::from_whole(1),
                rate: TokenCreditRate::from(10u64.pow(18)),
                expected: "1.0",
                description: "1 RECALL buys 1 whole credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(50),
                rate: TokenCreditRate::from(10u64.pow(18)),
                expected: "50.0",
                description: "50 RECALL buys 50 whole credits",
            },
            TestCase {
                tokens: TokenAmount::from_nano(233432100u64),
                rate: TokenCreditRate::from(10u64.pow(18)),
                expected: "0.2334321",
                description: "0.2334321 RECALL buys 0.2334321 credits",
            },
            TestCase {
                tokens: TokenAmount::from_nano(233432100u64),
                rate: TokenCreditRate::from(10u128.pow(36)),
                expected: "233432100000000000.0",
                description: "0.2334321 RECALL buys 233432100000000000 credits",
            },
            TestCase {
                tokens: TokenAmount::from_atto(1), // 1 attoRECALL
                rate: TokenCreditRate::from(10u128.pow(36)),
                expected: "1.0",
                description: "1 atto RECALL buys 1 credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(1),
                rate: TokenCreditRate::from(10u128.pow(18).div(4)),
                expected: "0.25",
                description: "1 RECALL buys 0.25 credit",
            },
            TestCase {
                tokens: TokenAmount::from_whole(1),
                rate: TokenCreditRate::from(10u128.pow(18).div(3)),
                expected: "0.333333333333333333",
                description: "1 RECALL buys 0.333333333333333333 credit",
            },
        ];

        for t in test_cases {
            let credits = t.tokens.clone() * &t.rate;
            assert_eq!(
                t.expected,
                credits.to_string(),
                "tc: {}, {}, {}",
                t.description,
                t.tokens,
                t.rate
            );
        }
    }
}
