// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_machine::{Kind, MachineState, WriteAccess};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{strict_bytes::ByteBuf, tuple::*};
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::address::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BIT_WIDTH: u32 = 8;

const MAX_LIST_LIMIT: usize = 10000;

/// The state represents an object store backed by a Hamt.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The machine robust owner address.
    pub owner: Address,
    /// Write access dictates who can write to the machine.
    pub write_access: WriteAccess,
    /// The root cid of the Hamt.
    pub root: Cid,
    /// User-defined metadata (e.g., bucket name, etc.).
    pub metadata: HashMap<String, String>,
}

impl MachineState for State {
    fn kind(&self) -> Kind {
        Kind::ObjectStore
    }

    fn owner(&self) -> Address {
        self.owner
    }

    fn write_access(&self) -> WriteAccess {
        self.write_access
    }

    fn metadata(&self) -> HashMap<String, String> {
        self.metadata.clone()
    }
}

/// The stored representation of an object in the object store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    /// The object content identifier.
    pub cid: ByteBuf,
    /// The size of the content.
    pub size: usize,
    /// Whether the object has been resolved.
    pub resolved: bool,
    /// User-defined object metadata (e.g., last modified timestamp, etc.).
    pub metadata: HashMap<String, String>,
}

/// A list of objects and their common prefixes.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectList {
    /// List of key-values matching the list query.
    pub objects: Vec<(Vec<u8>, Object)>,
    /// When a delimiter is used in the list query, this contains common key prefixes.
    pub common_prefixes: Vec<Vec<u8>>,
}

