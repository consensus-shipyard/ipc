// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fendermint_actor_blobs_shared::{accounts::AccountStatus, bytes::B256};
use fendermint_actor_recall_config_shared::RecallConfig;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, clock::ChainEpoch};
use log::{debug, warn};

use super::Account;
use crate::{caller::Caller, state::DeleteBlobStateParams, State};

impl State {
    /// Returns an [`Account`] by address.
    pub fn get_account<BS: Blockstore>(
        &self,
        store: &BS,
        address: Address,
    ) -> Result<Option<Account>, ActorError> {
        let accounts = self.accounts.hamt(store)?;
        accounts.get(&address)
    }

    /// Sets an account's [`TtlStatus`] by address.
    ///
    /// Flushes state to the blockstore.
    pub fn set_account_status<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        address: Address,
        status: AccountStatus,
        current_epoch: ChainEpoch,
    ) -> Result<(), ActorError> {
        let mut accounts = self.accounts.hamt(store)?;
        match status {
            // We don't want to create an account for default TTL
            AccountStatus::Default => {
                if let Some(mut account) = accounts.get(&address)? {
                    account.max_ttl = status.get_max_ttl(config.blob_default_ttl);
                    self.accounts
                        .save_tracked(accounts.set_and_flush_tracked(&address, account)?);
                }
            }
            _ => {
                // Get or create a new account
                let max_ttl = status.get_max_ttl(config.blob_default_ttl);
                let mut account = accounts
                    .get_or_create(&address, || Account::new(store, current_epoch, max_ttl))?;
                account.max_ttl = max_ttl;
                self.accounts
                    .save_tracked(accounts.set_and_flush_tracked(&address, account)?);
            }
        }
        Ok(())
    }

    /// Debits accounts for their blob usage and cleans up expired blob subscriptions.
    ///
    /// This method performs two main operations:
    /// 1. Deletes expired blob subscriptions based on the current epoch
    /// 2. Debits a batch of accounts for their ongoing blob storage usage
    ///
    /// The debiting process works in cycles, processing a subset of accounts in each call
    /// to avoid excessive computation in a single pass. The number of accounts processed
    /// in each batch is controlled by the subnet config parameter `account_debit_batch_size`.
    /// Similarly, expired blob deletion is controlled by `blob_delete_batch_size`.
    ///
    /// Flushes state to the blockstore.
    ///
    /// TODO: Break this into two methods called by a `cron_tick` actor method.
    pub fn debit_accounts<BS: Blockstore>(
        &mut self,
        store: &BS,
        config: &RecallConfig,
        current_epoch: ChainEpoch,
    ) -> Result<(HashSet<B256>, bool), ActorError> {
        // Delete expired subscriptions
        let mut delete_from_disc = HashSet::new();
        let mut num_deleted = 0;
        let mut expiries = self.blobs.expiries.clone();
        expiries.foreach_up_to_epoch(
            store,
            current_epoch,
            Some(config.blob_delete_batch_size),
            |_, subscriber, key| {
                match self.delete_blob(
                    store,
                    subscriber,
                    None,
                    DeleteBlobStateParams {
                        hash: key.hash,
                        id: key.id.clone(),
                        epoch: current_epoch,
                    },
                ) {
                    Ok((from_disc, _)) => {
                        num_deleted += 1;
                        if from_disc {
                            delete_from_disc.insert(key.hash);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "failed to delete blob {} for {} (id: {}): {}",
                            key.hash, subscriber, key.id, e
                        )
                    }
                }
                Ok(())
            },
        )?;

        debug!("deleted {} expired subscriptions", num_deleted);
        debug!(
            "{} blobs marked for deletion from disc",
            delete_from_disc.len()
        );

        // Debit accounts for existing usage
        let reader = self.accounts.hamt(store)?;
        let mut writer = self.accounts.hamt(store)?;
        let start_key = self.accounts.get_debit_start_address();
        let (count, next_account) = reader.for_each_ranged(
            start_key.as_ref(),
            Some(config.account_debit_batch_size as usize),
            |address, account| {
                let mut caller =
                    Caller::load_account(store, &reader, address, account.clone(), None)?;
                self.debit_caller(&mut caller, current_epoch);
                caller.save(&mut writer)?;
                Ok(true)
            },
        )?;

        // Save accounts
        self.accounts.save_tracked(writer.flush_tracked()?);
        self.accounts.save_debit_progress(next_account);

        debug!(
            "finished debiting {:#?} accounts, next account: {:#?}",
            count, next_account
        );

        Ok((delete_from_disc, next_account.is_some()))
    }
}
