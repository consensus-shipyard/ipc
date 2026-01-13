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
use fendermint_actor_ipc_storage_config_shared::IPCStorageConfig;
use fvm_ipld_blockstore::{Blockstore, MemoryBlockstore};
use fvm_shared::{address::Address, bigint::BigInt, clock::ChainEpoch, econ::TokenAmount};
use num_traits::Zero;

use super::{
    AddBlobStateParams, DeleteBlobStateParams, FinalizeBlobStateParams, SetPendingBlobStateParams,
};
use crate::{caller::DelegationOptions, testing::check_approval_used, State};

#[test]
fn test_add_blob_refund() {
    setup_logs();
    let config = IPCStorageConfig::default();
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
    let config = IPCStorageConfig::default();
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
    config: &IPCStorageConfig,
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
    let config = IPCStorageConfig::default();
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
    let config = IPCStorageConfig::default();
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
    config: &IPCStorageConfig,
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

    // Add a blob with a default subscription ID
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
    let res = state.set_blob_pending(
        &store,
        subscriber,
        SetPendingBlobStateParams {
            hash,
            size,
            id: id1.clone(),
            source,
        },
    );
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
            source,
            hash,
            size,
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
            skip_credit_return: false,
        },
    );

    assert!(res.is_ok());
    let (delete_from_disk, deleted_size, _) = res.unwrap();
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

    let config = IPCStorageConfig::default();
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
        let config = IPCStorageConfig::default();
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
    let config = IPCStorageConfig::default();
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
    let config = IPCStorageConfig::default();
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

    // Finalize as pending
    let finalize_epoch = ChainEpoch::from(11);
    let res = state.finalize_blob(
        &store,
        caller,
        FinalizeBlobStateParams {
            source,
            hash,
            size,
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
    let config = IPCStorageConfig::default();
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
        SetPendingBlobStateParams {
            hash,
            size,
            id: SubscriptionId::default(),
            source,
        },
    );
    assert!(res.is_ok());

    // Finalize as resolved
    let finalize_epoch = ChainEpoch::from(11);
    let res = state.finalize_blob(
        &store,
        caller,
        FinalizeBlobStateParams {
            source,
            hash,
            size,
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