impl State {
    pub fn new<BS: Blockstore>(
        store: &BS,
        creator: Address,
        write_access: WriteAccess,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        let root = match Hamt::<_, Object>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "objectstore actor failed to create empty Hamt: {}",
                    e
                ));
            }
        };
        Ok(Self {
            owner: creator,
            write_access,
            root,
            metadata,
        })
    }

    pub fn add<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        cid: Cid,
        size: usize,
        metadata: HashMap<String, String>,
        overwrite: bool,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let object = Object {
            cid: ByteBuf(cid.to_bytes()),
            size,
            resolved: false,
            metadata,
        };
        if overwrite {
            hamt.set(key, object)?;
        } else {
            hamt.set_if_absent(key, object)?;
        }
        self.root = hamt.flush()?;
        Ok(self.root)
    }

    pub fn resolve<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        value: Cid,
    ) -> anyhow::Result<()> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        match hamt.get(&key).map(|v| v.cloned())? {
            Some(mut object) => {
                // Ignore if value changed before it was resolved.
                if object.cid.0 == value.to_bytes() {
                    object.resolved = true;
                    hamt.set(key, object)?;
                    self.root = hamt.flush()?;
                }
                Ok(())
            }
            // Don't error here in case the key was deleted before the value was resolved.
            None => Ok(()),
        }
    }

    pub fn delete<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<(Object, Cid)> {
        let mut hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let object = hamt
            .delete(key)?
            .map(|o| o.1)
            .ok_or(anyhow::anyhow!("key not found"))?;
        self.root = hamt.flush()?;
        Ok((object, self.root))
    }

    pub fn get<BS: Blockstore>(&self, store: &BS, key: &BytesKey) -> anyhow::Result<Option<Object>> {
        let hamt = Hamt::<_, Object>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let object = hamt.get(key)
            .map(|v| v.cloned())?;
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
            if count <= offset {
                continue;
            }
            objects.push((key, v.to_owned()));
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
    use super::*;
    use cid::multihash::{Code, MultihashDigest};
    use fendermint_testing::arb::ArbCid;
    use fil_actors_runtime::MapKey;
    use fvm_ipld_blockstore::MemoryBlockstore;
    use fvm_ipld_encoding::DAG_CBOR;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;
    use std::str::FromStr;

    impl Arbitrary for Object {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Object {
                cid: ByteBuf(ArbCid::<64>::arbitrary(g).0.to_bytes()),
                size: usize::arbitrary(g),
                metadata: HashMap::arbitrary(g),
                resolved: false,
            }
        }
    }

    fn default_object() -> Object {
        Object {
            cid: ByteBuf(Cid::default().to_bytes()),
            size: 0,
            metadata: HashMap::<String, String>::new(),
            resolved: false,
        }
    }

    fn golden_object() -> Object {
        let mh_code = Code::Blake2b256;
        let mh = mh_code.digest(&[1, 2, 3, 4, 5]);
        let cid = Cid::new_v1(DAG_CBOR, mh);
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("_size".to_string(), String::from("5"));
        metadata.insert("_created".to_string(), String::from("1718464344"));
        metadata.insert("_modified".to_string(), String::from("1718464345"));
        Object {
            cid: ByteBuf(cid.to_bytes()),
            size: 5,
            metadata,
            resolved: false,
        }
    }

    const GOLDEN_CID: &str = "bafy2bzacebmog6w3ept45xctbw3lrt76i3rdbeaib6bikuhcddu5y5bqspozu";

    #[test]
    fn test_constructor() {
        let store = MemoryBlockstore::default();
        let state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        );
        assert!(state.is_ok());
        assert_eq!(
            state.unwrap().root,
            Cid::from_str("bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay")
                .unwrap()
        );
    }

    #[test]
    fn test_add() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();
        let object = golden_object();
        assert!(state
            .add(
                &store,
                BytesKey(vec![1, 2, 3]),
                Cid::from_bytes(&object.cid.0).unwrap(),
                object.size,
                object.metadata,
                true
            )
            .is_ok());

        assert_eq!(state.root, Cid::from_str(GOLDEN_CID).unwrap());
    }

    #[quickcheck]
    fn test_resolve(mut object: Object) {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        let cid = Cid::from_bytes(&object.cid.0).unwrap();
        let md = object.metadata.clone();
        state
            .add(&store, key.clone(), cid, object.size, md, true)
            .unwrap();
        assert!(state.resolve(&store, key.clone(), cid).is_ok());

        object.resolved = true;
        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), object);
    }

    #[quickcheck]
    fn test_delete(object: Object) {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        state
            .add(
                &store,
                key.clone(),
                Cid::from_bytes(&object.cid.0).unwrap(),
                object.size,
                object.metadata,
                true,
            )
            .unwrap();
        assert!(state.delete(&store, &key).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[quickcheck]
    fn test_get(object: Object) {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        let cid = Cid::from_bytes(&object.cid.0).unwrap();
        let md = object.metadata.clone();
        state
            .add(&store, key.clone(), cid, object.size, md, true)
            .unwrap();
        let result = state.get(&store, &key);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), object);
    }

    fn create_and_put_objects(
        state: &mut State,
        store: &MemoryBlockstore,
    ) -> anyhow::Result<(BytesKey, BytesKey, BytesKey)> {
        let jpeg_key = BytesKey("foo.jpeg".as_bytes().to_vec());
        state.add(
            store,
            jpeg_key.clone(),
            Cid::default(),
            0,
            HashMap::<String, String>::new(),
            false,
        )?;
        let bar_key = BytesKey("foo/bar.png".as_bytes().to_vec());
        state.add(
            store,
            bar_key.clone(),
            Cid::default(),
            0,
            HashMap::<String, String>::new(),
            false,
        )?;
        let baz_key = BytesKey("foo/baz.png".as_bytes().to_vec());
        state.add(
            store,
            baz_key.clone(),
            Cid::default(),
            0,
            HashMap::<String, String>::new(),
            false,
        )?;

        // We'll mostly ignore this one
        let other_key = BytesKey("zzzz/image.png".as_bytes().to_vec());
        state.add(
            &store,
            other_key.clone(),
            Cid::default(),
            0,
            HashMap::<String, String>::new(),
            false,
        )?;
        Ok((jpeg_key, bar_key, baz_key))
    }

    #[test]
    fn test_list_all_keys() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();

        let (_, _, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_obj = default_object();

        // List all keys with a limit
        let result = state.list(&store, vec![], vec![], 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 4);
        assert_eq!(result.objects.first(), Some(&(baz_key.0, default_obj)));
    }

    #[test]
    fn test_list_keys_with_prefix() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();

        let (_, bar_key, baz_key) = create_and_put_objects(&mut state, &store).unwrap();

        let default_obj = default_object();

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let result = state.list(&store, foo_key.0.clone(), vec![], 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 3);
        assert_eq!(result.objects[0], (baz_key.0, default_obj.clone()));
        assert_eq!(result.objects[1], (bar_key.0, default_obj.clone()));
    }

    #[test]
    fn test_list_keys_with_delimiter() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();

        let (jpeg_key, _, _) = create_and_put_objects(&mut state, &store).unwrap();

        let default_obj = default_object();

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let delimiter_key = BytesKey("/".as_bytes().to_vec());
        let full_key = [foo_key.clone(), delimiter_key.clone()].concat();
        let result = state.list(&store, foo_key.0.clone(), delimiter_key.0.clone(), 0, 3);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 1);
        assert_eq!(result.objects[0], (jpeg_key.0, default_obj));
        assert_eq!(result.common_prefixes[0], full_key);
    }

    #[test]
    fn test_list_keys_with_nested_delimiter() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();

        let jpeg_key = BytesKey("foo.jpeg".as_bytes().to_vec());
        state
            .add(
                &store,
                jpeg_key.clone(),
                Cid::default(),
                0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();
        let bar_key = BytesKey("bin/foo/bar.png".as_bytes().to_vec());
        state
            .add(
                &store,
                bar_key.clone(),
                Cid::default(),
                0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();
        let baz_key = BytesKey("bin/foo/baz.png".as_bytes().to_vec());
        state
            .add(
                &store,
                baz_key.clone(),
                Cid::default(),
                0,
                HashMap::<String, String>::new(),
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
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();

        let (_, bar_key, _) = create_and_put_objects(&mut state, &store).unwrap();

        let default_obj = default_object();

        // List all keys with a limit and offset
        let result = state.list(&store, vec![], vec![], 1, 1);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 1);
        // Note that baz is listed first in order, so an offset of 1 will return bar
        assert_eq!(result.objects.first(), Some(&(bar_key.0, default_obj)));
    }

    #[test]
    fn test_list_with_prefix_delimiter_and_offset_and_limit() {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();

        let one = BytesKey("hello/world".as_bytes().to_vec());
        state
            .add(
                &store,
                one.clone(),
                Cid::default(),
                0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();
        let two = BytesKey("hello/again".as_bytes().to_vec());
        state
            .add(
                &store,
                two.clone(),
                Cid::default(),
                0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();

        // List all keys with a limit and offset
        let result = state.list(
            &store,
            "hello/".as_bytes().to_vec(),
            "/".as_bytes().to_vec(),
            2,
            0,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.objects.len(), 0);
    }
}
