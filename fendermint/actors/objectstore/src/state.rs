// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;
use std::string::FromUtf8Error;

use cid::Cid;
use fendermint_actor_blobs_shared::state::Hash;
use fendermint_actor_machine::{Kind, MachineAddress, MachineState, WriteAccess};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::address::Address;
use serde::{Deserialize, Serialize};

const BIT_WIDTH: u32 = 8;
const MAX_LIST_LIMIT: usize = 10000;

fn state_error(e: fvm_ipld_hamt::Error) -> ActorError {
    ActorError::illegal_state(e.to_string())
}

fn utf8_error(e: FromUtf8Error) -> ActorError {
    ActorError::illegal_argument(e.to_string())
}

/// The state represents an object store backed by a Hamt.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The machine address set by the init actor.
    pub address: MachineAddress,
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
    fn new<BS: Blockstore>(
        store: &BS,
        owner: Address,
        write_access: WriteAccess,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<Self, ActorError> {
        let root = match Hamt::<_, ObjectState>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(ActorError::illegal_state(format!(
                    "objectstore actor failed to create empty Hamt: {}",
                    e
                )));
            }
        };
        Ok(Self {
            address: Default::default(),
            owner,
            write_access,
            root,
            metadata,
        })
    }

    fn init(&mut self, address: Address) -> anyhow::Result<(), ActorError> {
        self.address.set(address)
    }

    fn address(&self) -> MachineAddress {
        self.address.clone()
    }

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
pub struct ObjectState {
    /// The object blake3 hash.
    pub hash: Hash,
    /// User-defined object metadata (e.g., last modified timestamp, etc.).
    pub metadata: HashMap<String, String>,
}

/// A list of objects and their common prefixes.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectList {
    /// List of key-values matching the list query.
    pub objects: Vec<(Vec<u8>, ObjectState)>,
    /// When a delimiter is used in the list query, this contains common key prefixes.
    pub common_prefixes: Vec<Vec<u8>>,
}

