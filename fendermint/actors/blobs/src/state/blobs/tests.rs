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
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::{Blockstore, MemoryBlockstore};
use fvm_shared::{address::Address, bigint::BigInt, clock::ChainEpoch, econ::TokenAmount};
use num_traits::Zero;

use super::{AddBlobStateParams, DeleteBlobStateParams, FinalizeBlobStateParams};
use crate::{caller::DelegationOptions, testing::check_approval_used, State};

#[test]
fn test_add_blob_refund() {
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
    add_blob_refund(
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
fn test_add_blob_refund_with_approval() {
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
    add_blob_refund(
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
fn add_blob_refund<BS: Blockstore>(
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
    let token_credit_rate = BigInt::from(1_000_000_000_000_000_000u64);
    let mut credit_amount = token_amount.clone() * &config.token_credit_rate;

    // Add blob with default a subscription ID
    let (hash1, size1) = new_hash(1024);
    let add1_epoch = current_epoch;
    let id1 = SubscriptionId::default();
    let source = new_pk();
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash: hash1,
            metadata_hash: new_metadata_hash(),
            id: id1.clone(),
            size: size1,
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add1_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 1);
    assert_eq!(stats.bytes_added, size1);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add1_epoch);
    assert_eq!(
        account.credit_committed,
        Credit::from_whole(config.blob_min_ttl as u64 * size1),
    );
    credit_amount -= &account.credit_committed;
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size1);

    assert!(state
        .set_account_status(
            &store,
            config,
            subscriber,
            AccountStatus::Extended,
            current_epoch
        )
        .is_ok());

    // Add another blob past the first blob's expiry
    let (hash2, size2) = new_hash(2048);
    let add2_epoch = ChainEpoch::from(config.blob_min_ttl + 11);
    let id2 = SubscriptionId::new("foo").unwrap();
    let source = new_pk();
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash: hash2,
            metadata_hash: new_metadata_hash(),
            id: id2.clone(),
            size: size2,
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add2_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 2);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 2);
    assert_eq!(stats.bytes_added, size1 + size2);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add2_epoch);
    let blob1_expiry = ChainEpoch::from(config.blob_min_ttl + add1_epoch);
    let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
    assert_eq!(
        account.credit_committed, // this includes an overcharge that needs to be refunded
        Credit::from_whole(config.blob_min_ttl as u64 * size2 - overcharge),
    );
    credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size2);
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size1 + size2);

    // Check state
    assert_eq!(state.credits.credit_committed, account.credit_committed);
    assert_eq!(
        state.credits.credit_debited,
        (token_amount.clone() * &token_credit_rate)
            - (&account.credit_free + &account.credit_committed)
    );
    assert_eq!(state.blobs.bytes_size(), account.capacity_used);

    // Check indexes
    assert_eq!(state.blobs.expiries.len(store).unwrap(), 2);
    assert_eq!(state.blobs.added.len(), 2);
    assert_eq!(state.blobs.pending.len(), 0);

    // Add the first (now expired) blob again
    let add3_epoch = ChainEpoch::from(config.blob_min_ttl + 21);
    let id1 = SubscriptionId::default();
    let source = new_pk();
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash: hash1,
            metadata_hash: new_metadata_hash(),
            id: id1.clone(),
            size: size1,
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add3_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 2);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 2);
    assert_eq!(stats.bytes_added, size1 + size2);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add3_epoch);
    assert_eq!(
        account.credit_committed, // should not include overcharge due to refund
        Credit::from_whole(
            (config.blob_min_ttl - (add3_epoch - add2_epoch)) as u64 * size2
                + config.blob_min_ttl as u64 * size1
        ),
    );
    credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size1);
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size1 + size2);

    // Check state
    assert_eq!(state.credits.credit_committed, account.credit_committed);
    assert_eq!(
        state.credits.credit_debited,
        token_amount.clone() * &token_credit_rate
            - (&account.credit_free + &account.credit_committed)
    );
    assert_eq!(state.blobs.bytes_size(), account.capacity_used);

    // Check indexes
    assert_eq!(state.blobs.expiries.len(store).unwrap(), 2);
    assert_eq!(state.blobs.added.len(), 2);
    assert_eq!(state.blobs.pending.len(), 0);

    // Check approval
    if using_approval {
        check_approval_used(&state, store, caller, subscriber);
    }
}

