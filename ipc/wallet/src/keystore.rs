// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fs::{create_dir, File};
use fs_err as fs;
use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, ErrorKind, Read, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use log::{debug, error};
use serde::{Deserialize, Serialize};

use super::errors::WalletErr;

pub const PLAIN_JSON_KEYSTORE_NAME: &str = "keystore.json";

/// Environmental variable which holds the `KeyStore` encryption phrase.
pub const FOREST_KEYSTORE_PHRASE_ENV: &str = "FOREST_KEYSTORE_PHRASE";

use crate::crypto::*;

/// Configuration type for constructing a `KeyStore`
pub enum KeyStoreConfig {
    /// Create an in-memory only, empty keystore
    InMemory,
    /// Create a plain, un-encrypted keystore, not recommended outside of integration tests
    Plain { path: PathBuf },
    /// Create a encrypted keystore, using the given password
    Encrypted { location: PathBuf, password: String },
}

impl KeyStoreConfig {
    /// Create a new _plain_ keystore config item
    pub fn plain(path: impl AsRef<Path>) -> Self {
        Self::Plain {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Create a new _encrypted_ keystore config item
    pub fn encrypted(path: impl AsRef<Path>, password: impl Into<String>) -> Self {
        Self::Encrypted {
            location: path.as_ref().to_path_buf(),
            password: password.into(),
        }
    }
}

/// Plain persistent `KeyStore` in JSON clear text
#[derive(Clone, PartialEq, Debug, Eq)]
pub(crate) struct PlainPersistentKeyStore {
    /// Path of the keystore on the filesystem.
    path: PathBuf,
}

impl PlainPersistentKeyStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

use core::fmt::Debug;
use core::hash::Hash;

/// Derive an address from the key info
pub trait AddressDerivator<Key> {
    fn as_address(&self) -> Key;
}
/// Provides the default key identifier required to lookup the key information
pub trait DefaultKey {
    fn default_key() -> Self;
}

impl DefaultKey for String {
    fn default_key() -> Self {
        String::from("default")
    }
}

/// `KeyStore` structure, this contains a set of `KeyInfos` indexed by address.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CrownJewels<K: Hash + PartialEq + Eq + Debug, I, P> {
    // XXX Not a fan of the design, it makes it difficult to partition.
    // XXX It'd be better to have a trait bound and have a wrapping encryption store
    // XXX that utilizes the plain one internally
    pub(crate) key_info: HashMap<K, I>, // String, KeyInfo
    pub(crate) plain: Option<PlainPersistentKeyStore>,
    pub(crate) encryption: Option<EncryptionOverlay>,
    _phantom: PhantomData<P>,
}

impl<AddrT, InfoT, PersistReprT> CrownJewels<AddrT, InfoT, PersistReprT>
where
    AddrT: Hash + PartialEq + Eq + Debug + Serialize + Clone + for<'de> Deserialize<'de>,
    InfoT: AddressDerivator<AddrT>
        + Debug
        + Clone
        + PartialEq
        + Eq
        + for<'k, 'p> From<(&'k AddrT, &'p PersistReprT)>,
    PersistReprT: Debug
        + Clone
        + Serialize
        + for<'de> Deserialize<'de>
        + for<'k, 'i> From<(&'k AddrT, &'i InfoT)>,
    // XXX consider `&I: Into<K>` to allow for conversion of the in mem key to the address,
    // XXX which _should_ be deterministic based on the public key
{
    /// Create a new instance based on the configuration
    pub fn new(config: KeyStoreConfig) -> Result<Self, WalletErr> {
        match config {
            KeyStoreConfig::InMemory => Ok(Self {
                key_info: HashMap::new(),
                plain: None,
                encryption: None,
                _phantom: Default::default(),
            }),
            KeyStoreConfig::Plain { path: location } => {
                match File::open(&location) {
                    Ok(file) => {
                        let reader = BufReader::new(file);

                        // Existing cleartext JSON keystore
                        let persisted_key_info: HashMap<AddrT, PersistReprT> =
                            serde_json::from_reader(reader)
                                .inspect_err(|_| {
                                    error!(
                                        "failed to deserialize keyfile, initializing new keystore at: {:?}",
                                        location
                                    );
                                })
                                .unwrap_or_default();

                        let mut key_info = HashMap::<AddrT, InfoT>::with_capacity(128);
                        for (addr, persist) in persisted_key_info.iter() {
                            let info = <InfoT as From<_>>::from((addr, persist));
                            // the addr here does not match the one returned from `Addressable::as_address` due to some prefix fun in `fvm`

                            key_info.insert(
                                addr.clone(),
                                info, // KeyInfo {
                                      //     private_key: BASE64_STANDARD
                                      //         .decode(value.private_key.clone())
                                      //         .map_err(|error| Error::Other(error.to_string()))?,
                                      //     key_type: value.key_type,
                                      // },
                            );
                        }

                        Ok(Self {
                            key_info,
                            plain: Some(PlainPersistentKeyStore::new(location)),
                            encryption: None,
                            _phantom: Default::default(),
                        })
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::NotFound {
                            debug!(
                                "Keystore does not exist, initializing new keystore at: {:?}",
                                location
                            );
                            Ok(Self {
                                key_info: HashMap::new(),
                                plain: Some(PlainPersistentKeyStore::new(location)),
                                encryption: None,
                                _phantom: Default::default(),
                            })
                        } else {
                            Err(WalletErr::IO(e))
                        }
                    }
                }
            }
            KeyStoreConfig::Encrypted { location, password } => {
                if !location.exists() {
                    create_dir(location.parent().unwrap())?;
                }

                if !location.exists() {
                    File::create(location.clone())?;
                }

                match File::open(&location) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        let mut buf = vec![];
                        let read_bytes = reader.read_to_end(&mut buf)?;

                        if read_bytes == 0 {
                            // New encrypted keystore if file exists but is zero bytes (i.e., touch)
                            debug!(
                                "Keystore does not exist, initializing new keystore at {:?}",
                                location
                            );

                            Ok(Self {
                                key_info: HashMap::new(),
                                plain: Some(PlainPersistentKeyStore::new(location)),
                                encryption: Some(EncryptionOverlay::new(&password)?),
                                _phantom: Default::default(),
                            })
                        } else {
                            // Existing encrypted keystore
                            // Split off data from prepended salt
                            let data = buf.split_off(RECOMMENDED_SALT_LEN);
                            let mut prev_salt = [0; RECOMMENDED_SALT_LEN];
                            prev_salt.copy_from_slice(&buf);

                            let overlay = EncryptionOverlay::new_with_salt(&password, prev_salt)
                                .map_err(|error| {
                                    error!("Failed to create key from passphrase");
                                    WalletErr::Other(error.to_string())
                                })?;

                            let decrypted_data = overlay
                                .decrypt(&data)
                                .map_err(|error| WalletErr::Other(error.to_string()))?;

                            let key_info: HashMap<AddrT, PersistReprT> =
                                serde_ipld_dagcbor::from_slice(&decrypted_data)
                                    .inspect_err(|_| {
                                        // TODO XXX this is bonkers
                                        error!("Failed to deserialize keyfile, initializing new");
                                    })
                                    .unwrap_or_default();

                            let key_info: HashMap<AddrT, InfoT> = HashMap::from_iter(
                                key_info
                                    .iter()
                                    .map(|(k, p)| (k.clone(), <InfoT as From<_>>::from((k, p)))),
                            );
                            Ok(Self {
                                key_info,
                                plain: Some(PlainPersistentKeyStore::new(location)),
                                encryption: Some(overlay),
                                _phantom: Default::default(),
                            })
                        }
                    }
                    Err(_) => {
                        debug!("Encrypted keystore does not exist, initializing new keystore");

                        Ok(Self {
                            key_info: HashMap::new(),
                            plain: Some(PlainPersistentKeyStore::new(location)),
                            encryption: Some(EncryptionOverlay::new(&password)?),
                            _phantom: Default::default(),
                        })
                    }
                }
            }
        }
    }

    /// Write an updated version of the keystore to disk
    pub fn flush(&self) -> Result<(), WalletErr> {
        match &self.plain {
            Some(persistent_keystore) => {
                let dir = persistent_keystore
                    .path
                    .parent()
                    .ok_or_else(|| WalletErr::Other("Invalid Path".to_string()))?;
                fs::create_dir_all(dir)?;
                let file = File::create(&persistent_keystore.path)?;

                // Restrict permissions on files containing private keys
                crate::perm::set_user_perm(&file)?;

                let mut writer = BufWriter::new(file);

                match &self.encryption {
                    Some(encrypted_keystore) => {
                        // Flush For EncryptedKeyStore
                        let key_info =
                            HashMap::<AddrT, PersistReprT>::from_iter(self.key_info.iter().map(
                                |(k, i)| (k.clone(), <PersistReprT as From<_>>::from((k, i))),
                            ));
                        let data = serde_ipld_dagcbor::to_vec(&key_info).map_err(|e| {
                            WalletErr::Other(format!("failed to serialize and write key info: {e}"))
                        })?;

                        let salt_vec = encrypted_keystore.salt.to_vec();
                        writer.write_all(&salt_vec)?;

                        let encrypted_data = encrypted_keystore.encrypt(&data)?;
                        writer.write_all(&encrypted_data)?;

                        Ok(())
                    }
                    None => {
                        let key_info = HashMap::<AddrT, PersistReprT>::from_iter(
                            self.key_info.iter().map(|(addr, info)| {
                                (addr.clone(), <PersistReprT as From<_>>::from((addr, info)))
                            }),
                        );

                        // Flush for PersistentKeyStore
                        serde_json::to_writer_pretty(writer, &key_info).map_err(|e| {
                            WalletErr::Other(format!("failed to serialize and write key info: {e}"))
                        })?;

                        Ok(())
                    }
                }
            }
            None => {
                // NoOp for MemKeyStore
                Ok(())
            }
        }
    }

    fn available_keys(&self) -> Vec<String> {
        Vec::from_iter(self.key_info.keys().map(|addr| self.addr_to_string(addr)))
    }

    fn addr_to_string(&self, addr: &AddrT) -> String {
        dbg!(format!("{addr:?}"))
    }

    /// Return all of the keys that are stored in the `KeyStore`
    pub fn list(&self) -> Vec<AddrT> {
        Vec::from_iter(self.key_info.keys().cloned())
    }

    /// Return `KeyInfo` that corresponds to a given key
    pub fn get(&self, addr: &AddrT) -> Result<InfoT, WalletErr> {
        self.key_info
            .get(addr)
            .cloned()
            .ok_or_else(|| WalletErr::KeyNotFound {
                key: self.addr_to_string(addr),
                available_keys: self.available_keys(),
            })
    }

    /// Save a key/`KeyInfo` pair to the `KeyStore`
    pub fn put(&mut self, addr: AddrT, info: InfoT) -> Result<(), WalletErr>
    where
        AddrT: DefaultKey,
    {
        if <AddrT as DefaultKey>::default_key() != addr {
            debug_assert_eq!(&info.as_address(), &addr);
        }
        if self.key_info.contains_key(&addr) {
            return Err(WalletErr::KeyExists {
                key: self.addr_to_string(&addr),
            });
        }
        self.key_info.insert(addr, info);

        if self.plain.is_some() {
            self.flush()?;
        }

        Ok(())
    }

    /// Set the default key as a delegate/copy to what is referenced
    /// by the existing `key` passed as argument
    pub fn set_default(&mut self, key: &AddrT) -> Result<(), WalletErr>
    where
        AddrT: DefaultKey,
    {
        let default_key = <AddrT as DefaultKey>::default_key();
        let info = self.get(key)?;
        let _ = self.remove(default_key.clone());
        self.put(default_key, info)?;
        Ok(())
    }

    /// Obtain the default key, if any. No default key is not an error
    pub fn get_default(&self) -> Result<Option<AddrT>, WalletErr>
    where
        AddrT: DefaultKey,
    {
        let default_key = <AddrT as DefaultKey>::default_key();
        let Ok(info) = self.get(&default_key) else {
            return Ok(None); // FIXME TODO retain the error type
        };
        Ok(Some(info.as_address()))
    }

    /// Remove the key and corresponding `KeyInfo` from the `KeyStore`
    pub fn remove(&mut self, key: AddrT) -> Result<InfoT, WalletErr> {
        let key_out = self.key_info.remove(&key).ok_or(WalletErr::KeyNotFound {
            key: self.addr_to_string(&key),
            available_keys: self.available_keys(),
        })?;

        if self.plain.is_some() {
            self.flush()?;
        }

        Ok(key_out)
    }
}
