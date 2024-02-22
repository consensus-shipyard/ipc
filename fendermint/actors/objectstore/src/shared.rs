// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectParams {
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    pub value: Cid,
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
    ///
    /// We can't use Cid type because FVM will reject it as unreachable.
    #[serde(with = "strict_bytes")]
    pub value: Vec<u8>,
    /// Whether the object has been resolved.
    pub resolved: bool,
}

impl Object {
    fn new(value: Cid) -> Self {
        Object {
            value: value.to_bytes(),
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
        value: Cid,
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

    pub fn resolve<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        value: Cid,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        match hamt.get(&key).map(|v| v.map(|inner| inner.clone()))? {
            Some(mut obj) => {
                // Ignore if value changed before it was resolved.
                if obj.value == value.to_bytes() {
                    obj.resolved = true;
                    hamt.set(key, obj)?;
                    self.root = hamt.flush()?;
                }
                Ok(self.root)
            }
            // Don't error here in case key was deleted before value was resolved.
            None => Ok(self.root),
        }
    }

    pub fn delete<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<(Option<Object>, Cid)> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        if hamt.contains_key(key)? {
            let value = hamt.delete(key)?.map(|o| o.1);
            self.root = hamt.flush()?;
            return Ok((value, self.root));
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

    pub fn list<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<(Vec<u8>, Object)>> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let mut keys = Vec::new();
        hamt.for_each(|k, v| {
            keys.push((k.0.to_owned(), v.clone()));
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
            .put(&store, BytesKey(vec![1, 2, 3]), Cid::default(), true)
            .is_ok());

        assert_eq!(
            state.root,
            Cid::from_str("bafy2bzaced7xmsrlxozd2kac6vfmp6gw3ynz666vfdgsde2uh2iumbk3hgxcg")
                .unwrap()
        );
    }

    #[test]
    fn test_resolve() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(&store, key.clone(), Cid::default(), true)
            .unwrap();
        assert!(state.resolve(&store, key.clone(), Cid::default()).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object {
                value: Cid::default().to_bytes(),
                resolved: true,
            })
        );
    }

    #[test]
    fn test_delete() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(&store, key.clone(), Cid::default(), true)
            .unwrap();
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
        state
            .put(&store, key.clone(), Cid::default(), true)
            .unwrap();
        let result = state.get(&store, &key);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object {
                value: Cid::default().to_bytes(),
                resolved: false,
            })
        );
    }

    #[test]
    fn test_list() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(&store, key.clone(), Cid::default(), true)
            .unwrap();
        let result = state.list(&store);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result[0].0, key.0);
        assert_eq!(result[0].1.value, Cid::default().to_bytes());
    }
}