#[test]
fn test_add_blob_same_hash_same_account() {
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
    add_blob_same_hash_same_account(
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
fn test_add_blob_same_hash_same_account_with_approval() {
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
    add_blob_same_hash_same_account(
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
fn add_blob_same_hash_same_account<BS: Blockstore>(
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

    assert!(state
        .set_account_status(
            &store,
            config,
            subscriber,
            AccountStatus::Extended,
            current_epoch
        )
        .is_ok());

    // Add blob with a default subscription ID
    let (hash, size) = new_hash(1024);
    let add1_epoch = current_epoch;
    let id1 = SubscriptionId::default();
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
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add1_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());
    let (sub, _) = res.unwrap();
    assert_eq!(sub.added, add1_epoch);
    assert_eq!(sub.expiry, add1_epoch + config.blob_min_ttl);
    assert_eq!(sub.source, source);
    assert!(!sub.failed);
    if subscriber != caller {
        assert_eq!(sub.delegate, Some(caller));
    }

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 1);
    assert_eq!(stats.bytes_added, size);

    // Check the blob status
    assert_eq!(
        state
            .get_blob_status(&store, subscriber, hash, id1.clone())
            .unwrap(),
        Some(BlobStatus::Added)
    );

    // Check the blob
    let blob = state.get_blob(&store, hash).unwrap().unwrap();
    let subscribers = blob.subscribers.hamt(store).unwrap();
    assert_eq!(blob.subscribers.len(), 1);
    assert_eq!(blob.status, BlobStatus::Added);
    assert_eq!(blob.size, size);

    // Check the subscription group
    let group = subscribers.get(&subscriber).unwrap().unwrap();
    let group_hamt = group.hamt(store).unwrap();
    assert_eq!(group.len(), 1);
    let got_sub = group_hamt.get(&id1.clone()).unwrap().unwrap();
    assert_eq!(got_sub, sub);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add1_epoch);
    assert_eq!(
        account.credit_committed,
        Credit::from_whole(config.blob_min_ttl as u64 * size),
    );
    credit_amount -= &account.credit_committed;
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size);

    // Set to status pending
    let res = state.set_blob_pending(&store, subscriber, hash, size, id1.clone(), source);
    assert!(res.is_ok());

    // Check stats
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
    assert_eq!(
        state
            .get_blob_status(&store, subscriber, hash, id1.clone())
            .unwrap(),
        Some(BlobStatus::Resolved)
    );

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 0);
    assert_eq!(stats.bytes_added, 0);

    // Add the same blob again with a default subscription ID
    let add2_epoch = ChainEpoch::from(21);
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
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add2_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());
    let (sub, _) = res.unwrap();
    assert_eq!(sub.added, add1_epoch); // added should not change
    assert_eq!(sub.expiry, add2_epoch + config.blob_min_ttl);
    assert_eq!(sub.source, source);
    assert!(!sub.failed);
    if subscriber != caller {
        assert_eq!(sub.delegate, Some(caller));
    }

    // Check the blob status
    // Should already be resolved
    assert_eq!(
        state
            .get_blob_status(&store, subscriber, hash, id1.clone())
            .unwrap(),
        Some(BlobStatus::Resolved)
    );

    // Check the blob
    let blob = state.get_blob(&store, hash).unwrap().unwrap();
    let subscribers = blob.subscribers.hamt(store).unwrap();
    assert_eq!(blob.subscribers.len(), 1);
    assert_eq!(blob.status, BlobStatus::Resolved);
    assert_eq!(blob.size, size);

    // Check the subscription group
    let group = subscribers.get(&subscriber).unwrap().unwrap();
    let group_hamt = group.hamt(store).unwrap();
    assert_eq!(group.len(), 1); // Still only one subscription
    let got_sub = group_hamt.get(&id1.clone()).unwrap().unwrap();
    assert_eq!(got_sub, sub);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add2_epoch);
    assert_eq!(
        account.credit_committed, // stays the same becuase we're starting over
        Credit::from_whole(config.blob_min_ttl as u64 * size),
    );
    credit_amount -= Credit::from_whole((add2_epoch - add1_epoch) as u64 * size);
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size); // not changed

    assert_eq!(state.blobs.expiries.len(store).unwrap(), 1);
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);

    // Add the same blob again but use a different subscription ID
    let add3_epoch = ChainEpoch::from(31);
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
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add3_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());
    let (sub, _) = res.unwrap();
    assert_eq!(sub.added, add3_epoch);
    assert_eq!(sub.expiry, add3_epoch + config.blob_min_ttl);
    assert_eq!(sub.source, source);
    assert!(!sub.failed);
    if subscriber != caller {
        assert_eq!(sub.delegate, Some(caller));
    }

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 0);
    assert_eq!(stats.bytes_added, 0);

    // Check the blob status
    // Should already be resolved
    assert_eq!(
        state
            .get_blob_status(&store, subscriber, hash, id2.clone())
            .unwrap(),
        Some(BlobStatus::Resolved)
    );

    // Check the blob
    let blob = state.get_blob(&store, hash).unwrap().unwrap();
    let subscribers = blob.subscribers.hamt(store).unwrap();
    assert_eq!(blob.subscribers.len(), 1); // still only one subscriber
    assert_eq!(blob.status, BlobStatus::Resolved);
    assert_eq!(blob.size, size);

    // Check the subscription group
    let group = subscribers.get(&subscriber).unwrap().unwrap();
    let group_hamt = group.hamt(store).unwrap();
    assert_eq!(group.len(), 2);
    let got_sub = group_hamt.get(&id2.clone()).unwrap().unwrap();
    assert_eq!(got_sub, sub);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add3_epoch);
    assert_eq!(
        account.credit_committed, // stays the same becuase we're starting over
        Credit::from_whole(config.blob_min_ttl as u64 * size),
    );
    credit_amount -= Credit::from_whole((add3_epoch - add2_epoch) as u64 * size);
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size); // not changed

    // Debit all accounts
    let debit_epoch = ChainEpoch::from(41);
    let (deletes_from_disc, _) = state.debit_accounts(&store, config, debit_epoch).unwrap();
    assert!(deletes_from_disc.is_empty());

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, debit_epoch);
    assert_eq!(
        account.credit_committed, // debit reduces this
        Credit::from_whole((config.blob_min_ttl - (debit_epoch - add3_epoch)) as u64 * size),
    );
    assert_eq!(account.credit_free, credit_amount); // not changed
    assert_eq!(account.capacity_used, size); // not changed

    // Check indexes
    assert_eq!(state.blobs.expiries.len(store).unwrap(), 2);
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);

    // Delete the default subscription ID
    let delete_epoch = ChainEpoch::from(51);
    let res = state.delete_blob(
        &store,
        caller,
        sponsor,
        DeleteBlobStateParams {
            hash,
            id: id1.clone(),
            epoch: delete_epoch,
        },
    );

    assert!(res.is_ok());
    let (delete_from_disk, deleted_size) = res.unwrap();
    assert!(!delete_from_disk);
    assert_eq!(deleted_size, size);

    // Check the blob
    let blob = state.get_blob(&store, hash).unwrap().unwrap();
    let subscribers = blob.subscribers.hamt(store).unwrap();

    assert_eq!(blob.subscribers.len(), 1); // still one subscriber
    assert_eq!(blob.status, BlobStatus::Resolved);
    assert_eq!(blob.size, size);

    // Check the subscription group
    let group = subscribers.get(&subscriber).unwrap().unwrap();
    let group_hamt = group.hamt(store).unwrap();
    assert_eq!(group.len(), 1);
    let sub = group_hamt.get(&id2.clone()).unwrap().unwrap();
    assert_eq!(sub.added, add3_epoch);
    assert_eq!(sub.expiry, add3_epoch + config.blob_min_ttl);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, delete_epoch);
    assert_eq!(
        account.credit_committed, // debit reduces this
        Credit::from_whole((config.blob_min_ttl - (delete_epoch - add3_epoch)) as u64 * size),
    );
    assert_eq!(account.credit_free, credit_amount); // not changed
    assert_eq!(account.capacity_used, size); // not changed

    // Check state
    assert_eq!(state.credits.credit_committed, account.credit_committed);
    assert_eq!(
        state.credits.credit_debited,
        (token_amount.clone() * &config.token_credit_rate)
            - (&account.credit_free + &account.credit_committed)
    );
    assert_eq!(state.blobs.bytes_size(), size);

    // Check indexes
    assert_eq!(state.blobs.expiries.len(store).unwrap(), 1);
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);

    // Check approval
    if using_approval {
        check_approval_used(&state, store, caller, subscriber);
    }
}

