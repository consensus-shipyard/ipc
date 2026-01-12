// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address};
use crate::State;

pub fn check_approval_used<BS: Blockstore>(
    state: &State,
    store: &BS,
    caller: Address,
    sponsor: Address,
) {
    assert_ne!(caller, sponsor);
    let subscriber_account = state.get_account(&store, sponsor).unwrap().unwrap();
    let subscriber_approval = subscriber_account
        .approvals_to
        .hamt(store)
        .unwrap()
        .get(&caller)
        .unwrap()
        .unwrap();
    assert_eq!(
        subscriber_approval.credit_used,
        state.credits.credit_debited.clone() + subscriber_account.credit_committed.clone()
    );
    let origin_account = state.get_account(&store, caller).unwrap().unwrap();
    let origin_approval = origin_account
        .approvals_from
        .hamt(store)
        .unwrap()
        .get(&sponsor)
        .unwrap()
        .unwrap();
    assert_eq!(
        subscriber_approval.credit_used,
        &state.credits.credit_debited + &subscriber_account.credit_committed
    );
    assert_eq!(subscriber_approval.credit_used, origin_approval.credit_used);
}
