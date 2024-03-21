// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use fil_actors_evm_shared::address::EthAddress;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{strict_bytes::ByteBuf, tuple::*, DAG_CBOR};
use fvm_ipld_hamt::{BytesKey, Hamt};
use serde::{Deserialize, Serialize};

pub const BIT_WIDTH: u32 = 8;

const MAX_LIST_LIMIT: usize = 10000;

/// The state represents an object store backed by a Hamt.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The machine creator
    pub creator: EthAddress,
    /// The root cid of the Hamt.
    pub root: Cid,
}

/// The stored representation of an object in the object store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Object {
    /// Internal objects are stored on-chain.
    Internal(ByteBuf),
    /// External objects reference an off-chain object by Cid.
    /// The bool indicates whether the object has been resolved.
    External((ByteBuf, bool)),
}

/// The kind of object. This is used during object creation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObjectKind {
    /// Internal objects are stored on-chain.
    Internal(ByteBuf),
    /// External objects reference an off-chain object by Cid.
    External(Cid),
}

/// A list of objects and their common prefixes.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectList {
    pub objects: Vec<(Vec<u8>, ObjectListItem)>,
    pub common_prefixes: Vec<Vec<u8>>,
}

/// The kind of object. This is used during object creation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjectListItem {
    /// Internal objects are stored on-chain.
    Internal((Cid, u64)),
    /// External objects reference an off-chain object by Cid.
    External((Cid, bool)),
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS, creator: EthAddress) -> anyhow::Result<Self> {
        let root = match Hamt::<_, Object>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "objectstore actor failed to create empty Hamt: {}",
                    e
                ));
            }
        };
        Ok(Self { creator, root })
    }

    pub fn put<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        kind: ObjectKind,
        overwrite: bool,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let object = match kind {
            ObjectKind::Internal(buf) => Object::Internal(buf),
            ObjectKind::External(cid) => Object::External((ByteBuf(cid.to_bytes()), false)),
        };
        if overwrite {
            hamt.set(key, object)?;
        } else {
            hamt.set_if_absent(key, object)?;
        }
        self.root = hamt.flush()?;
        Ok(self.root)
    }

    pub fn resolve_external<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        value: Cid,
    ) -> anyhow::Result<()> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        match hamt.get(&key).map(|v| v.cloned())? {
            Some(object) => {
                match object {
                    Object::Internal(_) => Ok(()),
                    Object::External((v, _)) => {
                        // Ignore if value changed before it was resolved.
                        if v.0 == value.to_bytes() {
                            hamt.set(key, Object::External((v, true)))?;
                            self.root = hamt.flush()?;
                        }
                        Ok(())
                    }
                }
            }
            // Don't error here in case key was deleted before value was resolved.
            None => Ok(()),
        }
    }

    pub fn delete<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<(Option<Object>, Cid)> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        if hamt.contains_key(key)? {
            let object = hamt.delete(key)?.map(|o| o.1);
            self.root = hamt.flush()?;
            return Ok((object, self.root));
        }
        Err(anyhow::anyhow!("key not found"))
    }

    pub fn get<BS: Blockstore>(
        &self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<Option<Object>> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let object = hamt.get(key).map(|v| v.cloned())?;
        Ok(object)
    }

    pub fn list<BS: Blockstore>(
        &self,
        store: &BS,
        prefix: Vec<u8>,
        delimiter: Vec<u8>,
        offset: u64,
        limit: u64,
    ) -> anyhow::Result<ObjectList> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let mut objects = Vec::new();
        let mut common_prefixes = std::collections::BTreeSet::<Vec<u8>>::new();
        let limit = if limit == 0 {
            MAX_LIST_LIMIT
        } else {
            (limit as usize).min(MAX_LIST_LIMIT)
        };
        let mut count = 0;
        for pair in &hamt {
            let (k, v) = pair?;
            let key = k.0.clone();
            if !prefix.is_empty() && !key.starts_with(&prefix) {
                continue;
            }
            if !delimiter.is_empty() {
                let utf8_prefix = String::from_utf8(prefix.clone())?;
                let prefix_length = utf8_prefix.len();
                let utf8_key = String::from_utf8(key.clone())?;
                let utf8_delimiter = String::from_utf8(delimiter.clone())?;
                if let Some(index) = utf8_key[prefix_length..].find(&utf8_delimiter) {
                    let subset = utf8_key[..=(index + prefix_length)].as_bytes().to_owned();
                    common_prefixes.insert(subset);
                    continue;
                }
            }
            count += 1;
            if count < offset {
                continue;
            }
            let item = match v {
                Object::Internal(b) => {
                    let mh_code = Code::Blake2b256;
                    let mh = mh_code.digest(&b.0);
                    let cid = Cid::new_v1(DAG_CBOR, mh);
                    ObjectListItem::Internal((cid, b.0.len() as u64))
                }
                Object::External((b, resolved)) => {
                    ObjectListItem::External((Cid::try_from(b.0.as_slice())?, *resolved))
                }
            };
            objects.push((key, item));
            if limit > 0 && objects.len() >= limit {
                break;
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
        let store = MemoryBlockstore::default();
        let state = State::new(&store, EthAddress::from_id(100));
        assert!(state.is_ok());
        assert_eq!(
            state.unwrap().root,
            Cid::from_str("bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay")
                .unwrap()
        );
    }

    #[test]
    fn test_put_internal() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        assert!(state
            .put(
                &store,
                BytesKey(vec![1, 2, 3]),
                ObjectKind::Internal(ByteBuf(vec![4, 5, 6])),
                true
            )
            .is_ok());

        assert_eq!(
            state.root,
            Cid::from_str("bafy2bzacecyfqn52y34p5fu2yxne263thvwc2pitaec36ul3l7ghocmwjli5k")
                .unwrap()
        );
    }

    #[test]
    fn test_put_external() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        assert!(state
            .put(
                &store,
                BytesKey(vec![1, 2, 3]),
                ObjectKind::External(Cid::default()),
                true
            )
            .is_ok());

        assert_eq!(
            state.root,
            Cid::from_str("bafy2bzaceaq7b4t24dmwbnkztaib3jlv7bcajecsqobshg6juoitave2rxws2")
                .unwrap()
        );
    }

    #[test]
    fn test_resolve_external() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(
                &store,
                key.clone(),
                ObjectKind::External(Cid::default()),
                true,
            )
            .unwrap();
        assert!(state
            .resolve_external(&store, key.clone(), Cid::default())
            .is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object::External((ByteBuf(Cid::default().to_bytes()), true)))
        );
    }

    #[test]
    fn test_delete_internal() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(
                &store,
                key.clone(),
                ObjectKind::Internal(ByteBuf(vec![4, 5, 6])),
                true,
            )
            .unwrap();
        assert!(state.delete(&store, &key).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_delete_external() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(
                &store,
                key.clone(),
                ObjectKind::External(Cid::default()),
                true,
            )
            .unwrap();
        assert!(state.delete(&store, &key).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_get_internal() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(
                &store,
                key.clone(),
                ObjectKind::Internal(ByteBuf(vec![4, 5, 6])),
                true,
            )
            .unwrap();
        let result = state.get(&store, &key);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object::Internal(ByteBuf(vec![4, 5, 6]))),
        );
    }

    #[test]
    fn test_get_external() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .put(
                &store,
                key.clone(),
                ObjectKind::External(Cid::default()),
                true,
            )
            .unwrap();
        let result = state.get(&store, &key);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object::External((
                ByteBuf(Cid::default().to_bytes()),
                false
            )))
        );
    }

    fn create_and_put_objects(
        state: &mut State,
        store: &MemoryBlockstore,
    ) -> anyhow::Result<(BytesKey, BytesKey, BytesKey)> {
        let jpeg_key = BytesKey("foo.jpeg".as_bytes().to_vec());
        state.put(
            store,
            jpeg_key.clone(),
            ObjectKind::Internal(ByteBuf(vec![4, 5, 6])),
            false,
        )?;
        let bar_key = BytesKey("foo/bar.png".as_bytes().to_vec());
        state.put(
            store,
            bar_key.clone(),
            ObjectKind::External(Cid::default()),
            false,
        )?;
        let baz_key = BytesKey("foo/baz.png".as_bytes().to_vec());
        state.put(
            store,
            baz_key.clone(),
            ObjectKind::External(Cid::default()),
            false,
        )?;

        // We'll mostly ignore this one
        let other_key = BytesKey("zzzz/image.png".as_bytes().to_vec());
        state.put(
            &store,
            other_key.clone(),
            ObjectKind::External(Cid::default()),
            false,
        )?;
        Ok((jpeg_key, bar_key, baz_key))
    }

    #[test]
    fn test_list_all_keys() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();

        let (_, _, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_item = ObjectListItem::External((Cid::default(), false));

        // List all keys with a limit
        let result = state.list(&store, vec![], vec![], 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 4);
        assert_eq!(result.objects.first(), Some(&(baz_key.0, default_item)));
    }

    #[test]
    fn test_list_keys_with_prefix() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();

        let (_, bar_key, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_item = ObjectListItem::External((Cid::default(), false));

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let result = state.list(&store, foo_key.0.clone(), vec![], 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 3);
        assert_eq!(result.objects[0], (baz_key.0, default_item.clone()));
        assert_eq!(result.objects[1], (bar_key.0, default_item.clone()));
    }

    #[test]
    fn test_list_keys_with_delimiter() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();

        let (jpeg_key, _, _) = create_and_put_objects(&mut state, &store).unwrap();

        let mh_code = Code::Blake2b256;
        let mh = mh_code.digest(&[4, 5, 6]);
        let cid = Cid::new_v1(DAG_CBOR, mh);
        let default_item = ObjectListItem::Internal((cid, 3));

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let delimiter_key = BytesKey("/".as_bytes().to_vec());
        let full_key = [foo_key.clone(), delimiter_key.clone()].concat();
        let result = state.list(&store, foo_key.0.clone(), delimiter_key.0.clone(), 0, 3);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.objects[0], (jpeg_key.0, default_item));
        assert_eq!(result.common_prefixes[0], full_key);
    }

    #[test]
    fn test_list_keys_with_nested_delimiter() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();

        let jpeg_key = BytesKey("foo.jpeg".as_bytes().to_vec());
        state
            .put(
                &store,
                jpeg_key.clone(),
                ObjectKind::External(Cid::default()),
                false,
            )
            .unwrap();
        let bar_key = BytesKey("bin/foo/bar.png".as_bytes().to_vec());
        state
            .put(
                &store,
                bar_key.clone(),
                ObjectKind::External(Cid::default()),
                false,
            )
            .unwrap();
        let baz_key = BytesKey("bin/foo/baz.png".as_bytes().to_vec());
        state
            .put(
                &store,
                baz_key.clone(),
                ObjectKind::External(Cid::default()),
                false,
            )
            .unwrap();

        let bin_key = BytesKey("bin/".as_bytes().to_vec());
        let full_key = BytesKey("bin/foo/".as_bytes().to_vec());
        let delimiter_key = BytesKey("/".as_bytes().to_vec());
        let result = state.list(&store, bin_key.0.clone(), delimiter_key.0.clone(), 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 0);
        assert_eq!(result.common_prefixes.len(), 1);
        assert_eq!(result.common_prefixes[0], full_key.0);
    }

    #[test]
    fn test_list_with_offset_and_limit() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(&store, EthAddress::from_id(100)).unwrap();

        let (_, _, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_item = ObjectListItem::External((Cid::default(), false));

        // List all keys with a limit and offset
        let result = state.list(&store, vec![], vec![], 1, 1);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.objects.first(), Some(&(baz_key.0, default_item)));
    }
}