#[test]
fn test_add_blob_ttl_exceeds_account_max_ttl() {
    setup_logs();

    let config = RecallConfig::default();
    const YEAR: ChainEpoch = 365 * 24 * 60 * 60;

    // Test cases structure
    struct TestCase {
        name: &'static str,
        account_ttl_status: AccountStatus,
        blob_ttl: Option<ChainEpoch>,
        should_succeed: bool,
        expected_account_ttl: ChainEpoch,
        expected_blob_ttl: ChainEpoch,
    }

    // Define test cases
    let test_cases = vec![
        TestCase {
            name: "Reduced status rejects even minimum TTL",
            account_ttl_status: AccountStatus::Reduced,
            blob_ttl: Some(config.blob_min_ttl),
            should_succeed: false,
            expected_account_ttl: 0,
            expected_blob_ttl: 0,
        },
        TestCase {
            name: "Reduced status rejects no TTL",
            account_ttl_status: AccountStatus::Reduced,
            blob_ttl: Some(config.blob_min_ttl),
            should_succeed: false,
            expected_account_ttl: 0,
            expected_blob_ttl: 0,
        },
        TestCase {
            name: "Default status allows default TTL",
            account_ttl_status: AccountStatus::Default,
            blob_ttl: Some(config.blob_default_ttl),
            should_succeed: true,
            expected_account_ttl: config.blob_default_ttl,
            expected_blob_ttl: config.blob_default_ttl,
        },
        TestCase {
            name: "Default status sets no TTL to default without auto renew",
            account_ttl_status: AccountStatus::Default,
            blob_ttl: None,
            should_succeed: true,
            expected_account_ttl: config.blob_default_ttl,
            expected_blob_ttl: config.blob_default_ttl,
        },
        TestCase {
            name: "Default status preserves given TTL if it's less than default",
            account_ttl_status: AccountStatus::Default,
            blob_ttl: Some(config.blob_default_ttl - 1),
            should_succeed: true,
            expected_account_ttl: config.blob_default_ttl,
            expected_blob_ttl: config.blob_default_ttl - 1,
        },
        TestCase {
            name: "Default status rejects TTLs higher than default",
            account_ttl_status: AccountStatus::Default,
            blob_ttl: Some(config.blob_default_ttl + 1),
            should_succeed: false,
            expected_account_ttl: config.blob_default_ttl,
            expected_blob_ttl: 0,
        },
        TestCase {
            name: "Extended status allows any TTL",
            account_ttl_status: AccountStatus::Extended,
            blob_ttl: Some(YEAR),
            should_succeed: true,
            expected_account_ttl: ChainEpoch::MAX,
            expected_blob_ttl: YEAR,
        },
    ];

    // Run all test cases
    for tc in test_cases {
        let config = RecallConfig::default();
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);
        let amount = TokenAmount::from_whole(10);

        state
            .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
            .unwrap();
        state
            .set_account_status(
                &store,
                &config,
                caller,
                tc.account_ttl_status,
                current_epoch,
            )
            .unwrap();

        let (hash, size) = new_hash(1024);
        let res = state.add_blob(
            &store,
            &config,
            caller,
            None,
            AddBlobStateParams {
                hash,
                metadata_hash: new_metadata_hash(),
                id: SubscriptionId::default(),
                size,
                ttl: tc.blob_ttl,
                source: new_pk(),
                epoch: current_epoch,
                token_amount: TokenAmount::zero(),
            },
        );

        let account_ttl = state.get_account_max_ttl(&config, &store, caller).unwrap();
        assert_eq!(
            account_ttl, tc.expected_account_ttl,
            "Test case '{}' has unexpected account TTL (expected {}, got {})",
            tc.name, tc.expected_account_ttl, account_ttl
        );

        if tc.should_succeed {
            assert!(
                res.is_ok(),
                "Test case '{}' should succeed but failed: {:?}",
                tc.name,
                res.err()
            );

            let res = state.get_blob(&store, hash);
            assert!(res.is_ok(), "Failed to get blob: {:?}", res.err());
            let blob = res.unwrap().unwrap();
            let subscribers = blob.subscribers.hamt(&store).unwrap();
            subscribers
                .for_each(|_, group| {
                    let group_hamt = group.hamt(&store).unwrap();
                    for val in group_hamt.iter() {
                        let (_, sub) = val.unwrap();
                        assert_eq!(
                            sub.expiry,
                            current_epoch + tc.expected_blob_ttl,
                            "Test case '{}' has unexpected blob expiry",
                            tc.name
                        );
                    }
                    Ok(())
                })
                .unwrap();
        } else {
            assert!(
                res.is_err(),
                "Test case '{}' should fail but succeeded",
                tc.name
            );
            assert_eq!(
                res.err().unwrap().msg(),
                format!(
                    "attempt to add a blob with TTL ({}) that exceeds account's max allowed TTL ({})",
                    tc.blob_ttl.map_or_else(|| "none".to_string(), |ttl| ttl.to_string()), tc.account_ttl_status.get_max_ttl(config.blob_default_ttl),
                ),
                "Test case '{}' failed with unexpected error message",
                tc.name
            );
        }
    }
}

