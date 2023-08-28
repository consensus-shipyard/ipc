// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Memory key store

use crate::evm::{KeyInfo, KeyStore};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Default)]
pub struct MemoryKeyStore<T> {
    pub(crate) data: HashMap<T, KeyInfo>,
}

impl<T: Clone + Eq + Hash + TryFrom<KeyInfo>> KeyStore for MemoryKeyStore<T> {
    type Key = T;

    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>> {
        Ok(self.data.get(addr).cloned())
    }

    fn list(&self) -> Result<Vec<Self::Key>> {
        Ok(self.data.keys().cloned().collect())
    }

    fn put(&mut self, info: KeyInfo) -> Result<Self::Key> {
        let addr = Self::Key::try_from(info.clone())
            .map_err(|_| anyhow!("cannot convert private key to public key"))?;
        self.data.insert(addr.clone(), info);
        Ok(addr)
    }

    fn remove(&mut self, addr: &Self::Key) -> Result<()> {
        self.data.remove(addr);
        Ok(())
    }
}