impl State {
    pub fn add<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        hash: Hash,
        metadata: HashMap<String, String>,
        overwrite: bool,
    ) -> anyhow::Result<Cid, ActorError> {
        let mut hamt = Hamt::<_, ObjectState>::load_with_bit_width(&self.root, store, BIT_WIDTH)
            .map_err(state_error)?;
        let object = ObjectState { hash, metadata };
        if overwrite {
            hamt.set(key, object).map_err(state_error)?;
        } else {
            hamt.set_if_absent(key, object).map_err(state_error)?;
        }
        self.root = hamt.flush().map_err(state_error)?;
        Ok(self.root)
    }

    pub fn delete<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<(ObjectState, Cid), ActorError> {
        let mut hamt = Hamt::<_, ObjectState>::load_with_bit_width(&self.root, store, BIT_WIDTH)
            .map_err(state_error)?;
        let object = hamt
            .delete(key)
            .map_err(state_error)?
            .map(|o| o.1)
            .ok_or(ActorError::not_found("key not found".into()))?;
        self.root = hamt.flush().map_err(state_error)?;
        Ok((object, self.root))
    }

    pub fn get<BS: Blockstore>(
        &self,
        store: &BS,
        key: &BytesKey,
    ) -> anyhow::Result<Option<ObjectState>, ActorError> {
        let hamt = Hamt::<_, ObjectState>::load_with_bit_width(&self.root, store, BIT_WIDTH)
            .map_err(state_error)?;
        let object = hamt.get(key).map(|v| v.cloned()).map_err(state_error)?;
        Ok(object)
    }

    pub fn list<BS: Blockstore, F>(
        &self,
        store: &BS,
        prefix: Vec<u8>,
        delimiter: Vec<u8>,
        offset: u64,
        limit: u64,
        mut collector: F,
    ) -> anyhow::Result<Vec<Vec<u8>>, ActorError>
    where
        F: FnMut(Vec<u8>, ObjectState) -> anyhow::Result<(), ActorError>,
    {
        let hamt = Hamt::<_, ObjectState>::load_with_bit_width(&self.root, store, BIT_WIDTH)
            .map_err(state_error)?;
        let mut common_prefixes = std::collections::BTreeSet::<Vec<u8>>::new();
        let limit = if limit == 0 {
            MAX_LIST_LIMIT
        } else {
            (limit as usize).min(MAX_LIST_LIMIT)
        };
        let mut count = 0;
        let mut collected: usize = 0;
        for pair in &hamt {
            let (k, v) = pair.map_err(state_error)?;
            let key = k.0.clone();
            if !prefix.is_empty() && !key.starts_with(&prefix) {
                continue;
            }
            if !delimiter.is_empty() {
                let utf8_prefix = String::from_utf8(prefix.clone()).map_err(utf8_error)?;
                let prefix_length = utf8_prefix.len();
                let utf8_key = String::from_utf8(key.clone()).map_err(utf8_error)?;
                let utf8_delimiter = String::from_utf8(delimiter.clone()).map_err(utf8_error)?;
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
            collector(key, v.to_owned())?;
            collected += 1;
            if limit > 0 && collected >= limit {
                break;
            }
        }
        let common_prefixes = common_prefixes.into_iter().collect();
        Ok(common_prefixes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fvm_ipld_blockstore::MemoryBlockstore;
    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;
    use rand::RngCore;
    use std::str::FromStr;

    pub fn new_hash(size: usize) -> (Hash, u64) {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; size];
        rng.fill_bytes(&mut data);
        (
            Hash(*iroh_base::hash::Hash::new(&data).as_bytes()),
            size as u64,
        )
    }

    impl Arbitrary for ObjectState {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let hash = new_hash(u16::arbitrary(g) as usize);
            ObjectState {
                hash: hash.0,
                metadata: HashMap::arbitrary(g),
            }
        }
    }

    fn object_one() -> ObjectState {
        let data = [1, 2, 3, 4, 5];
        let hash = iroh_base::hash::Hash::new(data);
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("_created".to_string(), String::from("1718464344"));
        metadata.insert("_modified".to_string(), String::from("1718464345"));
        ObjectState {
            hash: Hash(*hash.as_bytes()),
            metadata,
        }
    }

    const OBJECT_ONE_CID: &str = "bafy2bzacecywsmknwmizkt3yamzqfvik65diqjepqla4e6qlhigblhobtunx4";

    fn object_two() -> ObjectState {
        let data = [6, 7, 8, 9, 10, 11];
        let hash = iroh_base::hash::Hash::new(data);
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("_created".to_string(), String::from("1718464456"));
        metadata.insert("_modified".to_string(), String::from("1718480987"));
        ObjectState {
            hash: Hash(*hash.as_bytes()),
            metadata,
        }
    }

    fn object_three() -> ObjectState {
        let data = [11, 12, 13, 14, 15, 16, 17];
        let hash = iroh_base::hash::Hash::new(data);
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("_created".to_string(), String::from("1718465678"));
        metadata.insert("_modified".to_string(), String::from("1718512346"));
        ObjectState {
            hash: Hash(*hash.as_bytes()),
            metadata,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn list<BS: Blockstore>(
        state: State,
        store: &BS,
        prefix: Vec<u8>,
        delimiter: Vec<u8>,
        offset: u64,
        limit: u64,
    ) -> anyhow::Result<(Vec<(Vec<u8>, ObjectState)>, Vec<Vec<u8>>), ActorError> {
        let mut objects = Vec::new();
        let prefixes = state.list(
            store,
            prefix,
            delimiter,
            offset,
            limit,
            |key: Vec<u8>, object: ObjectState| -> anyhow::Result<(), ActorError> {
                objects.push((key, object));
                Ok(())
            },
        )?;
        Ok((objects, prefixes))
    }

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
        let object = object_one();
        assert!(state
            .add(
                &store,
                BytesKey(vec![1, 2, 3]),
                object.hash,
                object.metadata,
                true
            )
            .is_ok());

        assert_eq!(state.root, Cid::from_str(OBJECT_ONE_CID).unwrap());
    }

    #[quickcheck]
    fn test_delete(object: ObjectState) {
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
            .add(&store, key.clone(), object.hash, object.metadata, true)
            .unwrap();
        assert!(state.delete(&store, &key).is_ok());

        let result = state.get(&store, &key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[quickcheck]
    fn test_get(object: ObjectState) {
        let store = MemoryBlockstore::default();
        let mut state = State::new(
            &store,
            Address::new_id(100),
            WriteAccess::OnlyOwner,
            HashMap::new(),
        )
        .unwrap();
        let key = BytesKey(vec![1, 2, 3]);
        let md = object.metadata.clone();
        state
            .add(&store, key.clone(), object.hash, md, true)
            .unwrap();
        let result = state.get(&store, &key);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), object);
    }

    fn create_and_put_objects(
        state: &mut State,
        store: &MemoryBlockstore,
    ) -> anyhow::Result<(BytesKey, BytesKey, BytesKey)> {
        let baz_key = BytesKey("foo/baz.png".as_bytes().to_vec()); // index 0
        let object = object_one();
        state.add(store, baz_key.clone(), object.hash, object.metadata, false)?;
        let bar_key = BytesKey("foo/bar.png".as_bytes().to_vec()); // index 1
        let object = object_two();
        state.add(store, bar_key.clone(), object.hash, object.metadata, false)?;
        // We'll mostly ignore this one
        let other_key = BytesKey("zzzz/image.png".as_bytes().to_vec()); // index 2
        let hash = new_hash(256);
        state.add(
            &store,
            other_key.clone(),
            hash.0,
            HashMap::<String, String>::new(),
            false,
        )?;
        let jpeg_key = BytesKey("foo.jpeg".as_bytes().to_vec()); // index 3
        let object = object_three();
        state.add(store, jpeg_key.clone(), object.hash, object.metadata, false)?;
        Ok((baz_key, bar_key, jpeg_key))
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

        let (baz_key, _, _) = create_and_put_objects(&mut state, &store).unwrap();

        // List all keys with a limit
        let result = list(state, &store, vec![], vec![], 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0.len(), 4);
        assert_eq!(result.0.first(), Some(&(baz_key.0, object_one())));
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

        let (baz_key, bar_key, _) = create_and_put_objects(&mut state, &store).unwrap();

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let result = list(state, &store, foo_key.0.clone(), vec![], 0, 0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0.len(), 3);
        assert_eq!(result.0[0], (baz_key.0, object_one()));
        assert_eq!(result.0[1], (bar_key.0, object_two()));
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

        let (_, _, jpeg_key) = create_and_put_objects(&mut state, &store).unwrap();

        let foo_key = BytesKey("foo".as_bytes().to_vec());
        let delimiter_key = BytesKey("/".as_bytes().to_vec());
        let full_key = [foo_key.clone(), delimiter_key.clone()].concat();
        let result = list(
            state,
            &store,
            foo_key.0.clone(),
            delimiter_key.0.clone(),
            0,
            3,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0], (jpeg_key.0, object_three()));
        assert_eq!(result.1[0], full_key);
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
        let hash = new_hash(256);
        state
            .add(
                &store,
                jpeg_key.clone(),
                hash.0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();
        let bar_key = BytesKey("bin/foo/bar.png".as_bytes().to_vec());
        let hash = new_hash(256);
        state
            .add(
                &store,
                bar_key.clone(),
                hash.0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();
        let baz_key = BytesKey("bin/foo/baz.png".as_bytes().to_vec());
        let hash = new_hash(256);
        state
            .add(
                &store,
                baz_key.clone(),
                hash.0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();

        let bin_key = BytesKey("bin/".as_bytes().to_vec());
        let full_key = BytesKey("bin/foo/".as_bytes().to_vec());
        let delimiter_key = BytesKey("/".as_bytes().to_vec());
        let result = list(
            state,
            &store,
            bin_key.0.clone(),
            delimiter_key.0.clone(),
            0,
            0,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0.len(), 0);
        assert_eq!(result.1.len(), 1);
        assert_eq!(result.1[0], full_key.0);
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

        // List all keys with a limit and offset
        let result = list(state, &store, vec![], vec![], 1, 1);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0.len(), 1);
        // Note that baz is listed first in order, so an offset of 1 will return bar
        assert_eq!(result.0.first(), Some(&(bar_key.0, object_two())));
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
        let hash = new_hash(256);
        state
            .add(
                &store,
                one.clone(),
                hash.0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();
        let two = BytesKey("hello/again".as_bytes().to_vec());
        let hash = new_hash(256);
        state
            .add(
                &store,
                two.clone(),
                hash.0,
                HashMap::<String, String>::new(),
                false,
            )
            .unwrap();

        // List all keys with a limit and offset
        let result = list(
            state,
            &store,
            "hello/".as_bytes().to_vec(),
            "/".as_bytes().to_vec(),
            2,
            0,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0.len(), 0);
    }
}
