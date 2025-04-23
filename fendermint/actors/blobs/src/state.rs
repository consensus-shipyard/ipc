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
    use crate::state::blobs::{AddBlobStateParams, FinalizeBlobStateParams};
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
    use std::ops::{AddAssign, SubAssign};

    #[test]
    fn test_simulate_one_day() {
        setup_logs();

        let config = RecallConfig {
            blob_credit_debit_interval: ChainEpoch::from(60),
            blob_min_ttl: ChainEpoch::from(10),
            ..Default::default()
        };

        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        struct TestBlob {
            hash: B256,
            metadata_hash: B256,
            size: u64,
            added: Option<ChainEpoch>,
            resolve: Option<ChainEpoch>,
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
                    added: None,
                    resolve: None,
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
        let blob_pool_size: i64 = epochs; // some may not be used, some will be used more than once
        let min_ttl = config.blob_min_ttl;
        let max_ttl = epochs;
        let min_size = 8;
        let max_size = 1024;
        let add_intervals = [1, 2, 4, 8, 10, 12, 15, 20]; // used to add at random intervals
        let max_resolve_epochs = 30; // max num. epochs in future to resolve
        let debit_interval: i64 = config.blob_credit_debit_interval; // interval at which to debit all accounts
        let percent_fail_resolve = 0.1; // controls % of subscriptions that fail resolve

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

        // Map of resolve epochs to set of blob indexes
        #[allow(clippy::type_complexity)]
        let mut resolves: BTreeMap<
            ChainEpoch,
            HashMap<Address, HashMap<usize, (SubscriptionId, B256, Credit)>>,
        > = BTreeMap::new();

        // Walk epochs.
        // We go for twice the paramaterized epochs to ensure all subscriptions can expire.
        let mut num_added = 0;
        let mut num_readded = 0;
        let mut num_resolved = 0;
        let mut num_failed = 0;
        let mut credit_used: HashMap<Address, Credit> = HashMap::new();
        for epoch in 1..=epochs * 2 {
            if epoch <= epochs {
                let add_interval = add_intervals.choose(&mut rng).unwrap().to_owned();
                if epoch % add_interval == 0 {
                    // Add a random blob with a random user
                    let blob_index = rng.gen_range(0..blobs.len());
                    let blob = unsafe { blobs.get_unchecked_mut(blob_index) };
                    if blob.added.is_none() {
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
                        if blob.added.is_none() {
                            num_added += 1;
                            warn!(
                                "added new blob {} at epoch {} with ttl {}",
                                blob.hash, epoch, ttl
                            );
                        } else {
                            warn!(
                                "added new sub to blob {} at epoch {} with ttl {}",
                                blob.hash, epoch, ttl
                            );
                            num_readded += 1;
                        }
                        blob.added = Some(epoch);

                        // Determine how much credit should get committed for this blob
                        let credit = state.get_storage_cost(ttl, &blob.size);
                        // Track credit amount for user, assuming the whole committed amount gets debited
                        credit_used
                            .entry(user)
                            .and_modify(|c| c.add_assign(&credit))
                            .or_insert(credit.clone());

                        // Schedule a resolve to happen in the future
                        let resolve = rng.gen_range(1..=max_resolve_epochs) + epoch;
                        resolves
                            .entry(resolve)
                            .and_modify(|entry| {
                                entry
                                    .entry(user)
                                    .and_modify(|subs| {
                                        subs.insert(
                                            blob_index,
                                            (sub_id.clone(), source, credit.clone()),
                                        );
                                    })
                                    .or_insert(HashMap::from([(
                                        blob_index,
                                        (sub_id.clone(), source, credit.clone()),
                                    )]));
                            })
                            .or_insert(HashMap::from([(
                                user,
                                HashMap::from([(blob_index, (sub_id, source, credit))]),
                            )]));
                    }
                }
            }

            // Resolve blob(s)
            if let Some(users) = resolves.get(&epoch) {
                for (user, index) in users {
                    for (i, (sub_id, source, credit)) in index {
                        let blob = unsafe { blobs.get_unchecked(*i) };
                        let fail = rng.gen_bool(percent_fail_resolve);
                        let status = if fail {
                            num_failed += 1;
                            credit_used
                                .entry(*user)
                                .and_modify(|c| c.sub_assign(credit));
                            BlobStatus::Failed
                        } else {
                            num_resolved += 1;
                            BlobStatus::Resolved
                        };
                        // Simulate the chain putting this blob into pending state, which is
                        // required before finalization.
                        state
                            .set_blob_pending(
                                &store,
                                *user,
                                blob.hash,
                                blob.size,
                                sub_id.clone(),
                                *source,
                            )
                            .unwrap();
                        state
                            .finalize_blob(
                                &store,
                                *user,
                                FinalizeBlobStateParams {
                                    hash: blob.hash,
                                    id: sub_id.clone(),
                                    status,
                                    epoch,
                                },
                            )
                            .unwrap();
                    }
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
        }

        let mut total_credit_used = Credit::zero();
        for (_, credit) in credit_used.clone() {
            total_credit_used.add_assign(&credit);
        }

        debug!("credit used: {}", total_credit_used);
        debug!("num. blobs added: {}", num_added);
        debug!("num. blobs re-added: {}", num_readded);
        debug!("num. blobs resolved: {}", num_resolved);
        debug!("num. blobs failed: {}", num_failed);

        // Check the account balances
        for (i, user) in users.iter().enumerate() {
            let account = state.get_account(&store, *user).unwrap().unwrap();
            debug!("account {}: {:#?}", i, account);
            assert_eq!(account.capacity_used, 0);
            assert_eq!(account.credit_committed, Credit::zero());
            let credit_used = credit_used.get(user).unwrap();
            assert_eq!(account.credit_free, &user_credit - credit_used);
        }

        // Check state.
        // Everything should be empty except for credit_debited.
        let stats = state.get_stats(&config, TokenAmount::zero());
        debug!("stats: {:#?}", stats);
        assert_eq!(stats.capacity_used, 0);
        assert_eq!(stats.credit_committed, Credit::zero());
        assert_eq!(stats.credit_debited, total_credit_used);
        assert_eq!(stats.num_blobs, 0);
        assert_eq!(stats.num_added, 0);
        assert_eq!(stats.bytes_added, 0);
        assert_eq!(stats.num_resolving, 0);
        assert_eq!(stats.bytes_resolving, 0);
    }
}
