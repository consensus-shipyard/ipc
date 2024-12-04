// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::FvmMessage;
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use num_traits::{ToPrimitive, Zero};

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

        if msg.gas_fee_cap >= base_fee {
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
    use crate::fvm::FvmMessage;
    use fvm_shared::address::Address;
    use fvm_shared::bigint::BigInt;
    use fvm_shared::econ::TokenAmount;

    fn create_msg(fee_cap: TokenAmount, premium: TokenAmount) -> FvmMessage {
        FvmMessage {
            version: 0,
            from: Address::new_id(10),
            to: Address::new_id(12),
            sequence: 0,
            value: Default::default(),
            method_num: 0,
            params: Default::default(),
            gas_limit: 0,
            gas_fee_cap: fee_cap,
            gas_premium: premium,
        }
    }

    #[test]
    fn base_fee_update_works() {
        let mut cal = TxnPriorityCalculator::new(3);

        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(0));

        // [10, None, None]
        cal.base_fee_updated(TokenAmount::from_atto(10));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(10));
        assert_eq!(
            cal.base_fee_history,
            vec![Some(TokenAmount::from_atto(10)), None, None]
        );

        // [10, 20, None]
        cal.base_fee_updated(TokenAmount::from_atto(20));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(10));
        assert_eq!(
            cal.base_fee_history,
            vec![
                Some(TokenAmount::from_atto(10)),
                Some(TokenAmount::from_atto(20)),
                None
            ]
        );

        // [10, 20, 5]
        cal.base_fee_updated(TokenAmount::from_atto(5));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(5));
        assert_eq!(
            cal.base_fee_history,
            vec![
                Some(TokenAmount::from_atto(10)),
                Some(TokenAmount::from_atto(20)),
                Some(TokenAmount::from_atto(5)),
            ]
        );

        // [6, 20, 5]
        cal.base_fee_updated(TokenAmount::from_atto(6));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(5));
        assert_eq!(
            cal.base_fee_history,
            vec![
                Some(TokenAmount::from_atto(6)),
                Some(TokenAmount::from_atto(20)),
                Some(TokenAmount::from_atto(5)),
            ]
        );

        // [6, 100, 5]
        cal.base_fee_updated(TokenAmount::from_atto(100));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(5));
        assert_eq!(
            cal.base_fee_history,
            vec![
                Some(TokenAmount::from_atto(6)),
                Some(TokenAmount::from_atto(100)),
                Some(TokenAmount::from_atto(5)),
            ]
        );

        // [6, 100, 10]
        cal.base_fee_updated(TokenAmount::from_atto(10));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(6));
        assert_eq!(
            cal.base_fee_history,
            vec![
                Some(TokenAmount::from_atto(6)),
                Some(TokenAmount::from_atto(100)),
                Some(TokenAmount::from_atto(10)),
            ]
        );

        // [10, 100, 10]
        cal.base_fee_updated(TokenAmount::from_atto(10));
        assert_eq!(cal.lowest_base_fee(), TokenAmount::from_atto(10));
        assert_eq!(
            cal.base_fee_history,
            vec![
                Some(TokenAmount::from_atto(10)),
                Some(TokenAmount::from_atto(100)),
                Some(TokenAmount::from_atto(10)),
            ]
        );
    }

    #[test]
    fn priority_calculation() {
        let mut cal = TxnPriorityCalculator::new(3);

        cal.base_fee_updated(TokenAmount::from_atto(10));
        cal.base_fee_updated(TokenAmount::from_atto(20));
        cal.base_fee_updated(TokenAmount::from_atto(30));

        // lowest base fee is 10

        let msg = create_msg(TokenAmount::from_atto(1), TokenAmount::from_atto(20));
        assert_eq!(cal.priority(&msg), 0);

        let msg = create_msg(TokenAmount::from_atto(10), TokenAmount::from_atto(20));
        assert_eq!(cal.priority(&msg), 20);

        let msg = create_msg(TokenAmount::from_atto(15), TokenAmount::from_atto(20));
        assert_eq!(cal.priority(&msg), 25);

        let msg = create_msg(
            TokenAmount::from_atto(BigInt::from(i64::MAX)),
            TokenAmount::from_atto(BigInt::from(i64::MAX)),
        );
        assert_eq!(cal.priority(&msg), i64::MAX);
    }
}
