// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use encryption_overlay::EncryptionOverlay;
use fs::{create_dir, File};
use fs_err as fs;
use std::{
    fmt::Display,
    io::{BufReader, BufWriter, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use ahash::{HashMap, HashMapExt};
use argon2::{
    password_hash::SaltString, Argon2, ParamsBuilder, PasswordHasher, RECOMMENDED_SALT_LEN,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use fvm_shared::crypto::signature::SignatureType;
use log::{debug, error};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xsalsa20poly1305::{
    aead::{generic_array::GenericArray, Aead},
    KeyInit, XSalsa20Poly1305, NONCE_SIZE,
};

use super::errors::Error;

pub const PLAIN_KEYSTORE_NAME: &str = "keystore.json";
pub const ENCRYPTED_KEYSTORE_NAME: &str = "keystore";

/// Environmental variable which holds the `KeyStore` encryption phrase.
pub const FOREST_KEYSTORE_PHRASE_ENV: &str = "FOREST_KEYSTORE_PHRASE";

#[cfg(test)]
mod tests;

pub mod encryption_overlay;
pub mod json;
pub mod types;

pub use types::*;

/// `KeyStore` structure, this contains a set of `KeyInfos` indexed by address.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct KeyStore {
    key_info: HashMap<String, KeyInfo>,
    plain: Option<PlainPersistentKeyStore>,
    encryption_overlay: Option<EncryptionOverlay>,
}

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
    pub fn plain(path: impl AsRef<Path>) -> Self {
        Self::Plain {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub fn encrypted(path: impl AsRef<Path>, password: impl Into<String>) -> Self {
        Self::Encrypted {
            location: path.as_ref().to_path_buf(),
            password: password.into(),
        }
    }
}

/// Plain persistent `KeyStore` in JSON clear text in [`PLAIN_KEYSTORE_LOCATION`]
#[derive(Clone, PartialEq, Debug, Eq)]
struct PlainPersistentKeyStore {
    /// Path of the keystore on the filesystem.
    path: PathBuf,
}

impl KeyStore {
    pub fn new(config: KeyStoreConfig) -> Result<Self, Error> {
        match config {
            KeyStoreConfig::InMemory => Ok(Self {
                key_info: HashMap::new(),
                plain: None,
                encryption_overlay: None,
            }),
            KeyStoreConfig::Plain { path: location } => {
                let file_path = location.join(PLAIN_KEYSTORE_NAME);

                match File::open(&file_path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);

                        // Existing cleartext JSON keystore
                        let persisted_key_info: HashMap<String, PersistentKeyInfo> =
                            serde_json::from_reader(reader)
                                .inspect_err(|_| {
                                    error!(
                                        "failed to deserialize keyfile, initializing new keystore at: {:?}",
                                        file_path
                                    );
                                })
                                .unwrap_or_default();

                        let mut key_info = HashMap::new();
                        for (key, value) in persisted_key_info.iter() {
                            key_info.insert(
                                key.to_string(),
                                KeyInfo {
                                    private_key: BASE64_STANDARD
                                        .decode(value.private_key.clone())
                                        .map_err(|error| Error::Other(error.to_string()))?,
                                    key_type: value.key_type,
                                },
                            );
                        }

                        Ok(Self {
                            key_info,
                            plain: Some(PlainPersistentKeyStore { path: file_path }),
                            encryption_overlay: None,
                        })
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::NotFound {
                            debug!(
                                "Keystore does not exist, initializing new keystore at: {:?}",
                                file_path
                            );
                            Ok(Self {
                                key_info: HashMap::new(),
                                plain: Some(PlainPersistentKeyStore { path: file_path }),
                                encryption_overlay: None,
                            })
                        } else {
                            Err(Error::Other(e.to_string()))
                        }
                    }
                }
            }
            KeyStoreConfig::Encrypted { location, password } => {
                if !location.exists() {
                    create_dir(location.clone())?;
                }

                let file_path = location.join(Path::new(ENCRYPTED_KEYSTORE_NAME));

                if !file_path.exists() {
                    File::create(file_path.clone())?;
                }

                match File::open(&file_path) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        let mut buf = vec![];
                        let read_bytes = reader.read_to_end(&mut buf)?;

                        if read_bytes == 0 {
                            // New encrypted keystore if file exists but is zero bytes (i.e., touch)
                            debug!(
                                "Keystore does not exist, initializing new keystore at {:?}",
                                file_path
                            );

                            let overlay = EncryptionOverlay::with_random_salt(&password).map_err(
                                |error| {
                                    error!("Failed to create key from passphrase");
                                    Error::Other(error.to_string())
                                },
                            )?;
                            Ok(Self {
                                key_info: HashMap::new(),
                                plain: Some(PlainPersistentKeyStore { path: file_path }),
                                encryption_overlay: Some(overlay),
                            })
                        } else {
                            // Existing encrypted keystore
                            // Split off data from prepended salt
                            let data = buf.split_off(RECOMMENDED_SALT_LEN);
                            let mut prev_salt = [0; RECOMMENDED_SALT_LEN];
                            prev_salt.copy_from_slice(&buf);
                            let overlay = EncryptionOverlay::from_salt(&password, prev_salt)
                                .map_err(|error| {
                                    error!("Failed to create key from passphrase");
                                    Error::Other(error.to_string())
                                })?;

                            let decrypted_data = overlay
                                .decrypt(&data)
                                .map_err(|error| Error::Other(error.to_string()))?;

                            let key_info = serde_ipld_dagcbor::from_slice(&decrypted_data)
                                .inspect_err(|_| {
                                    error!("Failed to deserialize keyfile, initializing new");
                                })
                                .unwrap_or_default();

                            Ok(Self {
                                key_info,
                                plain: Some(PlainPersistentKeyStore { path: file_path }),
                                encryption_overlay: Some(overlay),
                            })
                        }
                    }
                    Err(_) => {
                        debug!("Encrypted keystore does not exist, initializing new keystore");

                        let overlay =
                            EncryptionOverlay::with_random_salt(&password).map_err(|error| {
                                error!("Failed to create key from passphrase");
                                Error::Other(error.to_string())
                            })?;

                        Ok(Self {
                            key_info: HashMap::new(),
                            plain: Some(PlainPersistentKeyStore { path: file_path }),
                            encryption_overlay: Some(overlay),
                        })
                    }
                }
            }
        }
    }

    /// Write an updated version of the keystore to disk
    pub fn flush(&self) -> anyhow::Result<()> {
        match &self.plain {
            Some(persistent_keystore) => {
                let dir = persistent_keystore
                    .path
                    .parent()
                    .ok_or_else(|| Error::Other("Invalid Path".to_string()))?;
                fs::create_dir_all(dir)?;
                let file = File::create(&persistent_keystore.path)?;

                // Restrict permissions on files containing private keys
                #[cfg(unix)]
                crate::utils::set_user_perm(&file)?;

                let mut writer = BufWriter::new(file);

                match &self.encryption_overlay {
                    Some(overlay) => {
                        // Flush For EncryptedKeyStore
                        let data = serde_ipld_dagcbor::to_vec(&self.key_info).map_err(|e| {
                            Error::Other(format!("failed to serialize and write key info: {e}"))
                        })?;

                        let encrypted_data = overlay.encrypt(&data)?;
                        let mut salt_vec = overlay.salt.to_vec();
                        salt_vec.extend(encrypted_data);
                        writer.write_all(&salt_vec)?;

                        Ok(())
                    }
                    None => {
                        let mut key_info: HashMap<String, PersistentKeyInfo> = HashMap::new();
                        for (key, value) in self.key_info.iter() {
                            key_info.insert(
                                key.to_string(),
                                PersistentKeyInfo {
                                    private_key: BASE64_STANDARD.encode(value.private_key.clone()),
                                    key_type: value.key_type,
                                },
                            );
                        }

                        // Flush for PersistentKeyStore
                        serde_json::to_writer_pretty(writer, &key_info).map_err(|e| {
                            Error::Other(format!("failed to serialize and write key info: {e}"))
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

    /// Return all of the keys that are stored in the `KeyStore`
    pub fn list(&self) -> Vec<String> {
        self.key_info.keys().cloned().collect()
    }

    /// Return `KeyInfo` that corresponds to a given key
    pub fn get(&self, k: &str) -> Result<KeyInfo, Error> {
        self.key_info.get(k).cloned().ok_or(Error::KeyInfo)
    }

    /// Save a key/`KeyInfo` pair to the `KeyStore`
    pub fn put(&mut self, key: String, key_info: KeyInfo) -> Result<(), Error> {
        if self.key_info.contains_key(&key) {
            return Err(Error::KeyExists);
        }
        self.key_info.insert(key, key_info);

        if self.plain.is_some() {
            self.flush().map_err(|err| Error::Other(err.to_string()))?;
        }

        Ok(())
    }

    /// Remove the key and corresponding `KeyInfo` from the `KeyStore`
    pub fn remove(&mut self, key: String) -> anyhow::Result<KeyInfo> {
        let key_out = self.key_info.remove(&key).ok_or(Error::KeyInfo)?;

        if self.plain.is_some() {
            self.flush()?;
        }

        Ok(key_out)
    }
}

fn map_err_to_anyhow<T: Display>(e: T) -> anyhow::Error {
    anyhow::Error::msg(e.to_string())
}
