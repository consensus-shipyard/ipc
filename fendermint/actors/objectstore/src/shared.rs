// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::strict_bytes;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectParams {
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    #[serde(with = "strict_bytes")]
    pub content: Vec<u8>,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PutObject = frc42_dispatch::method_hash!("PutObject"),
    ResolveObject = frc42_dispatch::method_hash!("ResolveObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
}

pub const BIT_WIDTH: u32 = 8;

/// The state represents an object store backed by a Hamt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The root cid of the Hamt
    pub root: Cid,
}

/// An object in the object store
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Object {
    /// Cid in bytes representation
    #[serde(with = "strict_bytes")]
    pub value: Vec<u8>,
    pub resolved: bool,
}

impl Object {
    fn new(value: Vec<u8>) -> Self {
        Object {
            value,
            resolved: false,
        }
    }
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        let root = match Hamt::<_, Object>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "objectstore actor failed to create empty Hamt: {}",
                    e
                ));
            }
        };

        Ok(Self { root })
    }

    pub fn put<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        value: Vec<u8>,
        overwrite: bool,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;

        if overwrite {
            hamt.set(key, Object::new(value))?;
        } else {
            hamt.set_if_absent(key, Object::new(value))?;
        }
        self.root = hamt.flush()?;
        Ok(self.root)
    }

    pub fn resolve<BS: Blockstore>(&mut self, store: &BS, key: &BytesKey) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        match hamt.get(key).map(|v| v.map(|inner| inner.clone()))? {
            Some(mut obj) => {
                obj.resolved = true;
                hamt.set(key.clone(), obj)?;
                self.root = hamt.flush()?;
                Ok(self.root)
            }
            None => Err(anyhow::anyhow!("key not found")),
        }
    }

    pub fn delete<BS: Blockstore>(&mut self, store: &BS, key: &BytesKey) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        if hamt.contains_key(key)? {
            hamt.delete(key)?;
            self.root = hamt.flush()?;
            return Ok(self.root);
        }
        return Err(anyhow::anyhow!("key not found"));
    }

    pub fn get<BS: Blockstore>(
        &self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<Option<Object>> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let value = hamt.get(key).map(|v| v.map(|inner| inner.clone()))?;
        Ok(value)
    }

    pub fn list<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<Vec<u8>>> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let mut keys = Vec::new();
        hamt.for_each(|k, _| {
            keys.push(k.0.to_owned());
            Ok(())
        })?;
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_constructor() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let state = State::new(&store);
        assert!(state.is_ok());
        assert_eq!(
            state.unwrap().root,
            Cid::from_str("bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay")
                .unwrap()
        );
    }

    #[test]
    fn test_put() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        assert!(state
            .put(&store, BytesKey(vec![1, 2, 3]), vec![4, 5, 6], true)
            .is_ok());

        assert_eq!(
            state.root,
            Cid::from_str("bafy2bzaceaftc6vvulkujsfbntlcxkaywbz2l7u6x7vcanyztgxiv6rfjiick")
                .unwrap()
        );
    }

    #[test]
    fn test_resolve() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        let content = vec![4, 5, 6];
        state
            .put(&store, key.clone(), content.clone(), true)
            .unwrap();
        assert!(state.resolve(&store, &key).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object {
                value: content,
                resolved: true,
            })
        );
    }

    #[test]
    fn test_delete() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state.put(&store, key.clone(), vec![4, 5, 6], true).unwrap();
        assert!(state.delete(&store, &key).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_get() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        let content = vec![4, 5, 6];
        state
            .put(&store, key.clone(), content.clone(), true)
            .unwrap();
        let result = state.get(&store, &key);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object {
                value: content,
                resolved: false,
            })
        );
    }

    #[test]
    fn test_list() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state.put(&store, key.clone(), vec![4, 5, 6], true).unwrap();
        let result = state.list(&store);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![key.0]);
    }
}
