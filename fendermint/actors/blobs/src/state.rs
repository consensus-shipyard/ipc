// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::GetStatsReturn;
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::econ::TokenAmount;

pub mod accounts;
pub mod blobs;
pub mod credit;

use accounts::Accounts;
use blobs::{Blobs, DeleteBlobStateParams};
use credit::Credits;

/// The state represents all accounts and stored blobs.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// Struct containing credit-related state.
    pub credits: Credits,
    /// HAMT containing all accounts keyed by actor ID address.
    pub accounts: Accounts,
    /// HAMT containing all blobs keyed by blob hash.
    pub blobs: Blobs,
}

impl State {
    /// Creates a new [`State`].
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        Ok(Self {
            credits: Credits::default(),
            accounts: Accounts::new(store)?,
            blobs: Blobs::new(store)?,
        })
    }

    /// Returns stats about the current actor state.
    pub fn get_stats(&self, config: &RecallConfig, balance: TokenAmount) -> GetStatsReturn {
        GetStatsReturn {
            balance,
            capacity_free: self.capacity_available(config.blob_capacity),
            capacity_used: self.blobs.bytes_size(),
            credit_sold: self.credits.credit_sold.clone(),
            credit_committed: self.credits.credit_committed.clone(),
            credit_debited: self.credits.credit_debited.clone(),
            token_credit_rate: config.token_credit_rate.clone(),
            num_accounts: self.accounts.len(),
            num_blobs: self.blobs.len(),
            num_added: self.blobs.added.len(),
            bytes_added: self.blobs.added.bytes_size(),
            num_resolving: self.blobs.pending.len(),
            bytes_resolving: self.blobs.pending.bytes_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::blobs::{
        AddBlobStateParams, FinalizeBlobStateParams, SetPendingBlobStateParams,
    };
    use fendermint_actor_blobs_shared::{
        blobs::{BlobStatus, SubscriptionId},
        bytes::B256,
        credit::Credit,
    };
    use fendermint_actor_blobs_testing::{
        new_address, new_hash, new_metadata_hash, new_pk, new_subscription_id, setup_logs,
    };
    use fvm_ipld_blockstore::MemoryBlockstore;
    use fvm_shared::{address::Address, clock::ChainEpoch};
    use log::{debug, warn};
    use num_traits::Zero;
    use rand::{seq::SliceRandom, Rng};
    use std::collections::{BTreeMap, HashMap};

    #[allow(dead_code)]
    fn test_simulate_one_day_multiple_runs() {
        const NUM_RUNS: usize = 1000;
        let mut successful_runs = 0;

        for _ in 0..NUM_RUNS {
            // Run the test in a way that we can catch panics
            let result = std::panic::catch_unwind(|| {
                // Call the existing test method
                test_simulate_one_day();
            });

            match result {
                Ok(_) => {
                    successful_runs += 1;
                }
                Err(_) => {
                    break;
                }
            }
        }

        println!("------- Test Summary -------");
        println!("Total runs: {}", NUM_RUNS);
        println!("Successful runs: {}", successful_runs);
        println!("Failed runs: {}", NUM_RUNS - successful_runs);
        println!(
            "Success rate: {:.2}%",
            (successful_runs as f64 / NUM_RUNS as f64) * 100.0
        );

        // Fail the overall test if any run failed
        assert_eq!(
            successful_runs,
            NUM_RUNS,
            "{} out of {} test runs failed or didn't run",
            NUM_RUNS - successful_runs,
            NUM_RUNS
        );
    }

    #[test]
    fn test_simulate_one_day() {
        setup_logs();

        let config = RecallConfig {
            blob_credit_debit_interval: ChainEpoch::from(10),
            blob_min_ttl: ChainEpoch::from(10),
            ..Default::default()
        };

        #[derive(Clone, Debug)]
        struct TestBlob {
            hash: B256,
            metadata_hash: B256,
            size: u64,
            added: HashMap<Address, Vec<(String, ChainEpoch, ChainEpoch)>>, // added, expiry
        }

        fn generate_test_blobs(count: i64, min_size: usize, max_size: usize) -> Vec<TestBlob> {
            let mut blobs = Vec::new();
            let mut rng = rand::thread_rng();

            for _ in 0..count {
                let size = rng.gen_range(min_size..=max_size);
                let (hash, size) = new_hash(size);
                blobs.push(TestBlob {
                    hash,
                    metadata_hash: new_metadata_hash(),
                    size,
                    added: HashMap::new(),
                });
            }
            blobs
        }

        fn generate_test_users<BS: Blockstore>(
            config: &RecallConfig,
            store: &BS,
            state: &mut State,
            credit_tokens: TokenAmount,
            count: i64,
        ) -> Vec<Address> {
            let mut users = Vec::new();
            for _ in 0..count {
                let user = new_address();
                state
                    .buy_credit(&store, config, user, credit_tokens.clone(), 0)
                    .unwrap();
                users.push(user);
            }
            users
        }

        // Test params
        let epochs: i64 = 360; // num. epochs to run test for
        let user_pool_size: i64 = 10; // some may not be used, some will be used more than once
        let blob_pool_size: i64 = user_pool_size; // some may not be used, some will be used more than once
        let min_ttl = config.blob_min_ttl;
        let max_ttl = epochs;
        let min_size = 10;
        let max_size = 1000;
        let add_intervals = [1, 2, 4, 8, 10, 12, 15, 20]; // used to add at random intervals
        let max_resolve_epochs = 30; // max num. epochs in future to resolve
        let debit_interval: i64 = config.blob_credit_debit_interval; // interval at which to debit all accounts
        let percent_fail_resolve = 0.1; // controls % of subscriptions that fail to resolve

        // Set up store and state
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let mut rng = rand::thread_rng();

        // Get some users
        let credit_tokens = TokenAmount::from_whole(100); // buy a lot
        let user_credit: Credit = credit_tokens.clone() * &config.token_credit_rate;
        let users = generate_test_users(&config, &store, &mut state, credit_tokens, user_pool_size);

        // Get some blobs.
        let mut blobs = generate_test_blobs(blob_pool_size, min_size, max_size);

        // Map of resolve epochs to a set of blob indexes
        #[allow(clippy::type_complexity)]
        let mut resolves: BTreeMap<
            ChainEpoch,
            Vec<(Address, SubscriptionId, B256, u64, B256)>,
        > = BTreeMap::new();
        #[allow(clippy::type_complexity)]
        let mut statuses: HashMap<
            (Address, SubscriptionId, B256),
            (BlobStatus, ChainEpoch),
        > = HashMap::new();

        // Walk epochs.
        // We go for twice the paramaterized epochs to ensure all subscriptions can expire.
        let mut num_added = 0;
        let mut num_readded = 0;
        let mut num_resolved = 0;
        let mut num_failed = 0;
        for epoch in 1..=epochs * 2 {
            if epoch <= epochs {
                let add_interval = add_intervals.choose(&mut rng).unwrap().to_owned();
                if epoch % add_interval == 0 {
                    // Add a random blob with a random user
                    let blob_index = rng.gen_range(0..blobs.len());
                    let blob = unsafe { blobs.get_unchecked_mut(blob_index) };
                    let user_index = rng.gen_range(0..users.len());
                    let user = users[user_index];
                    let sub_id = new_subscription_id(7);
                    let ttl = rng.gen_range(min_ttl..=max_ttl);
                    let source = new_pk();

                    let res = state.add_blob(
                        &store,
                        &config,
                        user,
                        None,
                        AddBlobStateParams {
                            hash: blob.hash,
                            metadata_hash: blob.metadata_hash,
                            id: sub_id.clone(),
                            size: blob.size,
                            ttl: Some(ttl),
                            source,
                            epoch,
                            token_amount: TokenAmount::zero(),
                        },
                    );
                    assert!(res.is_ok());

                    if blob.added.is_empty() {
                        num_added += 1;
                        warn!(
                            "added new blob {} at epoch {} with ttl {}",
                            blob.hash, epoch, ttl
                        );
                    } else {
                        num_readded += 1;
                        warn!(
                            "added new sub to blob {} at epoch {} with ttl {}",
                            blob.hash, epoch, ttl
                        );
                    }

                    // Determine if this will fail or not
                    let fail = rng.gen_bool(percent_fail_resolve);
                    let status = if fail {
                        BlobStatus::Failed
                    } else {
                        BlobStatus::Resolved
                    };
                    statuses.insert((user, sub_id.clone(), blob.hash), (status.clone(), 0));

                    // Track blob interval per user
                    let expiry = epoch + ttl;
                    let added = blob.added.entry(user).or_insert(Vec::new());
                    added.push((sub_id.into(), epoch, expiry));
                }
            }

            // Every debit interval epochs we debit all acounts
            if epoch % debit_interval == 0 {
                let (deletes_from_disc, _) = state.debit_accounts(&store, &config, epoch).unwrap();
                warn!(
                    "deleting {} blobs at epoch {}",
                    deletes_from_disc.len(),
                    epoch
                );
            }

            // Move added blobs to pending state
            let added_blobs = state.get_added_blobs(&store, 1000).unwrap();
            for (hash, size, sources) in added_blobs {
                for (user, id, source) in sources {
                    warn!(
                        "processing added blob {} for {} at epoch {} (id: {})",
                        hash, user, epoch, id
                    );
                    state
                        .set_blob_pending(
                            &store,
                            user,
                            SetPendingBlobStateParams {
                                source,
                                hash,
                                size,
                                id,
                            },
                        )
                        .unwrap();
                }
            }

            // Schedule pending blobs for finalization
            let pending_blobs = state.get_pending_blobs(&store, 1000).unwrap();
            for (hash, size, sources) in pending_blobs {
                for (user, id, source) in sources {
                    if let Some(status) = statuses.get_mut(&(user, id.clone(), hash)) {
                        if status.1 == 0 {
                            let resolve_epoch = rng.gen_range(1..=max_resolve_epochs) + epoch;

                            warn!(
                                "processing pending blob {} for {} at epoch {} (id: {})",
                                hash, user, epoch, id
                            );

                            status.1 = resolve_epoch;
                            resolves
                                .entry(resolve_epoch)
                                .and_modify(|entry| {
                                    entry.push((user, id.clone(), hash, size, source));
                                })
                                .or_insert(vec![(user, id.clone(), hash, size, source)]);
                        }
                    }
                }
            }

            // Resolve blobs
            if let Some(entries) = resolves.get(&epoch) {
                for (user, id, hash, size, source) in entries {
                    let status = statuses.get_mut(&(*user, id.clone(), *hash)).unwrap();
                    match status.0 {
                        BlobStatus::Failed => {
                            num_failed += 1;
                        }
                        BlobStatus::Resolved => {
                            num_resolved += 1;
                        }
                        _ => unreachable!(),
                    }
                    warn!(
                        "finalizing blob {} for {} to status {} at epoch {} (id: {})",
                        hash, user, status.0, epoch, id
                    );
                    let finalized = state
                        .finalize_blob(
                            &store,
                            *user,
                            FinalizeBlobStateParams {
                                source: *source,
                                hash: *hash,
                                size: *size,
                                id: id.clone(),
                                status: status.0.clone(),
                                epoch,
                            },
                        )
                        .unwrap();
                    if !finalized {
                        status.1 = 0;
                    }
                }
            }
        }

        debug!("num. blobs added: {}", num_added);
        debug!("num. blobs re-added: {}", num_readded);
        debug!("num. blobs resolved: {}", num_resolved);
        debug!("num. blobs failed: {}", num_failed);

        // Check global state.
        let stats = state.get_stats(&config, TokenAmount::zero());
        debug!("stats: {:#?}", stats);
        assert_eq!(stats.num_blobs, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);

        // Check the account balances
        let mut total_credit = Credit::zero();
        for (i, user) in users.iter().enumerate() {
            let account = state.get_account(&store, *user).unwrap().unwrap();
            debug!("account {} {}: {:#?}", i, user, account);

            let mut total_user_credit = Credit::zero();
            for blob in blobs.iter() {
                if let Some(added) = blob.added.get(user) {
                    debug!("{} subscriptions to {}", user, blob.hash);
                    let mut intervals = Vec::new();
                    for (id, start, end) in added {
                        if let Some((status, resolve_epoch)) =
                            statuses.get(&(*user, SubscriptionId::new(id).unwrap(), blob.hash))
                        {
                            debug!(
                                "id: {}, size: {}, start: {}, expiry: {}, status: {}, resolved: {}",
                                id, blob.size, start, end, status, resolve_epoch
                            );
                            if status == &BlobStatus::Resolved
                                || (status == &BlobStatus::Failed && *resolve_epoch == 0)
                            {
                                intervals.push((*start as u64, *end as u64));
                            }
                        }
                    }
                    let duration = get_total_duration(intervals) as ChainEpoch;
                    debug!("total duration: {}", duration);
                    let credit = state.get_storage_cost(duration, &blob.size);
                    total_user_credit += &credit;
                }
            }
            debug!("total user credit: {}", total_user_credit);

            assert_eq!(account.capacity_used, 0);
            assert_eq!(account.credit_free, &user_credit - &total_user_credit);
            assert_eq!(account.credit_committed, Credit::zero());

            total_credit += &total_user_credit;
        }

        // Check more global state.
        assert_eq!(stats.capacity_used, 0);
        assert_eq!(stats.credit_committed, Credit::zero());
        assert_eq!(stats.credit_debited, total_credit);
    }

    fn get_total_duration(mut intervals: Vec<(u64, u64)>) -> u64 {
        if intervals.is_empty() {
            return 0;
        }

        // Sort intervals by start time
        intervals.sort_by_key(|&(start, _)| start);

        let mut merged = Vec::new();
        let mut current = intervals[0];

        // Merge overlapping intervals
        for &(start, end) in &intervals[1..] {
            if start <= current.1 {
                // Overlapping interval, extend if needed
                current.1 = current.1.max(end);
            } else {
                // Non-overlapping interval
                merged.push(current);
                current = (start, end);
            }
        }
        merged.push(current);

        merged.iter().map(|&(start, end)| end - start).sum()
    }

    #[test]
    fn test_total_non_overlapping_duration() {
        assert_eq!(get_total_duration(vec![]), 0);
        assert_eq!(get_total_duration(vec![(1, 5)]), 4);
        assert_eq!(get_total_duration(vec![(1, 5), (10, 15)]), 9);
        assert_eq!(get_total_duration(vec![(1, 5), (3, 8)]), 7);
        assert_eq!(get_total_duration(vec![(1, 10), (3, 5)]), 9);
        assert_eq!(
            get_total_duration(vec![(1, 5), (2, 7), (6, 9), (11, 13)]),
            10
        );
        assert_eq!(get_total_duration(vec![(1, 5), (5, 10)]), 9);
        assert_eq!(
            get_total_duration(vec![(11, 13), (1, 5), (6, 9), (2, 7)]),
            10
        );
        assert_eq!(
            get_total_duration(vec![(1, 3), (2, 6), (8, 10), (15, 18), (4, 7), (16, 17)]),
            11
        );
    }
}
