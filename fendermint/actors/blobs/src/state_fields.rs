// Copyright 2022-2024 Protocol Labs
// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashSet};
use std::fmt::Display;

use fendermint_actor_blobs_shared::state::{Account, PublicKey, SubscriptionId};
use fendermint_actor_blobs_shared::state::{Blob, Hash};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use hoku_ipld::hamt;
use hoku_ipld::hamt::map::TrackedFlushResult;
use hoku_ipld::hamt::{BytesKey, MapKey};
use serde::{Deserialize, Serialize};

use crate::state::ExpiryKey;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AccountsState {
    pub root: hamt::Root<Address, Account>,
    size: u64,
}

impl AccountsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, Account>::new(store, "accounts")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Address, Account>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Address, Account>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsState {
    pub root: hamt::Root<Hash, Blob>,
    size: u64,
}

impl BlobsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Hash, Blob>::new(store, "blobs")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Hash, Blob>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Hash, Blob>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Default, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsProgressCollection {
    map: BTreeMap<Hash, BlobsProgressValue>,
    bytes_size: u64,
}

type BlobsProgressValue = HashSet<(Address, SubscriptionId, PublicKey)>;

impl BlobsProgressCollection {
    /// Number of bytes for blobs in the collection
    pub fn bytes_size(&self) -> u64 {
        self.bytes_size
    }

    /// Number of entries
    pub fn len(&self) -> u64 {
        self.map.len() as u64
    }

    /// Add/update added with hash and its source
    pub fn upsert(
        &mut self,
        hash: Hash,
        subscriber: Address,
        id: SubscriptionId,
        source: PublicKey,
        blob_size: u64,
    ) {
        self.map
            .entry(hash)
            .and_modify(|sources| {
                sources.insert((subscriber, id.clone(), source));
            })
            .or_insert_with(|| {
                self.bytes_size += blob_size;
                HashSet::from([(subscriber, id, source)])
            });
    }

    pub fn take_page(&self, size: u32) -> Vec<(Hash, BlobsProgressValue)> {
        self.map
            .iter()
            .take(size as usize)
            .map(|element| (*element.0, element.1.clone()))
            .collect::<Vec<_>>()
    }

    pub fn remove_entry(
        &mut self,
        hash: Hash,
        subscriber: Address,
        sub_id: SubscriptionId,
        source: PublicKey,
        blob_size: u64,
    ) {
        if let Some(entry) = self.map.get_mut(&hash) {
            entry.remove(&(subscriber, sub_id, source));
            self.bytes_size -= blob_size;
            if entry.is_empty() {
                self.map.remove(&hash);
            }
        }
    }

    pub fn insert(
        &mut self,
        hash: Hash,
        value: BlobsProgressValue,
        blob_size: u64,
    ) -> Option<BlobsProgressValue> {
        let result = self.map.insert(hash, value);
        self.bytes_size += blob_size;
        result
    }

    pub fn remove(&mut self, hash: &Hash, blob_size: u64) -> Option<BlobsProgressValue> {
        let result = self.map.remove(hash);
        if result.is_some() {
            self.bytes_size -= blob_size;
        }
        result
    }
}

/// Chain Epoch as a strictly positive number.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ChainEpochAbsolute(u64);

impl TryFrom<ChainEpoch> for ChainEpochAbsolute {
    type Error = ActorError;

    fn try_from(chain_epoch: ChainEpoch) -> Result<Self, Self::Error> {
        if chain_epoch < 0 {
            return Err(ActorError::illegal_argument(
                "Epoch cannot be negative".to_string(),
            ));
        }
        Ok(ChainEpochAbsolute(chain_epoch as u64))
    }
}

impl From<ChainEpochAbsolute> for ChainEpoch {
    fn from(chain_epoch: ChainEpochAbsolute) -> Self {
        chain_epoch.0 as ChainEpoch
    }
}

impl Display for ChainEpochAbsolute {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.0)
    }
}

/// HAMT iteratation is over a byte representation of the keys.
/// Here we use big-endian bytes to represent the chain epoch, suitable for lexicographical ordering.
impl MapKey for ChainEpochAbsolute {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        let arr: [u8; 8] = b
            .try_into()
            .map_err(|_| "Invalid byte length for ChainEpochAbsolute".to_string())?;
        Ok(ChainEpochAbsolute(u64::from_be_bytes(arr)))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.0.to_be_bytes().to_vec())
    }
}

type PerChainEpochRoot = hamt::Root<Address, hamt::Root<ExpiryKey, bool>>;