#[test]
fn test_add_blob_with_overflowing_ttl() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let current_epoch = ChainEpoch::from(1);
    let amount = TokenAmount::from_whole(1000000);
    state
        .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
        .unwrap();

    let res = state.set_account_status(
        &store,
        &config,
        caller,
        AccountStatus::Extended,
        current_epoch,
    );
    assert!(res.is_ok());

    let (hash, size) = new_hash(1024);
    let res = state.add_blob(
        &store,
        &config,
        caller,
        None,
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size,
            ttl: Some(ChainEpoch::MAX),
            source: new_pk(),
            epoch: current_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());
    let (sub, _) = res.unwrap();
    assert_eq!(sub.expiry, ChainEpoch::MAX);
}

#[test]
fn test_finalize_blob_from_bad_state() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let current_epoch = ChainEpoch::from(1);
    let amount = TokenAmount::from_whole(10);
    state
        .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
        .unwrap();

    // Add a blob
    let (hash, size) = new_hash(1024);
    let res = state.add_blob(
        &store,
        &config,
        caller,
        None,
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

    // Finalize as pending
    let finalize_epoch = ChainEpoch::from(11);
    let res = state.finalize_blob(
        &store,
        caller,
        FinalizeBlobStateParams {
            hash,
            id: SubscriptionId::default(),
            status: BlobStatus::Pending,
            epoch: finalize_epoch,
        },
    );
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().msg(),
        format!("cannot finalize blob {} as added or pending", hash)
    );
}

#[test]
fn test_finalize_blob_resolved() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let current_epoch = ChainEpoch::from(1);
    let amount = TokenAmount::from_whole(10);
    state
        .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
        .unwrap();

    // Add a blob
    let (hash, size) = new_hash(1024);
    let source = new_pk();
    let res = state.add_blob(
        &store,
        &config,
        caller,
        None,
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size,
            ttl: None,
            source,
            epoch: current_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Set to status pending
    let res = state.set_blob_pending(
        &store,
        caller,
        hash,
        size,
        SubscriptionId::default(),
        source,
    );
    assert!(res.is_ok());

    // Finalize as resolved
    let finalize_epoch = ChainEpoch::from(11);
    let res = state.finalize_blob(
        &store,
        caller,
        FinalizeBlobStateParams {
            hash,
            id: SubscriptionId::default(),
            status: BlobStatus::Resolved,
            epoch: finalize_epoch,
        },
    );
    assert!(res.is_ok());

    // Check status
    let status = state
        .get_blob_status(&store, caller, hash, SubscriptionId::default())
        .unwrap()
        .unwrap();
    assert!(matches!(status, BlobStatus::Resolved));

    // Check indexes
    assert_eq!(state.blobs.expiries.len(&store).unwrap(), 1);
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);
}

#[test]
fn test_finalize_blob_failed() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let current_epoch = ChainEpoch::from(1);
    let amount = TokenAmount::from_whole(10);
    state
        .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
        .unwrap();
    let credit_amount = amount * &config.token_credit_rate;

    // Add a blob
    let add_epoch = current_epoch;
    let (hash, size) = new_hash(1024);
    let source = new_pk();
    let res = state.add_blob(
        &store,
        &config,
        caller,
        None,
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size,
            ttl: None,
            source,
            epoch: add_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Set to status pending
    let res = state.set_blob_pending(
        &store,
        caller,
        hash,
        size,
        SubscriptionId::default(),
        source,
    );
    assert!(res.is_ok());

    // Finalize as failed
    let finalize_epoch = ChainEpoch::from(11);
    let res = state.finalize_blob(
        &store,
        caller,
        FinalizeBlobStateParams {
            hash,
            id: SubscriptionId::default(),
            status: BlobStatus::Failed,
            epoch: finalize_epoch,
        },
    );
    assert!(res.is_ok());

    // Check status
    let status = state
        .get_blob_status(&store, caller, hash, SubscriptionId::default())
        .unwrap()
        .unwrap();
    assert!(matches!(status, BlobStatus::Failed));

    // Check the account balance
    let account = state.get_account(&store, caller).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add_epoch);
    assert_eq!(account.credit_committed, Credit::from_whole(0)); // credit was released
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, 0); // capacity was released

    // Check state
    assert_eq!(state.credits.credit_committed, Credit::from_whole(0)); // credit was released
    assert_eq!(state.credits.credit_debited, Credit::from_whole(0));
    assert_eq!(state.blobs.bytes_size(), 0); // capacity was released

    // Check indexes
    assert_eq!(state.blobs.expiries.len(&store).unwrap(), 1); // remains until the blob is explicitly deleted
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);
}

