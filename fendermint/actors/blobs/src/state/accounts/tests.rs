// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    accounts::AccountStatus,
    blobs::{BlobStatus, SubscriptionId},
    credit::Credit,
};
use fendermint_actor_blobs_testing::{
    new_address, new_hash, new_metadata_hash, new_pk, setup_logs,
};
use fendermint_actor_recall_config_shared::RecallConfig;
use fvm_ipld_blockstore::{Blockstore, MemoryBlockstore};
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use num_traits::Zero;

use crate::{
    caller::DelegationOptions,
    state::blobs::{AddBlobStateParams, FinalizeBlobStateParams},
    testing::check_approval_used,
    State,
};

#[test]
fn test_set_account_status() {
    setup_logs();

    let config = RecallConfig::default();

    struct TestCase {
        name: &'static str,
        initial_ttl_status: Option<AccountStatus>, // None means don't set initial status
        new_ttl_status: AccountStatus,
        expected_ttl: ChainEpoch,
    }

    let test_cases = vec![
        TestCase {
            name: "Setting Reduced on new account",
            initial_ttl_status: None,
            new_ttl_status: AccountStatus::Reduced,
            expected_ttl: 0,
        },
        TestCase {
            name: "Setting Default on new account",
            initial_ttl_status: None,
            new_ttl_status: AccountStatus::Default,
            expected_ttl: config.blob_default_ttl,
        },
        TestCase {
            name: "Changing from Default to Reduced",
            initial_ttl_status: Some(AccountStatus::Default),
            new_ttl_status: AccountStatus::Reduced,
            expected_ttl: 0,
        },
        TestCase {
            name: "Changing from Extended to Reduced",
            initial_ttl_status: Some(AccountStatus::Extended),
            new_ttl_status: AccountStatus::Reduced,
            expected_ttl: 0,
        },
        TestCase {
            name: "Changing from Reduced to Extended",
            initial_ttl_status: Some(AccountStatus::Reduced),
            new_ttl_status: AccountStatus::Extended,
            expected_ttl: ChainEpoch::MAX,
        },
    ];

    for tc in test_cases {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let address = new_address();
        let current_epoch = ChainEpoch::from(1);

        // Initialize the account if needed
        if tc.initial_ttl_status.is_some() {
            state
                .set_account_status(
                    &store,
                    &config,
                    address,
                    tc.initial_ttl_status.unwrap(),
                    current_epoch,
                )
                .unwrap();
        }

        // Change TTL status
        let res =
            state.set_account_status(&store, &config, address, tc.new_ttl_status, current_epoch);
        assert!(
            res.is_ok(),
            "Test case '{}' failed to set TTL status",
            tc.name
        );

        // Verify max TTL
        let max_ttl = state.get_account_max_ttl(&config, &store, address).unwrap();
        assert_eq!(
            max_ttl, tc.expected_ttl,
            "Test case '{}' failed: expected max TTL {}, got {}",
            tc.name, tc.expected_ttl, max_ttl
        );
    }
}

#[test]
fn test_debit_accounts_delete_from_disc() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let current_epoch = ChainEpoch::from(1);
    let token_amount = TokenAmount::from_whole(10);
    state
        .buy_credit(&store, &config, caller, token_amount.clone(), current_epoch)
        .unwrap();
    debit_accounts_delete_from_disc(
        &config,
        &store,
        state,
        caller,
        None,
        current_epoch,
        token_amount,
        false,
    );
}

#[test]
fn test_debit_accounts_delete_from_disc_with_approval() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let sponsor = new_address();
    let current_epoch = ChainEpoch::from(1);
    let token_amount = TokenAmount::from_whole(10);
    state
        .buy_credit(
            &store,
            &config,
            sponsor,
            token_amount.clone(),
            current_epoch,
        )
        .unwrap();
    state
        .approve_credit(
            &config,
            &store,
            sponsor,
            caller,
            DelegationOptions::default(),
            current_epoch,
        )
        .unwrap();
    debit_accounts_delete_from_disc(
        &config,
        &store,
        state,
        caller,
        Some(sponsor),
        current_epoch,
        token_amount,
        true,
    );
}