#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct ExpiriesState {
    pub root: hamt::Root<ChainEpochAbsolute, PerChainEpochRoot>,
    size: u64,
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
        let root = hamt::Root::<ChainEpochAbsolute, PerChainEpochRoot>::new(
            store,
            &ExpiriesState::store_name(),
        )?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, ChainEpochAbsolute, PerChainEpochRoot>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<ChainEpochAbsolute, PerChainEpochRoot>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn foreach_up_to_epoch<BS: Blockstore, F>(
        &self,
        store: BS,
        chain_epoch: ChainEpoch,
        mut f: F,
    ) -> Result<(), ActorError>
    where
        F: FnMut(ChainEpoch, Address, ExpiryKey, bool) -> Result<(), ActorError>,
    {
        let expiries = self.hamt(&store)?;
        let ending_key = BytesKey::from(
            ChainEpochAbsolute::try_from(chain_epoch)?
                .to_bytes()
                .unwrap(),
        );
        expiries.for_each_until(None, &ending_key, |chain_epoch, per_chain_epoch_root| {
            let per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 0)?; // The size is not important here
            per_chain_epoch_hamt.for_each(|address, per_address_root| {
                let per_address_hamt = per_address_root.hamt(&store, 0)?; // The size is not important here
                per_address_hamt.for_each(|expiry_key, auto_renew| {
                    f(chain_epoch.into(), address, expiry_key, *auto_renew)
                })
            })
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
        let mut expiries = self.hamt(&store)?;
        for update in updates {
            match update {
                ExpiryUpdate::Add(chain_epoch, auto_renew) => {
                    let chain_epoch_absolute = ChainEpochAbsolute::try_from(chain_epoch)?;
                    // You cannot do get_or_create here: it expects value, we give it Result<Option<Value>>
                    let per_chain_epoch_root =
                        if let Some(per_chain_epoch_root) = expiries.get(&chain_epoch_absolute)? {
                            per_chain_epoch_root
                        } else {
                            hamt::Root::<Address, hamt::Root<ExpiryKey, bool>>::new(
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
                            hamt::Root::<ExpiryKey, bool>::new(
                                &store,
                                &ExpiriesState::store_name_per_address(chain_epoch, &subscriber),
                            )?
                        };
                    let mut per_address_hamt = per_address_root.hamt(&store, 1)?; // The size does not matter here
                    let expiry_key = ExpiryKey::new(hash, id);
                    let per_address_root =
                        per_address_hamt.set_and_flush(&expiry_key, auto_renew)?;
                    let per_chain_epoch_root =
                        per_chain_epoch_hamt.set_and_flush(&subscriber, per_address_root)?;
                    self.save_tracked(
                        expiries
                            .set_and_flush_tracked(&chain_epoch_absolute, per_chain_epoch_root)?,
                    );
                }
                ExpiryUpdate::Remove(chain_epoch) => {
                    let chain_epoch_absolute = ChainEpochAbsolute::try_from(chain_epoch)?;
                    if let Some(mut per_chain_epoch_root) = expiries.get(&chain_epoch_absolute)? {
                        let mut per_chain_epoch_hamt = per_chain_epoch_root.hamt(&store, 1)?; // The size does not matter here
                        if let Some(mut per_address_root) = per_chain_epoch_hamt.get(&subscriber)? {
                            let mut per_address_hamt = per_address_root.hamt(&store, 1)?; // The size does not matter here
                            let expiry_key = ExpiryKey::new(hash, id);
                            per_address_root = per_address_hamt.delete_and_flush(&expiry_key)?;
                            if per_address_hamt.is_empty() {
                                per_chain_epoch_root =
                                    per_chain_epoch_hamt.delete_and_flush(&subscriber)?;
                            } else {
                                per_chain_epoch_root = per_chain_epoch_hamt
                                    .set_and_flush(&subscriber, per_address_root)?;
                            }
                        }
                        if per_chain_epoch_hamt.is_empty() {
                            self.save_tracked(
                                expiries.delete_and_flush_tracked(&chain_epoch_absolute)?,
                            );
                        } else {
                            self.save_tracked(expiries.set_and_flush_tracked(
                                &chain_epoch_absolute,
                                per_chain_epoch_root,
                            )?);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub enum ExpiryUpdate {
    Add(ChainEpoch, bool), // chain_epoch, auto_renew, it is just annoying to type struct on the caller side
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
        for i in 1..=100 {
            let (hash, _) = new_hash(1024);
            let id = SubscriptionId::default();
            let expiry = ChainEpoch::from(i);
            state
                .update_index(
                    &store,
                    addr,
                    hash,
                    &id,
                    vec![ExpiryUpdate::Add(expiry, false)],
                )
                .unwrap();
        }
        assert_eq!(state.len(), 100);

        let mut range = vec![];
        state
            .foreach_up_to_epoch(&store, 10, |chain_epoch, _, _, _| {
                range.push(chain_epoch);
                Ok(())
            })
            .unwrap();
        assert_eq!(range.len(), 10);
    }
}