#[test]
fn test_finalize_blob_failed_refund() {
    setup_logs();
    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let caller = new_address();
    let current_epoch = ChainEpoch::from(1);
    let amount = TokenAmount::from_whole(10);
    state
        .buy_credit(&store, &config, caller, amount.clone(), current_epoch)
        .unwrap();
    let mut credit_amount = amount.clone() * &config.token_credit_rate;

    assert!(state
        .set_account_status(
            &store,
            &config,
            caller,
            AccountStatus::Extended,
            current_epoch
        )
        .is_ok());

    // Add a blob
    let add_epoch = current_epoch;
    let (hash, size) = new_hash(1024);
    let source = new_pk();
    let res = state.add_blob(
        &store,
        &config,
        caller,
        None,
        AddBlobStateParams {
            hash,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size,
            ttl: Some(config.blob_min_ttl),
            source,
            epoch: add_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Check the account balance
    let account = state.get_account(&store, caller).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add_epoch);
    assert_eq!(
        account.credit_committed,
        Credit::from_whole(config.blob_min_ttl as u64 * size),
    );
    credit_amount -= &account.credit_committed;
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size);

    // Check state
    assert_eq!(state.credits.credit_committed, account.credit_committed);
    assert_eq!(state.credits.credit_debited, Credit::from_whole(0));
    assert_eq!(state.blobs.bytes_size(), account.capacity_used); // capacity was released

    // Debit accounts to trigger a refund when we fail below
    let debit_epoch = ChainEpoch::from(11);
    let (deletes_from_disc, _) = state.debit_accounts(&store, &config, debit_epoch).unwrap();
    assert!(deletes_from_disc.is_empty());

    // Check the account balance
    let account = state.get_account(&store, caller).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, debit_epoch);
    assert_eq!(
        account.credit_committed,
        Credit::from_whole((config.blob_min_ttl - (debit_epoch - add_epoch)) as u64 * size),
    );
    assert_eq!(account.credit_free, credit_amount); // not changed
    assert_eq!(account.capacity_used, size);

    // Check state
    assert_eq!(state.credits.credit_committed, account.credit_committed);
    assert_eq!(
        state.credits.credit_debited,
        Credit::from_whole((debit_epoch - add_epoch) as u64 * size)
    );
    assert_eq!(state.blobs.bytes_size(), account.capacity_used);

    // Set to status pending
    let res = state.set_blob_pending(
        &store,
        caller,
        hash,
        size,
        SubscriptionId::default(),
        source,
    );
    assert!(res.is_ok());

    // Finalize as failed
    let finalize_epoch = ChainEpoch::from(21);
    let res = state.finalize_blob(
        &store,
        caller,
        FinalizeBlobStateParams {
            hash,
            id: SubscriptionId::default(),
            status: BlobStatus::Failed,
            epoch: finalize_epoch,
        },
    );
    assert!(res.is_ok());

    // Check status
    let status = state
        .get_blob_status(&store, caller, hash, SubscriptionId::default())
        .unwrap()
        .unwrap();
    assert!(matches!(status, BlobStatus::Failed));

    // Check the account balance
    let account = state.get_account(&store, caller).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, debit_epoch);
    assert_eq!(account.credit_committed, Credit::from_whole(0)); // credit was released
    assert_eq!(
        account.credit_free,
        amount.clone() * &config.token_credit_rate
    ); // credit was refunded
    assert_eq!(account.capacity_used, 0); // capacity was released

    // Check state
    assert_eq!(state.credits.credit_committed, Credit::from_whole(0)); // credit was released
    assert_eq!(state.credits.credit_debited, Credit::from_whole(0)); // credit was refunded and released
    assert_eq!(state.blobs.bytes_size(), 0); // capacity was released

    // Check indexes
    assert_eq!(state.blobs.expiries.len(&store).unwrap(), 1); // remains until the blob is explicitly deleted
    assert_eq!(state.blobs.added.len(), 0);
    assert_eq!(state.blobs.pending.len(), 0);
}

