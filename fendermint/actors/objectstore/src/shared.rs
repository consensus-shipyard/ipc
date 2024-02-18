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

pub const BIT_WIDTH: u32 = 8;

// The state represents an object store backed by a Hamt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // The root cid of the Hamt
    pub root: Cid,
}

// TODO:(carsonfarmer) We'll likely want to define the metadata type that will actually be placed in the Hamt

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        let root = match Hamt::<_, Vec<u8>>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "objectstore actor failed to create empty Hamt: {}",
                    e
                ))
            }
        };

        Ok(Self { root })
    }

    pub fn put<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        content: Vec<u8>,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Vec<u8>>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;

        // TODO:(carsonfarmer) We could use set_if_absent here to avoid overwriting existing objects.
        hamt.set(key, content)?;
        self.root = hamt.flush()?;
        Ok(self.root)
    }

    pub fn append<BS: Blockstore>(
        &mut self,
        store: &BS,
        key: BytesKey,
        content: Vec<u8>,
    ) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Vec<u8>>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let new_content = match hamt.get(&key)? {
            Some(existing) => {
                let mut new_content = existing.clone();
                new_content.extend(content);
                new_content
            }
            None => content,
        };
        hamt.set(key, new_content)?;
        self.root = hamt.flush()?;
        Ok(self.root)
    }

    pub fn delete<BS: Blockstore>(&mut self, store: &BS, key: &BytesKey) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, Vec<u8>>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
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
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let hamt = Hamt::<_, Vec<u8>>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let value = hamt.get(key).map(|v| v.map(|inner| inner.to_owned()))?;
        Ok(value)
    }

    pub fn list<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<Vec<u8>>> {
        let hamt = Hamt::<_, Vec<u8>>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let mut keys = Vec::new();
        hamt.for_each(|k, _| {
            keys.push(k.0.to_owned());
            Ok(())
        })?;
        Ok(keys)
    }
}

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectParams {
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    pub content: Vec<u8>,
    // pub file: String,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PutObject = frc42_dispatch::method_hash!("PutObject"),
    AppendObject = frc42_dispatch::method_hash!("AppendObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::str::FromStr;
//
//     #[test]
//     fn test_constructor() {
//         let store = fvm_ipld_blockstore::MemoryBlockstore::default();
//         let state = State::new(&store);
//         assert!(state.is_ok());
//         assert_eq!(
//             state.unwrap().root,
//             Cid::from_str("bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay")
//                 .unwrap()
//         );
//     }
//
//     #[test]
//     fn test_put_object() {
//         let store = fvm_ipld_blockstore::MemoryBlockstore::default();
//         let mut state = State::new(&store).unwrap();
//         let params = ObjectParams {
//             key: vec![1, 2, 3],
//             content: vec![4, 5, 6],
//         };
//         assert!(state
//             .put(&store, BytesKey(params.key), params.content)
//             .is_ok());
//
//         assert_eq!(
//             state.root,
//             Cid::from_str("bafy2bzacedojzjpwtx565wt43ard7e3kordcw4hf7bbpywpnkctasuwfh7c5m")
//                 .unwrap()
//         );
//     }
//
//     #[test]
//     fn test_append_object() {
//         let store = fvm_ipld_blockstore::MemoryBlockstore::default();
//         let mut state = State::new(&store).unwrap();
//         let params = ObjectParams {
//             key: vec![1, 2, 3],
//             content: vec![4, 5, 6],
//         };
//         assert!(state
//             .append(&store, BytesKey(params.key), params.content)
//             .is_ok());
//     }
//
//     #[test]
//     fn test_delete_object() {
//         let store = fvm_ipld_blockstore::MemoryBlockstore::default();
//         let mut state = State::new(&store).unwrap();
//         let params = ObjectParams {
//             key: vec![1, 2, 3],
//             content: vec![4, 5, 6],
//         };
//         let key = BytesKey(params.key);
//         state.put(&store, key.clone(), params.content).unwrap();
//         assert!(state.delete(&store, &key).is_ok());
//
//         let result = state.get(&store, &key);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), None);
//     }
//
//     #[test]
//     fn test_get_object() {
//         let store = fvm_ipld_blockstore::MemoryBlockstore::default();
//         let mut state = State::new(&store).unwrap();
//         let params = ObjectParams {
//             key: vec![1, 2, 3],
//             content: vec![4, 5, 6],
//         };
//
//         let key = BytesKey(params.key);
//         state.put(&store, key.clone(), params.content).unwrap();
//         let result = state.get(&store, &key);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), Some(vec![4, 5, 6]));
//     }
//
//     #[test]
//     fn test_list_objects() {
//         let store = fvm_ipld_blockstore::MemoryBlockstore::default();
//         let mut state = State::new(&store).unwrap();
//         let params = ObjectParams {
//             key: vec![1, 2, 3],
//             content: vec![4, 5, 6],
//         };
//         state
//             .put(&store, BytesKey(params.key), params.content)
//             .unwrap();
//         let result = state.list(&store);
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), vec![vec![1, 2, 3]]);
//     }
// }