#[allow(clippy::too_many_arguments)]
fn debit_accounts_delete_from_disc<BS: Blockstore>(
    config: &RecallConfig,
    store: &BS,
    mut state: State,
    caller: Address,
    sponsor: Option<Address>,
    current_epoch: ChainEpoch,
    token_amount: TokenAmount,
    using_approval: bool,
) {
    let subscriber = sponsor.unwrap_or(caller);
    let mut credit_amount =
        Credit::from_atto(token_amount.atto().clone()) * &config.token_credit_rate;

    // Add blob with default a subscription ID
    let (hash, size) = new_hash(1024);
    let add1_epoch = current_epoch;
    let id1 = SubscriptionId::default();
    let ttl1 = ChainEpoch::from(config.blob_min_ttl);
    let source = new_pk();
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: id1.clone(),
            size,
            ttl: Some(ttl1),
            source,
            epoch: add1_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    let stats = state.get_stats(config, TokenAmount::zero());
    // Using a credit delegation creates both the from and to account
    let expected_num_accounts = if using_approval { 2 } else { 1 };
    assert_eq!(stats.num_accounts, expected_num_accounts);
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 1);
    assert_eq!(stats.bytes_added, size);

    // Set to status pending
    let res = state.set_blob_pending(&store, subscriber, hash, size, id1.clone(), source);
    assert!(res.is_ok());
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 1);
    assert_eq!(stats.bytes_resolving, size);
    assert_eq!(stats.num_added, 0);
    assert_eq!(stats.bytes_added, 0);

    // Finalize as resolved
    let finalize_epoch = ChainEpoch::from(11);
    let res = state.finalize_blob(
        &store,
        subscriber,
        FinalizeBlobStateParams {
            hash,
            id: id1.clone(),
            status: BlobStatus::Resolved,
            epoch: finalize_epoch,
        },
    );
    assert!(res.is_ok());
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 0);
    assert_eq!(stats.bytes_added, 0);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add1_epoch);
    assert_eq!(
        account.credit_committed,
        Credit::from_whole(ttl1 as u64 * size)
    );
    credit_amount -= &account.credit_committed;
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size);

    // Add the same blob but this time uses a different subscription ID
    let add2_epoch = ChainEpoch::from(21);
    let ttl2 = ChainEpoch::from(config.blob_min_ttl);
    let id2 = SubscriptionId::new("foo").unwrap();
    let source = new_pk();
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: id2.clone(),
            size,
            ttl: Some(ttl2),
            source,
            epoch: add2_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 0);
    assert_eq!(stats.bytes_added, 0);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add2_epoch);
    assert_eq!(
        account.credit_committed, // stays the same becuase we're starting over
        Credit::from_whole(ttl2 as u64 * size),
    );
    credit_amount -= Credit::from_whole((add2_epoch - add1_epoch) as u64 * size);
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size); // not changed

    // Check the subscription group
    let blob = state.get_blob(&store, hash).unwrap().unwrap();
    let subscribers = blob.subscribers.hamt(store).unwrap();
    let group = subscribers.get(&subscriber).unwrap().unwrap();
    assert_eq!(group.len(), 2);

    // Debit all accounts at an epoch between the two expiries (3601-3621)
    let debit_epoch = ChainEpoch::from(config.blob_min_ttl + 11);
    let (deletes_from_disc, _) = state.debit_accounts(&store, config, debit_epoch).unwrap();
    assert!(deletes_from_disc.is_empty());

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, debit_epoch);
    assert_eq!(
        account.credit_committed, // debit reduces this
        Credit::from_whole((ttl2 - (debit_epoch - add2_epoch)) as u64 * size),
    );
    assert_eq!(account.credit_free, credit_amount); // not changed
    assert_eq!(account.capacity_used, size); // not changed

    // Check the subscription group
    let blob = state.get_blob(&store, hash).unwrap().unwrap();
    let subscribers = blob.subscribers.hamt(&store).unwrap();
    let group = subscribers.get(&subscriber).unwrap().unwrap();
    assert_eq!(group.len(), 1); // the first subscription was deleted

    // Debit all accounts at an epoch greater than group expiry (3621)
    let debit_epoch = ChainEpoch::from(config.blob_min_ttl + 31);
    let (deletes_from_disc, _) = state.debit_accounts(&store, config, debit_epoch).unwrap();
    assert!(!deletes_from_disc.is_empty()); // blob is marked for deletion

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, debit_epoch);
    assert_eq!(
        account.credit_committed, // the second debit reduces this to zero
        Credit::from_whole(0),
    );
    assert_eq!(account.credit_free, credit_amount); // not changed
    assert_eq!(account.capacity_used, 0);

    // Check state
    assert_eq!(state.credits.credit_committed, Credit::from_whole(0)); // credit was released
    assert_eq!(
        state.credits.credit_debited,
        token_amount * &config.token_credit_rate - &account.credit_free
    );
    assert_eq!(state.blobs.bytes_size(), 0); // capacity was released

    // Check indexes
    assert_eq!(state.blobs.expiries.len(store).unwrap(), 0);
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);

    // Check approval
    if using_approval {
        check_approval_used(&state, store, caller, subscriber);
    }
}