#[test]
fn test_delete_blob_refund() {
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
    delete_blob_refund(
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
fn test_delete_blob_refund_with_approval() {
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
    delete_blob_refund(
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
fn delete_blob_refund<BS: Blockstore>(
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
    let mut credit_amount = token_amount * &config.token_credit_rate;

    // Add a blob
    let add1_epoch = current_epoch;
    let (hash1, size1) = new_hash(1024);
    let source1 = new_pk();
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash: hash1,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size: size1,
            ttl: Some(config.blob_min_ttl),
            source: source1,
            epoch: add1_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Finalize as resolved
    let res = state.set_blob_pending(
        &store,
        subscriber,
        hash1,
        size1,
        SubscriptionId::default(),
        source1,
    );
    assert!(res.is_ok());
    let finalize_epoch = ChainEpoch::from(current_epoch + 1);
    let res = state.finalize_blob(
        &store,
        subscriber,
        FinalizeBlobStateParams {
            hash: hash1,
            id: SubscriptionId::default(),
            status: BlobStatus::Resolved,
            epoch: finalize_epoch,
        },
    );
    assert!(res.is_ok());

    // Check stats
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
        Credit::from_whole(config.blob_min_ttl as u64 * size1),
    );
    credit_amount -= &account.credit_committed;
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size1);

    // Add another blob past the first blob expiry
    // This will trigger a debit on the account
    let add2_epoch = ChainEpoch::from(config.blob_min_ttl + 10);
    let (hash2, size2) = new_hash(2048);
    let res = state.add_blob(
        &store,
        config,
        caller,
        sponsor,
        AddBlobStateParams {
            hash: hash2,
            metadata_hash: new_metadata_hash(),
            id: SubscriptionId::default(),
            size: size2,
            ttl: Some(config.blob_min_ttl),
            source: new_pk(),
            epoch: add2_epoch,
            token_amount: TokenAmount::zero(),
        },
    );
    assert!(res.is_ok());

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 2);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 1);
    assert_eq!(stats.bytes_added, size2);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add2_epoch);
    let blob1_expiry = ChainEpoch::from(config.blob_min_ttl + add1_epoch);
    let overcharge = BigInt::from((add2_epoch - blob1_expiry) as u64 * size1);
    assert_eq!(
        account.credit_committed, // this includes an overcharge that needs to be refunded
        Credit::from_whole(config.blob_min_ttl as u64 * size2 - overcharge),
    );
    credit_amount -= Credit::from_whole(config.blob_min_ttl as u64 * size2);
    assert_eq!(account.credit_free, credit_amount);
    assert_eq!(account.capacity_used, size1 + size2);

    // Delete the first blob
    let delete_epoch = ChainEpoch::from(config.blob_min_ttl + 20);
    let (delete_from_disc, deleted_size) = state
        .delete_blob(
            &store,
            caller,
            sponsor,
            DeleteBlobStateParams {
                hash: hash1,
                id: SubscriptionId::default(),
                epoch: delete_epoch,
            },
        )
        .unwrap();
    assert!(delete_from_disc);
    assert_eq!(size1, deleted_size);

    // Check stats
    let stats = state.get_stats(config, TokenAmount::zero());
    assert_eq!(stats.num_blobs, 1);
    assert_eq!(stats.num_resolving, 0);
    assert_eq!(stats.bytes_resolving, 0);
    assert_eq!(stats.num_added, 1);
    assert_eq!(stats.bytes_added, size2);

    // Check the account balance
    let account = state.get_account(&store, subscriber).unwrap().unwrap();
    assert_eq!(account.last_debit_epoch, add2_epoch); // not changed, blob is expired
    assert_eq!(
        account.credit_committed, // should not include overcharge due to refund
        Credit::from_whole(config.blob_min_ttl as u64 * size2),
    );
    assert_eq!(account.credit_free, credit_amount); // not changed
    assert_eq!(account.capacity_used, size2);

    // Check state
    assert_eq!(state.credits.credit_committed, account.credit_committed); // credit was released
    assert_eq!(
        state.credits.credit_debited,
        Credit::from_whole(config.blob_min_ttl as u64 * size1)
    );
    assert_eq!(state.blobs.bytes_size(), size2); // capacity was released

    // Check indexes
    assert_eq!(state.blobs.expiries.len(store).unwrap(), 1);
    assert_eq!(state.blobs.added.len(), 1);
    assert_eq!(state.blobs.pending.len(), 0);

    // Check approval
    if using_approval {
        check_approval_used(&state, store, caller, subscriber);
    }
}

