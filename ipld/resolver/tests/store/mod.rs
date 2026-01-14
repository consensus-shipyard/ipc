// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use cid::Cid as FvmCid;
use fvm_ipld_blockstore::Blockstore;
use ipc_ipld_resolver::missing_blocks::missing_blocks;
use libipld::Cid;
use libp2p_bitswap::BitswapStore;

#[derive(Debug, Clone, Default)]
pub struct TestBlockstore {
    blocks: Arc<RwLock<HashMap<Cid, Vec<u8>>>>,
}

impl Blockstore for TestBlockstore {
    fn has(&self, k: &FvmCid) -> Result<bool> {
        // Convert FvmCid (cid 0.11) to libipld::Cid (cid 0.10)
        let cid_bytes = k.to_bytes();
        let libipld_cid = Cid::try_from(cid_bytes.as_slice())?;
        Ok(self.blocks.read().unwrap().contains_key(&libipld_cid))
    }

    fn get(&self, k: &FvmCid) -> Result<Option<Vec<u8>>> {
        // Convert FvmCid (cid 0.11) to libipld::Cid (cid 0.10)
        let cid_bytes = k.to_bytes();
        let libipld_cid = Cid::try_from(cid_bytes.as_slice())?;
        Ok(self.blocks.read().unwrap().get(&libipld_cid).cloned())
    }

    fn put_keyed(&self, k: &FvmCid, block: &[u8]) -> Result<()> {
        // Convert FvmCid (cid 0.11) to libipld::Cid (cid 0.10)
        let cid_bytes = k.to_bytes();
        let libipld_cid = Cid::try_from(cid_bytes.as_slice())?;
        self.blocks
            .write()
            .unwrap()
            .insert(libipld_cid, block.into());
        Ok(())
    }
}

pub type TestStoreParams = libipld::DefaultParams;

impl BitswapStore for TestBlockstore {
    type Params = TestStoreParams;

    fn contains(&mut self, cid: &Cid) -> Result<bool> {
        // BitswapStore uses libipld::Cid directly, check HashMap directly to avoid double conversion
        Ok(self.blocks.read().unwrap().contains_key(cid))
    }

    fn get(&mut self, cid: &Cid) -> Result<Option<Vec<u8>>> {
        // BitswapStore uses libipld::Cid directly, check HashMap directly
        Ok(self.blocks.read().unwrap().get(cid).cloned())
    }

    fn insert(&mut self, block: &libipld::Block<Self::Params>) -> Result<()> {
        // BitswapStore uses libipld::Cid directly, insert into HashMap directly
        self.blocks
            .write()
            .unwrap()
            .insert(*block.cid(), block.data().to_vec());
        Ok(())
    }

    fn missing_blocks(&mut self, cid: &Cid) -> Result<Vec<Cid>> {
        missing_blocks::<Self, Self::Params>(self, cid)
    }
}
