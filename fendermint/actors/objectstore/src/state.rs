// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_ipld_hamt::{BytesKey, Hamt};

pub const BIT_WIDTH: u32 = 8;

const DEFAULT_LIST_LIMIT: usize = 100;
const MAX_LIST_LIMIT: usize = 500;

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

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectList {
    pub objects: Vec<(Vec<u8>, Object)>,
    pub common_prefixes: Vec<Vec<u8>>,
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
        match hamt.get(&key).map(|v| v.cloned())? {
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
        Err(anyhow::anyhow!("key not found"))
    }

    pub fn get<BS: Blockstore>(
        &self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<Option<Object>> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let value = hamt.get(key).map(|v| v.cloned())?;
        Ok(value)
    }

    pub fn list<BS: Blockstore>(
        &self,
        store: &BS,
        prefix: Option<&BytesKey>,
        delimiter: Option<&BytesKey>,
        limit: Option<usize>,
    ) -> anyhow::Result<ObjectList> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let iter = hamt.iter();
        let mut objects = Vec::new();
        let mut common_prefixes = std::collections::BTreeSet::<Vec<u8>>::new();
        let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT).min(MAX_LIST_LIMIT);
        'pairs: for pair in iter {
            if let Ok((k, v)) = pair {
                let key = k.0.to_owned();
                if let Some(prefix) = prefix {
                    if !key.starts_with(prefix) {
                        continue 'pairs;
                    }
                }
                if let Some(delimiter) = delimiter {
                    let utf8_key = String::from_utf8(key.clone()).unwrap();
                    let utf8_delimiter = String::from_utf8(delimiter.0.clone()).unwrap();
                    if let Some(index) = utf8_key.find(&utf8_delimiter) {
                        let subset = utf8_key[..index].as_bytes().to_owned();
                        common_prefixes.insert(subset);
                        continue 'pairs;
                    }
                }
                objects.push((key, v.clone()));
                if objects.len() >= limit as usize {
                    break 'pairs;
                }
            }
        }
        let common_prefixes = common_prefixes.into_iter().collect();
        let result = ObjectList {
            objects,
            common_prefixes,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use fvm_ipld_blockstore::MemoryBlockstore;

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

    fn create_and_put_objects(
        state: &mut State,
        store: &MemoryBlockstore,
    ) -> anyhow::Result<(BytesKey, BytesKey, BytesKey)> {
        let jpeg_key = BytesKey("foo.jpeg".as_bytes().to_vec());
        state.put(store, jpeg_key.clone(), Cid::default(), false)?;
        let bar_key = BytesKey("foo/bar.png".as_bytes().to_vec());
        state.put(store, bar_key.clone(), Cid::default(), false)?;
        let baz_key = BytesKey("foo/baz.png".as_bytes().to_vec());
        state.put(store, baz_key.clone(), Cid::default(), false)?;

        // We'll ignore this one entirely
        let other_key = BytesKey("zzzz/image.png".as_bytes().to_vec());
        state.put(&store, other_key.clone(), Cid::default(), false)?;
        Ok((jpeg_key, bar_key, baz_key))
    }

    #[test]
    fn test_list_all_keys() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();

        let (_, _, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_object = Object {
            value: Cid::default().to_bytes(),
            resolved: false,
        };

        // List all keys with a limit
        let result = state.list(&store, None, None, Some(1));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.objects.first(), Some(&(baz_key.0, default_object)));
    }

    #[test]
    fn test_list_keys_with_prefix() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();

        let (_, bar_key, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_object = Object {
            value: Cid::default().to_bytes(),
            resolved: false,
        };

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let result = state.list(&store, Some(&foo_key), None, None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 3);
        assert_eq!(result.objects[0], (baz_key.0, default_object.clone()));
        assert_eq!(result.objects[1], (bar_key.0, default_object.clone()));
    }

    #[test]
    fn test_list_keys_with_delimiter() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();

        let (jpeg_key, _, _) = create_and_put_objects(&mut state, &store).unwrap();

        let default_object = Object {
            value: Cid::default().to_bytes(),
            resolved: false,
        };

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let delimiter_key = BytesKey("/".as_bytes().to_vec());
        let result = state.list(&store, Some(&foo_key), Some(&delimiter_key), Some(3));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.objects[0], (jpeg_key.0, default_object));
        assert_eq!(result.common_prefixes[0], foo_key.0);
    }
}
