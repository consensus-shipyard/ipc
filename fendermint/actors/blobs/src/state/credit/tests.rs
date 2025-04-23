// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    blobs::SubscriptionId,
    credit::{Credit, CreditApproval},
};
use fendermint_actor_blobs_testing::{
    new_address, new_hash, new_metadata_hash, new_pk, setup_logs,
};
use fendermint_actor_recall_config_shared::RecallConfig;
use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use num_traits::Zero;

use crate::{caller::DelegationOptions, state::blobs::AddBlobStateParams, State};

fn check_approvals_match(
    state: &State,
    store: &MemoryBlockstore,
    from: Address,
    to: Address,
    expected: CreditApproval,
) {
    let from_account = state.get_account(&store, from).unwrap().unwrap();
    assert_eq!(
        from_account
            .approvals_to
            .hamt(store)
            .unwrap()
            .get(&to)
            .unwrap()
            .unwrap(),
        expected
    );
    let to_account = state.get_account(&store, to).unwrap().unwrap();
    assert_eq!(
        to_account
            .approvals_from
            .hamt(store)
            .unwrap()
            .get(&from)
            .unwrap()
            .unwrap(),
        expected
    );
}

#[test]
fn test_buy_credit_success() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let to = new_address();
    let amount = TokenAmount::from_whole(1);

    let res = state.buy_credit(&store, &config, to, amount.clone(), 1);
    assert!(res.is_ok());
    let account = res.unwrap();
    let credit_sold = amount.clone() * &config.token_credit_rate;
    assert_eq!(account.credit_free, credit_sold);
    assert_eq!(account.gas_allowance, amount);
    assert_eq!(state.credits.credit_sold, credit_sold);
    let account_back = state.get_account(&store, to).unwrap().unwrap();
    assert_eq!(account, account_back);
}

#[test]
fn test_buy_credit_negative_amount() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let to = new_address();
    let amount = TokenAmount::from_whole(-1);

    let res = state.buy_credit(&store, &config, to, amount, 1);
    assert!(res.is_err());
    assert_eq!(res.err().unwrap().msg(), "amount must be positive");
}

#[test]
fn test_buy_credit_at_capacity() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let to = new_address();
    let amount = TokenAmount::from_whole(1);

    state.blobs.set_capacity(config.blob_capacity);
    let res = state.buy_credit(&store, &config, to, amount, 1);
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().msg(),
        "subnet has reached storage capacity"
    );
}

#[test]
fn test_approve_credit_success() {
    setup_logs();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let from = new_address();
    let to = new_address();
    let current_epoch = 1;

    let config = RecallConfig::default();

    // No limit or expiry
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions::default(),
        current_epoch,
    );
    assert!(res.is_ok());
    let approval = res.unwrap();
    assert_eq!(approval.credit_limit, None);
    assert_eq!(approval.gas_allowance_limit, None);
    assert_eq!(approval.expiry, None);
    check_approvals_match(&state, &store, from, to, approval);

    // Add credit limit
    let limit = 1_000_000_000_000_000_000u64;
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions {
            credit_limit: Some(Credit::from_whole(limit)),
            ..Default::default()
        },
        current_epoch,
    );
    assert!(res.is_ok());
    let approval = res.unwrap();
    assert_eq!(approval.credit_limit, Some(Credit::from_whole(limit)));
    assert_eq!(approval.gas_allowance_limit, None);
    assert_eq!(approval.expiry, None);
    check_approvals_match(&state, &store, from, to, approval);

    // Add gas fee limit
    let limit = 1_000_000_000_000_000_000u64;
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions {
            gas_fee_limit: Some(TokenAmount::from_atto(limit)),
            ..Default::default()
        },
        current_epoch,
    );
    assert!(res.is_ok());
    let approval = res.unwrap();
    assert_eq!(approval.credit_limit, None);
    assert_eq!(
        approval.gas_allowance_limit,
        Some(TokenAmount::from_atto(limit))
    );
    assert_eq!(approval.expiry, None);
    check_approvals_match(&state, &store, from, to, approval);

    // Add ttl
    let ttl = ChainEpoch::from(config.blob_min_ttl);
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions {
            credit_limit: Some(Credit::from_whole(limit)),
            ttl: Some(ttl),
            ..Default::default()
        },
        current_epoch,
    );
    assert!(res.is_ok());
    let approval = res.unwrap();
    assert_eq!(approval.credit_limit, Some(Credit::from_whole(limit)));
    assert_eq!(approval.gas_allowance_limit, None);
    assert_eq!(approval.expiry, Some(ttl + current_epoch));
    check_approvals_match(&state, &store, from, to, approval);
}

