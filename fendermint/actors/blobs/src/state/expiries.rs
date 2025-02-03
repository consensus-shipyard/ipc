// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::Hash;
use fendermint_actor_blobs_shared::state::SubscriptionId;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use recall_ipld::amt::vec::TrackedFlushResult;
use recall_ipld::{amt, hamt};

use crate::state::ExpiryKey;

type PerChainEpochRoot = hamt::Root<Address, hamt::Root<ExpiryKey, ()>>;

#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct ExpiriesState {
    pub root: amt::Root<PerChainEpochRoot>,
}

impl ExpiriesState {
    fn store_name() -> String {
        "expiries".to_string()
    }

    fn store_name_per_chain_epoch(chain_epoch: ChainEpoch) -> String {
        format!("{}.{}", ExpiriesState::store_name(), chain_epoch)
    }

    fn store_name_per_address(chain_epoch: ChainEpoch, address: &Address) -> String {
        format!(
            "{}.{}",
            ExpiriesState::store_name_per_chain_epoch(chain_epoch),
            address
        )
    }

    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = amt::Root::<PerChainEpochRoot>::new(store)?;
        Ok(Self { root })
    }

    pub fn amt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<amt::vec::Amt<BS, PerChainEpochRoot>, ActorError> {
        self.root.amt(store)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<PerChainEpochRoot>) {
        self.root = tracked_flush_result.root;
    }

    pub fn len<BS: Blockstore>(&self, store: BS) -> Result<u64, ActorError> {
        Ok(self.root.amt(store)?.count())
    }

    pub fn foreach_up_to_epoch<BS: Blockstore, F>(
        &self,
        store: BS,
        epoch: ChainEpoch,
        mut f: F,
    ) -> Result<(), ActorError>
    where
        F: FnMut(ChainEpoch, Address, ExpiryKey) -> Result<(), ActorError>,
    {
        let expiries = self.amt(&store)?;
        expiries.for_each_while_ranged(None, None, |index, per_chain_epoch_root| {
            if index > epoch as u64 {
                return Ok(false);
            }
            let per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 0)?; // The size is not important here
            per_chain_epoch_hamt.for_each(|address, per_address_root| {
                let per_address_hamt = per_address_root.hamt(&store, 0)?; // The size is not important here
                per_address_hamt.for_each(|expiry_key, _| f(index as i64, address, expiry_key))
            })?;
            Ok(true)
        })?;
        Ok(())
    }

    pub fn update_index<BS: Blockstore>(
        &mut self,
        store: BS,
        subscriber: Address,
        hash: Hash,
        id: &SubscriptionId,
        updates: Vec<ExpiryUpdate>,
    ) -> Result<(), ActorError> {
        let mut expiries = self.amt(&store)?;
        for update in updates {
            match update {
                ExpiryUpdate::Add(chain_epoch) => {
                    // You cannot do get_or_create here: it expects value, we give it Result<Option<Value>>
                    let per_chain_epoch_root =
                        if let Some(per_chain_epoch_root) = expiries.get(chain_epoch as u64)? {
                            per_chain_epoch_root
                        } else {
                            hamt::Root::<Address, hamt::Root<ExpiryKey, ()>>::new(
                                &store,
                                &ExpiriesState::store_name_per_chain_epoch(chain_epoch),
                            )?
                        };
                    // The size does not matter
                    let mut per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 1)?;
                    // You cannot do get_or_create here: it expects value, we give it Result<Option<Value>>
                    let per_address_root =
                        if let Some(per_address_root) = per_chain_epoch_hamt.get(&subscriber)? {
                            per_address_root
                        } else {
                            hamt::Root::<ExpiryKey, ()>::new(
                                &store,
                                &ExpiriesState::store_name_per_address(chain_epoch, &subscriber),
                            )?
                        };
                    let mut per_address_hamt = per_address_root.hamt(&store, 1)?; // The size does not matter here
                    let expiry_key = ExpiryKey::new(hash, id);
                    let per_address_root = per_address_hamt.set_and_flush(&expiry_key, ())?;
                    let per_chain_epoch_root =
                        per_chain_epoch_hamt.set_and_flush(&subscriber, per_address_root)?;
                    self.save_tracked(
                        expiries.set_and_flush_tracked(chain_epoch as u64, per_chain_epoch_root)?,
                    );
                }
                ExpiryUpdate::Remove(chain_epoch) => {
                    if let Some(mut per_chain_epoch_root) = expiries.get(chain_epoch as u64)? {
                        let mut per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 1)?; // The size does not matter here
                        if let Some(mut per_address_root) = per_chain_epoch_hamt.get(&subscriber)? {
                            let mut per_address_hamt = per_address_root.hamt(&store, 1)?; // The size does not matter here
                            let expiry_key = ExpiryKey::new(hash, id);
                            (per_address_root, _) =
                                per_address_hamt.delete_and_flush(&expiry_key)?;
                            if per_address_hamt.is_empty() {
                                (per_chain_epoch_root, _) =
                                    per_chain_epoch_hamt.delete_and_flush(&subscriber)?;
                            } else {
                                per_chain_epoch_root = per_chain_epoch_hamt
                                    .set_and_flush(&subscriber, per_address_root)?;
                            }
                        }
                        if per_chain_epoch_hamt.is_empty() {
                            self.save_tracked(
                                expiries.delete_and_flush_tracked(chain_epoch as u64)?,
                            );
                        } else {
                            self.save_tracked(
                                expiries.set_and_flush_tracked(
                                    chain_epoch as u64,
                                    per_chain_epoch_root,
                                )?,
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub enum ExpiryUpdate {
    Add(ChainEpoch),
    Remove(ChainEpoch),
}

#[cfg(test)]
mod tests {
    use super::*;

    use fendermint_actor_blobs_testing::{new_address, new_hash};
    use fvm_ipld_blockstore::MemoryBlockstore;

    #[test]
    fn test_expiries_foreach_up_to_epoch() {
        let store = MemoryBlockstore::default();
        let mut state = ExpiriesState::new(&store).unwrap();

        let addr = new_address();
        let mut hashes = vec![];
        for i in 1..=100 {
            let (hash, _) = new_hash(1024);
            let expiry = ChainEpoch::from(i);
            state
                .update_index(
                    &store,
                    addr,
                    hash,
                    &SubscriptionId::default(),
                    vec![ExpiryUpdate::Add(expiry)],
                )
                .unwrap();
            hashes.push(hash);
        }
        assert_eq!(state.len(&store).unwrap(), 100);

        let mut range = vec![];
        state
            .foreach_up_to_epoch(&store, 10, |chain_epoch, _, _| {
                range.push(chain_epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(range.len(), 10);

        // Remove an element to test against a sparse state
        let remove_epoch = 5;
        let hash = hashes[remove_epoch - 1];
        state
            .update_index(
                &store,
                addr,
                hash,
                &SubscriptionId::default(),
                vec![ExpiryUpdate::Remove(remove_epoch as ChainEpoch)],
            )
            .unwrap();
        assert_eq!(state.len(&store).unwrap(), 99);

        let mut range = vec![];
        state
            .foreach_up_to_epoch(&store, 10, |chain_epoch, _, _| {
                range.push(chain_epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(range.len(), 9);
    }
}