#[test]
fn test_trim_blob_expiries() {
    setup_logs();
    let config = RecallConfig::default();

    const HOUR: ChainEpoch = 3600;
    const TWO_HOURS: ChainEpoch = HOUR * 2;
    const DAY: ChainEpoch = HOUR * 24;
    const YEAR: ChainEpoch = DAY * 365;

    let blobs_ttls: Vec<Option<ChainEpoch>> =
        vec![None, Some(HOUR), Some(TWO_HOURS), Some(DAY), Some(YEAR)];

    struct TestCase {
        name: &'static str,
        account_ttl: AccountStatus,
        expected_ttls: Vec<ChainEpoch>,
        limit: Option<u32>, // None means process all at once
    }

    let test_cases = vec![
        TestCase {
            name: "Set to zero with Reduced status",
            account_ttl: AccountStatus::Reduced,
            expected_ttls: vec![0, 0, 0, 0, 0],
            limit: None,
        },
        TestCase {
            name: "Set to default with Default status",
            account_ttl: AccountStatus::Default,
            expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, DAY],
            limit: None,
        },
        TestCase {
            name: "Set to extended with Extended status",
            account_ttl: AccountStatus::Extended,
            expected_ttls: vec![DAY, HOUR, TWO_HOURS, DAY, YEAR],
            limit: None,
        },
    ];

    for tc in test_cases {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);

        // Setup account with credits and TTL status
        let token = TokenAmount::from_whole(1000);
        state
            .buy_credit(&store, &config, caller, token, current_epoch)
            .unwrap();

        // Set extended TTL status to allow adding all blobs
        state
            .set_account_status(
                &store,
                &config,
                caller,
                AccountStatus::Extended,
                current_epoch,
            )
            .unwrap();

        // Add blobs
        let mut blob_hashes = Vec::new();
        let mut total_cost = Credit::zero();
        let mut expected_credits = Credit::zero();
        for (i, ttl) in blobs_ttls.iter().enumerate() {
            let size = (i + 1) * 1024;
            let (hash, _) = new_hash(size);
            let size = size as u64;
            let id = SubscriptionId::try_from(format!("blob-{}", i)).unwrap();
            let source = new_pk();
            blob_hashes.push(hash);

            state
                .add_blob(
                    &store,
                    &config,
                    caller,
                    None,
                    AddBlobStateParams {
                        hash,
                        metadata_hash: new_metadata_hash(),
                        id: id.clone(),
                        size,
                        ttl: *ttl,
                        source,
                        epoch: current_epoch,
                        token_amount: TokenAmount::zero(),
                    },
                )
                .unwrap();
            state
                .set_blob_pending(&store, caller, hash, size, id.clone(), source)
                .unwrap();
            state
                .finalize_blob(
                    &store,
                    caller,
                    FinalizeBlobStateParams {
                        hash,
                        id,
                        status: BlobStatus::Resolved,
                        epoch: current_epoch,
                    },
                )
                .unwrap();

            total_cost += state.get_storage_cost(ttl.unwrap_or(config.blob_default_ttl), &size);
            expected_credits += state.get_storage_cost(tc.expected_ttls[i], &size);
        }

        let account = state.get_account(&store, caller).unwrap().unwrap();
        assert_eq!(
            account.credit_committed, total_cost,
            "Test case '{}' failed: committed credits don't match",
            tc.name
        );

        state
            .set_account_status(&store, &config, caller, tc.account_ttl, current_epoch)
            .unwrap();

        let res = state.trim_blob_expiries(&config, &store, caller, current_epoch, None, tc.limit);
        assert!(
            res.is_ok(),
            "Test case '{}' failed to trim expiries: {}",
            tc.name,
            res.err().unwrap()
        );

        // Verify expiries were trimmed correctly
        for (i, hash) in blob_hashes.iter().enumerate() {
            // If the TTL is zero, the blob should be deleted
            if tc.expected_ttls[i] == 0 {
                assert!(
                    state.get_blob(&store, *hash).unwrap().is_none(),
                    "Test case '{}' failed: blob {} not deleted",
                    tc.name,
                    i
                );
            } else {
                let blob = state.get_blob(&store, *hash).unwrap().unwrap();
                let subscribers = blob.subscribers.hamt(&store).unwrap();
                let group = subscribers.get(&caller).unwrap().unwrap();
                let group_hamt = group.hamt(&store).unwrap();
                let sub = group_hamt
                    .get(&SubscriptionId::new(&format!("blob-{}", i)).unwrap())
                    .unwrap()
                    .unwrap();

                assert_eq!(
                    sub.expiry - sub.added,
                    tc.expected_ttls[i],
                    "Test case '{}' failed: blob {} expiry not trimmed correctly. Expected {}, got {}",
                    tc.name,
                    i,
                    tc.expected_ttls[i],
                    sub.expiry - sub.added,
                );
            }
        }

        let account = state.get_account(&store, caller).unwrap().unwrap();
        assert_eq!(
            account.credit_committed, expected_credits,
            "Test case '{}' failed: account's committed credits after blob expiry trimming don't match",
            tc.name
        );

        assert_eq!(
            state.credits.credit_committed, expected_credits,
            "Test case '{}' failed: state's committed credits after blob expiry trimming don't match",
            tc.name
        );
    }
}

#[test]
fn test_trim_blob_expiries_pagination() {
    setup_logs();
    let config = RecallConfig::default();

    // Test cases for pagination
    struct PaginationTest {
        name: &'static str,
        limit: Option<u32>,
        start: Option<usize>,
        expected_next_key: Option<usize>,
        expected_processed: usize,
    }

    let test_cases = vec![
        PaginationTest {
            name: "Process all at once",
            limit: None,
            start: None,
            expected_next_key: None,
            expected_processed: 5,
        },
        PaginationTest {
            name: "Process two at a time from beginning",
            limit: Some(2),
            start: None,
            expected_next_key: Some(2),
            expected_processed: 2,
        },
        PaginationTest {
            name: "Process one at a time with offset",
            limit: Some(1),
            start: Some(1),
            expected_next_key: Some(2),
            expected_processed: 1,
        },
        PaginationTest {
            name: "Out of bounds limit",
            limit: Some(10),
            start: Some(1),
            expected_next_key: None,
            expected_processed: 4,
        },
        PaginationTest {
            name: "With offset ending at last item",
            limit: Some(2),
            start: Some(3),
            expected_next_key: None,
            expected_processed: 2,
        },
    ];

    for tc in test_cases {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let caller = new_address();
        let current_epoch = ChainEpoch::from(1);

        // Setup account with credits and Extended TTL status to allow adding all blobs
        state
            .buy_credit(
                &store,
                &config,
                caller,
                TokenAmount::from_whole(1000),
                current_epoch,
            )
            .unwrap();
        state
            .set_account_status(
                &store,
                &config,
                caller,
                AccountStatus::Extended,
                current_epoch,
            )
            .unwrap();

        // Add 5 blobs with different sizes to ensure different hashes
        for i in 0..5 {
            let (hash, size) = new_hash((i + 1) * 1024);
            let id = SubscriptionId::try_from(format!("blob-{}", i)).unwrap();
            let source = new_pk();
            state
                .add_blob(
                    &store,
                    &config,
                    caller,
                    None,
                    AddBlobStateParams {
                        hash,
                        metadata_hash: new_metadata_hash(),
                        id: id.clone(),
                        size,
                        ttl: Some(7200), // 2 hours
                        source,
                        epoch: current_epoch,
                        token_amount: TokenAmount::zero(),
                    },
                )
                .unwrap();
            state
                .set_blob_pending(&store, caller, hash, size, id.clone(), source)
                .unwrap();
            state
                .finalize_blob(
                    &store,
                    caller,
                    FinalizeBlobStateParams {
                        hash,
                        id,
                        status: BlobStatus::Resolved,
                        epoch: current_epoch,
                    },
                )
                .unwrap();
        }

        // Range over all blobs and store their hashes
        let mut blob_hashes = Vec::with_capacity(5);
        for _ in 0..5 {
            let res =
                state
                    .blobs
                    .hamt(&store)
                    .unwrap()
                    .for_each(|hash, _| -> Result<(), ActorError> {
                        blob_hashes.push(hash);
                        Ok(())
                    });
            assert!(
                res.is_ok(),
                "Failed to iterate over blobs: {}",
                res.err().unwrap()
            );
        }

        // Change to Reduced status and process blobs with pagination
        state
            .set_account_status(
                &store,
                &config,
                caller,
                AccountStatus::Reduced,
                current_epoch,
            )
            .unwrap();

        let res = state.trim_blob_expiries(
            &config,
            &store,
            caller,
            current_epoch,
            tc.start.map(|ind| blob_hashes[ind]),
            tc.limit,
        );
        assert!(
            res.is_ok(),
            "Test case '{}' failed to trim expiries: {}",
            tc.name,
            res.err().unwrap()
        );

        let (processed, next, deleted_blobs) = res.unwrap();

        assert_eq!(
            processed as usize, tc.expected_processed,
            "Test case '{}' had unexpected number of items processed",
            tc.name
        );

        assert_eq!(
            deleted_blobs.len(),
            tc.expected_processed,
            "Test case '{}' had unexpected number of deleted blobs",
            tc.name
        );

        if let Some(expected_next_key) = tc.expected_next_key {
            assert!(next.is_some(), "Test case '{}' expected next key", tc.name);
            assert_eq!(
                next.unwrap(),
                blob_hashes[expected_next_key],
                "Test case '{}' had unexpected next key",
                tc.name
            );
        } else {
            assert!(next.is_none(), "Test case '{}' had no next key", tc.name);
        }
    }
}

