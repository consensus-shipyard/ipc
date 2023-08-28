// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Persistent file key store

use crate::evm::memory::MemoryKeyStore;
use crate::evm::{KeyInfo, KeyStore};
use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, ErrorKind};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Default)]
pub struct PersistentKeyStore<T> {
    memory: MemoryKeyStore<T>,
    file_path: PathBuf,
}

/// The persistent key information written to disk
#[derive(Serialize, Deserialize)]
pub struct PersistentKeyInfo {
    /// The address associated with the private key. We can derive this from the private key
    /// but for the ease of debugging, we keep this field
    address: String,
    /// Hex encoded private key
    private_key: String,
}

impl PersistentKeyInfo {
    pub fn new(address: String, private_key: String) -> Self {
        Self {
            address,
            private_key,
        }
    }

    pub fn private_key(&self) -> &str {
        &self.private_key
    }
}

impl Drop for PersistentKeyInfo {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

impl<T: Clone + Eq + Hash + AsRef<[u8]> + TryFrom<KeyInfo>> KeyStore for PersistentKeyStore<T> {
    type Key = T;

    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>> {
        self.memory.get(addr)
    }

    fn list(&self) -> Result<Vec<Self::Key>> {
        self.memory.list()
    }

    fn put(&mut self, info: KeyInfo) -> Result<Self::Key> {
        let addr = self.memory.put(info)?;
        self.flush_no_encryption()?;
        Ok(addr)
    }

    fn remove(&mut self, addr: &Self::Key) -> Result<()> {
        self.memory.remove(addr)?;
        self.flush_no_encryption()
    }
}

impl<T: Clone + Eq + Hash + AsRef<[u8]> + TryFrom<KeyInfo>> PersistentKeyStore<T> {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(p) = path.parent() && !p.exists() {
            return Err(anyhow!("parent does not exist for key store"));
        }

        let p = match File::open(&path) {
            Ok(p) => p,
            Err(e) => {
                return if e.kind() == ErrorKind::NotFound {
                    log::info!("key store does not exist, initialized to empty key store");
                    Ok(Self {
                        memory: MemoryKeyStore {
                            data: Default::default(),
                        },
                        file_path: path,
                    })
                } else {
                    Err(anyhow!("cannot create key store: {e:}"))
                };
            }
        };
        let reader = BufReader::new(p);

        let persisted_key_info: Vec<PersistentKeyInfo> =
            serde_json::from_reader(reader).map_err(|e| {
                anyhow!(
                    "failed to deserialize keyfile, initializing new keystore at: {:?} due to: {e:}",
                    path
                )
            })?;

        let mut key_infos = HashMap::new();
        for info in persisted_key_info.iter() {
            let key_info = KeyInfo {
                private_key: hex::decode(&info.private_key)?,
            };
            let addr = T::try_from(key_info.clone())
                .map_err(|_| anyhow!("cannot convert private key to address"))?;

            key_infos.insert(addr, key_info);
        }

        Ok(Self {
            memory: MemoryKeyStore { data: key_infos },
            file_path: path,
        })
    }

    /// Write all keys to file without any encryption.
    fn flush_no_encryption(&self) -> Result<()> {
        let dir = self
            .file_path
            .parent()
            .ok_or_else(|| anyhow!("Key store parent path not exists"))?;

        fs::create_dir_all(dir)?;

        let file = File::create(&self.file_path)?;

        // TODO: do we need to set path permission?

        let writer = BufWriter::new(file);

        let to_persist = self
            .memory
            .data
            .iter()
            .map(|(key, val)| {
                let address = hex::encode(key.as_ref());
                let private_key = hex::encode(&val.private_key);
                PersistentKeyInfo {
                    address,
                    private_key,
                }
            })
            .collect::<Vec<_>>();

        serde_json::to_writer_pretty(writer, &to_persist)
            .map_err(|e| anyhow!("failed to serialize and write key info: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::evm::KeyInfo;
    use crate::{EvmKeyStore, PersistentKeyStore};

    #[derive(Clone, Eq, PartialEq, Hash)]
    struct Key {
        data: String,
    }

    impl TryFrom<KeyInfo> for Key {
        type Error = ();

        fn try_from(value: KeyInfo) -> Result<Self, Self::Error> {
            Ok(Key {
                data: hex::encode(value.private_key.clone()),
            })
        }
    }

    impl AsRef<[u8]> for Key {
        fn as_ref(&self) -> &[u8] {
            self.data.as_bytes()
        }
    }

    #[test]
    fn test_read_write_keystore() {
        let keystore_folder = tempfile::tempdir().unwrap().into_path();
        let keystore_location = keystore_folder.join("eth_keystore");

        let mut ks = PersistentKeyStore::new(keystore_location.clone()).unwrap();

        let key_info = KeyInfo {
            private_key: vec![0, 1, 2],
        };
        let addr = Key::try_from(key_info.clone()).unwrap();

        ks.put(key_info.clone()).unwrap();

        let key_from_store = ks.get(&addr).unwrap();
        assert!(key_from_store.is_some());
        assert_eq!(key_from_store.unwrap(), key_info);

        // Create the key store again
        let ks = PersistentKeyStore::new(keystore_location).unwrap();
        let key_from_store = ks.get(&addr).unwrap();
        assert!(key_from_store.is_some());
        assert_eq!(key_from_store.unwrap(), key_info);
    }
}