#[test]
fn test_approve_credit_invalid_ttl() {
    setup_logs();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let from = new_address();
    let to = new_address();
    let current_epoch = 1;

    let config = RecallConfig::default();
    let ttl = ChainEpoch::from(config.blob_min_ttl - 1);
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions {
            ttl: Some(ttl),
            ..Default::default()
        },
        current_epoch,
    );
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().msg(),
        format!("minimum approval TTL is {}", config.blob_min_ttl)
    );
}

#[test]
fn test_approve_credit_overflowing_ttl() {
    setup_logs();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let from = new_address();
    let to = new_address();
    let current_epoch = 1;

    let config = RecallConfig::default();

    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions {
            ttl: Some(ChainEpoch::MAX),
            ..Default::default()
        },
        current_epoch,
    );
    assert!(res.is_ok());
    let approval = res.unwrap();
    assert_eq!(approval.expiry, Some(i64::MAX));
}

#[test]
fn test_approve_credit_insufficient_credit() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let from = new_address();
    let to = new_address();
    let current_epoch = 1;

    let amount = TokenAmount::from_whole(10);
    state
        .buy_credit(&store, &config, from, amount.clone(), current_epoch)
        .unwrap();
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions::default(),
        current_epoch,
    );
    assert!(res.is_ok());

    // Add a blob
    let (hash, size) = new_hash(1024);
    let res = state.add_blob(
        &store,
        &config,
        to,
        Some(from),
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size,
            ttl: None,
            source: new_pk(),
            epoch: current_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Check approval
    let account = state.get_account(&store, from).unwrap().unwrap();
    let approval = account
        .approvals_to
        .hamt(&store)
        .unwrap()
        .get(&to)
        .unwrap()
        .unwrap();
    assert_eq!(account.credit_committed, approval.credit_used);

    // Try to update approval with a limit below what's already been committed
    let limit = 1_000u64;
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions {
            credit_limit: Some(Credit::from_whole(limit)),
            ..Default::default()
        },
        current_epoch,
    );
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().msg(),
        format!(
            "limit cannot be less than amount of already used credits ({})",
            approval.credit_used
        )
    );
}

#[test]
fn test_revoke_credit_success() {
    setup_logs();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let from = new_address();
    let to = new_address();
    let current_epoch = 1;

    let config = RecallConfig::default();
    let res = state.approve_credit(
        &config,
        &store,
        from,
        to,
        DelegationOptions::default(),
        current_epoch,
    );
    assert!(res.is_ok());

    // Check the account approvals
    let from_account = state.get_account(&store, from).unwrap().unwrap();
    assert_eq!(from_account.approvals_to.len(), 1);
    let to_account = state.get_account(&store, to).unwrap().unwrap();
    assert_eq!(to_account.approvals_from.len(), 1);

    // Remove the approval
    let res = state.revoke_credit(&store, from, to);
    assert!(res.is_ok());
    let from_account = state.get_account(&store, from).unwrap().unwrap();
    assert_eq!(from_account.approvals_to.len(), 0);
    let to_account = state.get_account(&store, to).unwrap().unwrap();
    assert_eq!(to_account.approvals_from.len(), 0);
}

#[test]
fn test_revoke_credit_account_not_found() {
    setup_logs();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let from = new_address();
    let to = new_address();

    let res = state.revoke_credit(&store, from, to);
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().msg(),
        format!("{} not found in accounts", to)
    );
}
