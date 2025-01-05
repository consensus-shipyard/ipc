// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Display;
use std::marker::PhantomData;

use cid::Cid;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_hamt::BytesKey;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::core::{Map, MapKey, DEFAULT_HAMT_CONFIG};

#[derive(Clone, PartialEq, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Root<K, V>
where
    K: MapKey + Display,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    cid: Cid,
    name: String,
    #[serde(skip)]
    key_type: PhantomData<K>,
    #[serde(skip)]
    value_type: PhantomData<V>,
}

impl<K, V> Root<K, V>
where
    K: MapKey + Display,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    pub fn new<BS: Blockstore>(store: BS, name: &str) -> Result<Self, ActorError> {
        Hamt::<BS, K, V>::flush_empty(store, name.to_owned())
    }

    pub fn from_cid(cid: Cid, name: String) -> Self {
        Self {
            cid,
            name,
            key_type: Default::default(),
            value_type: Default::default(),
        }
    }

    pub fn hamt<BS: Blockstore>(&self, store: BS, size: u64) -> Result<Hamt<BS, K, V>, ActorError> {
        Hamt::load(store, &self.cid, self.name.clone(), size)
    }

    pub fn cid(&self) -> &Cid {
        &self.cid
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct Hamt<BS, K, V>
where
    BS: Blockstore,
    K: MapKey + Display,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    map: Map<BS, K, V>,
    size: u64,
}

#[derive(Debug, Clone)]
pub struct TrackedFlushResult<K, V>
where
    K: MapKey + Display,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    pub root: Root<K, V>,
    pub size: u64,
}

impl<BS, K, V> Hamt<BS, K, V>
where
    BS: Blockstore,
    K: MapKey + Display,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    fn load(store: BS, root: &Cid, name: String, size: u64) -> Result<Self, ActorError> {
        let map = Map::<BS, K, V>::load(store, root, DEFAULT_HAMT_CONFIG, name)?;
        Ok(Self { map, size })
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, ActorError> {
        self.map.get(key).map(|value| value.cloned())
    }

    pub fn set(&mut self, key: &K, value: V) -> Result<Option<V>, ActorError> {
        let previous = self.map.set(key, value)?;
        if previous.is_none() {
            self.size += 1;
        }
        Ok(previous)
    }

    pub fn set_if_absent(&mut self, key: &K, value: V) -> Result<bool, ActorError> {
        let was_absent = self.map.set_if_absent(key, value.clone())?;
        if was_absent {
            self.size += 1;
        }
        Ok(was_absent)
    }

    pub fn set_and_flush(&mut self, key: &K, value: V) -> Result<Root<K, V>, ActorError> {
        self.set(key, value)?;
        let cid = self.map.flush()?;
        Ok(Root::from_cid(cid, self.map.name()))
    }

    pub fn set_and_flush_tracked(
        &mut self,
        key: &K,
        value: V,
    ) -> Result<TrackedFlushResult<K, V>, ActorError> {
        let root = self.set_and_flush(key, value)?;
        Ok(TrackedFlushResult {
            root,
            size: self.size,
        })
    }

    pub fn get_or_err(&self, key: &K) -> Result<V, ActorError> {
        self.get(key)?.ok_or_else(|| {
            ActorError::not_found(format!("{} not found in {}", key, self.map.name()))
        })
    }

    pub fn get_or_create<F>(&self, key: &K, create_fn: F) -> Result<V, ActorError>
    where
        F: FnOnce() -> V,
    {
        if let Some(value) = self.map.get(key)? {
            Ok(value.clone())
        } else {
            Ok(create_fn())
        }
    }

    pub fn contains_key(&self, key: &K) -> Result<bool, ActorError> {
        self.map.contains_key(key)
    }

    pub fn delete(&mut self, key: &K) -> Result<Option<V>, ActorError> {
        let deleted = self.map.delete(key)?;
        if deleted.is_some() {
            self.size -= 1;
        }
        Ok(deleted)
    }

    pub fn delete_and_flush(&mut self, key: &K) -> Result<(Root<K, V>, Option<V>), ActorError> {
        let deleted = self.delete(key)?;
        let cid = self.map.flush()?;
        Ok((Root::from_cid(cid, self.map.name()), deleted))
    }

    pub fn delete_and_flush_tracked(
        &mut self,
        key: &K,
    ) -> Result<(TrackedFlushResult<K, V>, Option<V>), ActorError> {
        let (root, deleted) = self.delete_and_flush(key)?;
        Ok((
            TrackedFlushResult {
                root,
                size: self.size,
            },
            deleted,
        ))
    }

    pub fn flush(&mut self) -> Result<Root<K, V>, ActorError> {
        let cid = self.map.flush()?;
        Ok(Root::from_cid(cid, self.map.name()))
    }

    pub fn flush_empty(store: BS, name: String) -> Result<Root<K, V>, ActorError> {
        let cid = Map::<BS, K, V>::flush_empty(store, DEFAULT_HAMT_CONFIG)?;
        Ok(Root::from_cid(cid, name))
    }

    pub fn flush_tracked(&mut self) -> Result<TrackedFlushResult<K, V>, ActorError> {
        let root = self.flush()?;
        Ok(TrackedFlushResult {
            root,
            size: self.size,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn for_each<F>(&self, mut f: F) -> Result<(), ActorError>
    where
        F: FnMut(K, &V) -> Result<(), ActorError>,
    {
        self.map.for_each(&mut f)
    }

    pub fn for_each_ranged<F>(
        &self,
        starting_key: Option<&BytesKey>,
        max: Option<usize>,
        mut f: F,
    ) -> Result<(usize, Option<K>), ActorError>
    where
        F: FnMut(K, &V) -> Result<(), ActorError>,
    {
        self.map.for_each_ranged(starting_key, max, &mut f)
    }

    pub fn for_each_until<F>(
        &self,
        starting_key: Option<&BytesKey>,
        ending_key: &BytesKey,
        mut f: F,
    ) -> Result<(), ActorError>
    where
        F: FnMut(K, &V) -> Result<(), ActorError>,
    {
        self.map.for_each_until(starting_key, ending_key, &mut f)
    }
}
