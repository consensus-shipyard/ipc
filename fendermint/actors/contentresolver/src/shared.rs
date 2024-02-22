// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const BIT_WIDTH: u32 = 8;

// The state represents an object store backed by a Hamt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // The root cid of the Amt
    pub root: Cid,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        let root = match Hamt::<_, u8>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "contentresolver actor failed to create empty Hamt: {}",
                    e
                ));
            }
        };

        Ok(Self { root })
    }

    pub fn push<BS: Blockstore>(&mut self, store: &BS, cid: BytesKey) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, u8>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;

        hamt.set_if_absent(cid, 1)?;
        self.root = hamt.flush()?;
        Ok(self.root)
    }

    pub fn delete<BS: Blockstore>(&mut self, store: &BS, cid: &BytesKey) -> anyhow::Result<Cid> {
        let mut hamt = Hamt::<_, u8>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        if hamt.contains_key(cid)? {
            hamt.delete(cid)?;
            self.root = hamt.flush()?;
            return Ok(self.root);
        }
        return Err(anyhow::anyhow!("cid not found"));
    }

    pub fn list<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<Vec<u8>>> {
        let hamt = Hamt::<_, u8>::load_with_bit_width(&self.root, store, BIT_WIDTH)?;
        let mut cids = Vec::new();
        hamt.for_each(|k, _| {
            cids.push(k.0.to_owned());
            Ok(())
        })?;
        Ok(cids)
    }
}

pub const CONTENTRESOLVER_ACTOR_NAME: &str = "contentresolver";

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PushCid = frc42_dispatch::method_hash!("PushCid"),
    DeleteCid = frc42_dispatch::method_hash!("DeleteCid"),
    ListCids = frc42_dispatch::method_hash!("ListCids"),
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
    fn test_push_cid() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        assert!(state.push(&store, BytesKey(vec![1, 2, 3])).is_ok());

        assert_eq!(
            state.root,
            Cid::from_str("bafy2bzaceb3ytldnk4acp3f3yglh2uobmlx5gtmnuoe4i6htojkgko74tehew")
                .unwrap()
        );
    }

    #[test]
    fn test_delete_cid() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let cid = BytesKey(vec![1, 2, 3]);
        state.push(&store, cid.clone()).unwrap();
        assert!(state.delete(&store, &cid).is_ok());

        let result = state.list(&store);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_list_cids() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        state.push(&store, BytesKey(vec![1, 2, 3])).unwrap();
        let result = state.list(&store);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![vec![1, 2, 3]]);
    }
}
