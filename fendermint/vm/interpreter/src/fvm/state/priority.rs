// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::FvmMessage;
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use lazy_static::lazy_static;
use num_traits::{ToPrimitive, Zero};
use std::str::FromStr;

lazy_static! {
    // Max U256
    static ref MAX_GAS: TokenAmount = TokenAmount::from_atto(BigInt::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap());
}

/// The transaction priority calculator. The priority calculated is used to determine the ordering
/// in the mempool.
#[derive(Clone, Debug)]
pub struct TxnPriorityCalculator {
    /// Ring buffer of base fee history
    base_fee_history: Vec<Option<TokenAmount>>,
    /// Next slot in the ring buffer
    next_slot: usize,
}

impl TxnPriorityCalculator {
    pub fn new(size: usize) -> Self {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(None);
        }
        Self {
            base_fee_history: v,
            next_slot: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.base_fee_history.len()
    }

    pub fn base_fee_updated(&mut self, base_fee: TokenAmount) {
        self.base_fee_history[self.next_slot] = Some(base_fee);
        self.next_slot = (self.next_slot + 1) % self.base_fee_history.len();
    }

    pub fn priority(&self, msg: &FvmMessage) -> i64 {
        let base_fee = self.lowest_base_fee();

        if msg.gas_fee_cap > base_fee {
            let i = msg.gas_fee_cap.clone() - base_fee + msg.gas_premium.clone();
            i.atto()
                .min(&BigInt::from(i64::MAX))
                .to_i64()
                .expect("clipped to i64 max")
        } else {
            0
        }
    }

    fn lowest_base_fee(&self) -> TokenAmount {
        let mut out: Option<TokenAmount> = None;
        for v in &self.base_fee_history {
            let Some(v) = v.as_ref() else { continue };

            match out {
                Some(min) => out = Some(min.min(v.clone())),
                None => out = Some(v.clone()),
            }
        }

        out.unwrap_or(TokenAmount::zero())
    }
}

#[cfg(test)]
mod tests {
    use crate::fvm::state::priority::TxnPriorityCalculator;
    use fvm_shared::econ::TokenAmount;

    #[test]
    fn base_fee_update_works() {
        let mut cal = TxnPriorityCalculator::new(3);

        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(0));

        cal.base_fee_updated(TokenAmount::from_atto(10));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(10));

        cal.base_fee_updated(TokenAmount::from_atto(20));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(10));

        cal.base_fee_updated(TokenAmount::from_atto(5));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(5));

        cal.base_fee_updated(TokenAmount::from_atto(6));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(5));

        cal.base_fee_updated(TokenAmount::from_atto(100));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(5));

        cal.base_fee_updated(TokenAmount::from_atto(10));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(6));

        cal.base_fee_updated(TokenAmount::from_atto(10));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(10));
    }
}