#[test]
fn test_paginated_debit_accounts() {
    let config = RecallConfig {
        account_debit_batch_size: 5, // Process 5 accounts at a time (10 accounts total)
        ..Default::default()
    };

    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let current_epoch = ChainEpoch::from(1);

    // Create more than one batch worth of accounts (>5)
    for i in 0..10 {
        let address = Address::new_id(1000 + i);
        let token_amount = TokenAmount::from_whole(10);

        // Buy credits for each account
        state
            .buy_credit(
                &store,
                &config,
                address,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();

        // Add some storage usage
        let mut accounts = state.accounts.hamt(&store).unwrap();
        let mut account = accounts.get(&address).unwrap().unwrap();
        account.capacity_used = 1000;
        accounts.set(&address, account).unwrap();
    }

    // First batch (should process 5 accounts)
    assert!(state.accounts.get_debit_start_address().is_none());
    let (deletes1, _) = state
        .debit_accounts(&store, &config, current_epoch + 1)
        .unwrap();
    assert!(deletes1.is_empty()); // No expired blobs
    assert!(state.accounts.get_debit_start_address().is_some());

    // Second batch (should process remaining 5 accounts and clear state)
    let (deletes2, _) = state
        .debit_accounts(&store, &config, current_epoch + 1)
        .unwrap();
    assert!(deletes2.is_empty());
    assert!(state.accounts.get_debit_start_address().is_none()); // The state should be cleared after all accounts processed

    // Verify all accounts were processed
    let reader = state.accounts.hamt(&store).unwrap();
    reader
        .for_each(|_, account| {
            assert_eq!(account.last_debit_epoch, current_epoch + 1);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_multiple_debit_cycles() {
    let config = RecallConfig {
        account_debit_batch_size: 5, // Process 5 accounts at a time (10 accounts total)
        ..Default::default()
    };

    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let current_epoch = ChainEpoch::from(1);

    // Create accounts
    for i in 0..10 {
        let address = Address::new_id(1000 + i);
        let token_amount = TokenAmount::from_whole(10);
        state
            .buy_credit(
                &store,
                &config,
                address,
                token_amount.clone(),
                current_epoch,
            )
            .unwrap();

        let mut accounts = state.accounts.hamt(&store).unwrap();
        let mut account = accounts.get(&address).unwrap().unwrap();
        account.capacity_used = 1000;
        accounts.set(&address, account).unwrap();
    }

    // First cycle
    let (deletes1, _) = state
        .debit_accounts(&store, &config, current_epoch + 1)
        .unwrap();
    assert!(deletes1.is_empty());
    assert!(state.accounts.get_debit_start_address().is_some());

    let (deletes2, _) = state
        .debit_accounts(&store, &config, current_epoch + 1)
        .unwrap();
    assert!(deletes2.is_empty());
    assert!(state.accounts.get_debit_start_address().is_none()); // First cycle complete

    // Second cycle
    let (deletes3, _) = state
        .debit_accounts(&store, &config, current_epoch + 2)
        .unwrap();
    assert!(deletes3.is_empty());
    assert!(state.accounts.get_debit_start_address().is_some());

    let (deletes4, _) = state
        .debit_accounts(&store, &config, current_epoch + 2)
        .unwrap();
    assert!(deletes4.is_empty());
    assert!(state.accounts.get_debit_start_address().is_none()); // Second cycle complete
}