#[test]
fn test_trim_blob_expiries_for_multiple_accounts() {
    setup_logs();

    let config = RecallConfig::default();
    let store = MemoryBlockstore::default();
    let mut state = State::new(&store).unwrap();
    let address1 = new_address();
    let address2 = new_address();
    let current_epoch = ChainEpoch::from(1);

    // Setup accounts with credits and Extended TTL status to allow adding all blobs
    state
        .buy_credit(
            &store,
            &config,
            address1,
            TokenAmount::from_whole(1000),
            current_epoch,
        )
        .unwrap();
    state
        .buy_credit(
            &store,
            &config,
            address2,
            TokenAmount::from_whole(1000),
            current_epoch,
        )
        .unwrap();
    state
        .set_account_status(
            &store,
            &config,
            address1,
            AccountStatus::Extended,
            current_epoch,
        )
        .unwrap();
    state
        .set_account_status(
            &store,
            &config,
            address2,
            AccountStatus::Extended,
            current_epoch,
        )
        .unwrap();

    // Add blobs for both accounts
    let mut blob_hashes_account1 = Vec::new();
    let mut blob_hashes_account2 = Vec::new();
    for i in 0..3 {
        let (hash, size) = new_hash((i + 1) * 1024);
        let id = SubscriptionId::try_from(format!("blob-1-{}", i)).unwrap();
        let source = new_pk();
        blob_hashes_account1.push(hash);
        state
            .add_blob(
                &store,
                &config,
                address1,
                None,
                AddBlobStateParams {
                    hash,
                    metadata_hash: new_metadata_hash(),
                    id: id.clone(),
                    size,
                    ttl: Some(7200), // 2 hours
                    source,
                    epoch: current_epoch,
                    token_amount: TokenAmount::zero(),
                },
            )
            .unwrap();
        state
            .set_blob_pending(&store, address1, hash, size, id.clone(), source)
            .unwrap();
        state
            .finalize_blob(
                &store,
                address1,
                FinalizeBlobStateParams {
                    hash,
                    id,
                    status: BlobStatus::Resolved,
                    epoch: current_epoch,
                },
            )
            .unwrap();
    }
    for i in 0..3 {
        let (hash, size) = new_hash((i + 1) * 1024);
        let id = SubscriptionId::try_from(format!("blob-2-{}", i)).unwrap();
        let source = new_pk();
        blob_hashes_account2.push(hash);
        state
            .add_blob(
                &store,
                &config,
                address2,
                None,
                AddBlobStateParams {
                    hash,
                    metadata_hash: new_metadata_hash(),
                    id: id.clone(),
                    size,
                    ttl: Some(7200), // 2 hours
                    source,
                    epoch: current_epoch,
                    token_amount: TokenAmount::zero(),
                },
            )
            .unwrap();
        state
            .set_blob_pending(&store, address2, hash, size, id.clone(), source)
            .unwrap();
        state
            .finalize_blob(
                &store,
                address2,
                FinalizeBlobStateParams {
                    hash,
                    id,
                    status: BlobStatus::Resolved,
                    epoch: current_epoch,
                },
            )
            .unwrap();
    }

    // Change TTL status for account1 and trim expiries
    state
        .set_account_status(
            &store,
            &config,
            address1,
            AccountStatus::Reduced,
            current_epoch,
        )
        .unwrap();
    let res = state.trim_blob_expiries(&config, &store, address1, current_epoch, None, None);
    assert!(
        res.is_ok(),
        "Failed to trim expiries for account1: {}",
        res.err().unwrap()
    );

    // Verify account1's blobs were trimmed
    for hash in &blob_hashes_account1 {
        assert!(
            state.get_blob(&store, *hash).unwrap().is_none(),
            "Blob {} for account1 was not deleted",
            hash,
        );
    }

    // Verify account2's blobs were not trimmed
    for hash in &blob_hashes_account2 {
        assert!(
            state.get_blob(&store, *hash).unwrap().is_some(),
            "Blob {} for account2 was incorrectly deleted",
            hash,
        );
    }
}
