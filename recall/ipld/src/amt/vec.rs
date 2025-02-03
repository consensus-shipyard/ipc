// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

use super::core::{Vec, DEFAULT_AMT_CONFIG};

#[derive(Clone, PartialEq, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Root<V>
where
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    cid: Cid,
    #[serde(skip)]
    value_type: PhantomData<V>,
}

impl<V> Root<V>
where
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    pub fn new<BS: Blockstore>(store: BS) -> Result<Self, ActorError> {
        Amt::<BS, V>::flush_empty(store)
    }

    pub fn from_cid(cid: Cid) -> Self {
        Self {
            cid,
            value_type: Default::default(),
        }
    }

    pub fn amt<BS: Blockstore>(&self, store: BS) -> Result<Amt<BS, V>, ActorError> {
        Amt::load(store, &self.cid)
    }

    pub fn cid(&self) -> &Cid {
        &self.cid
    }
}

pub struct Amt<BS, V>
where
    BS: Blockstore,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    vec: Vec<BS, V>,
}

#[derive(Debug, Clone)]
pub struct TrackedFlushResult<V>
where
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    pub root: Root<V>,
}

impl<BS, V> Amt<BS, V>
where
    BS: Blockstore,
    V: DeserializeOwned + Serialize + PartialEq + Clone,
{
    fn load(store: BS, root: &Cid) -> Result<Self, ActorError> {
        let vec = Vec::<BS, V>::load(store, root)?;
        Ok(Self { vec })
    }

    pub fn get(&self, index: u64) -> Result<Option<V>, ActorError> {
        self.vec.get(index).map(|value| value.cloned())
    }

    pub fn get_or_err(&self, index: u64) -> Result<V, ActorError> {
        self.get(index)?
            .ok_or_else(|| ActorError::not_found(format!("value at index {} not found", index)))
    }

    pub fn set(&mut self, index: u64, value: V) -> Result<(), ActorError> {
        self.vec.set(index, value)
    }

    pub fn set_and_flush(&mut self, index: u64, value: V) -> Result<Root<V>, ActorError> {
        self.set(index, value)?;
        let cid = self.vec.flush()?;
        Ok(Root::from_cid(cid))
    }

    pub fn set_and_flush_tracked(
        &mut self,
        index: u64,
        value: V,
    ) -> Result<TrackedFlushResult<V>, ActorError> {
        let root = self.set_and_flush(index, value)?;
        Ok(TrackedFlushResult { root })
    }

    pub fn delete(&mut self, index: u64) -> Result<Option<V>, ActorError> {
        self.vec.delete(index)
    }

    pub fn delete_and_flush(&mut self, index: u64) -> Result<Root<V>, ActorError> {
        self.delete(index)?;
        let cid = self.vec.flush()?;
        Ok(Root::from_cid(cid))
    }

    pub fn delete_and_flush_tracked(
        &mut self,
        index: u64,
    ) -> Result<TrackedFlushResult<V>, ActorError> {
        let root = self.delete_and_flush(index)?;
        Ok(TrackedFlushResult { root })
    }

    pub fn flush(&mut self) -> Result<Root<V>, ActorError> {
        let cid = self.vec.flush()?;
        Ok(Root::from_cid(cid))
    }

    pub fn flush_empty(store: BS) -> Result<Root<V>, ActorError> {
        let cid = Vec::<BS, V>::flush_empty(store, DEFAULT_AMT_CONFIG)?;
        Ok(Root::from_cid(cid))
    }

    pub fn height(&self) -> u32 {
        self.vec.height()
    }

    pub fn count(&self) -> u64 {
        self.vec.count()
    }

    pub fn for_each_while_ranged<F>(
        &self,
        start_at: Option<u64>,
        limit: Option<u64>,
        mut f: F,
    ) -> Result<(u64, Option<u64>), ActorError>
    where
        F: FnMut(u64, &V) -> Result<bool, ActorError>,
    {
        self.vec.for_each_while_ranged(start_at, limit, &mut f)
    }
}
