// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::evm::key::EvmKeyInfo;
use crate::{AddressDerivator, CrownJewels, DefaultKey, KeyStoreConfig};
use std::convert::Infallible;
use std::fmt::{Display, Formatter};

use super::key::EvmPersistentKeyInfo;

#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize)]
#[serde(from = "String")]
#[serde(into = "String")]
pub struct Key {
    data: String,
}

impl From<String> for Key {
    fn from(data: String) -> Self {
        Self { data }
    }
}

impl From<Key> for String {
    fn from(val: Key) -> Self {
        val.data
    }
}

impl DefaultKey for Key {
    fn default_key() -> Self {
        Self {
            data: "default-key".to_owned(),
        }
    }
}
impl AddressDerivator<Key> for EvmKeyInfo {
    fn as_address(&self) -> Key {
        Key::try_from(self).unwrap()
    }
}
impl TryFrom<&EvmKeyInfo> for Key {
    type Error = Infallible;

    fn try_from(value: &EvmKeyInfo) -> Result<Self, Self::Error> {
        Ok(Key {
            data: hex::encode(&value.private_key),
        })
    }
}

impl From<(&Key, &EvmKeyInfo)> for EvmPersistentKeyInfo {
    fn from(value: (&Key, &EvmKeyInfo)) -> Self {
        let sk = hex::encode(&value.1.private_key);
        let address = value.0.clone();
        EvmPersistentKeyInfo {
            private_key: sk,
            address: address.to_string(),
        }
    }
}
impl From<(&Key, &EvmPersistentKeyInfo)> for EvmKeyInfo {
    fn from(value: (&Key, &EvmPersistentKeyInfo)) -> Self {
        let sk = hex::decode(&value.1.private_key).expect("TODO");
        EvmKeyInfo { private_key: sk }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

type EvmCrownJewelsTest = CrownJewels<Key, EvmKeyInfo, EvmPersistentKeyInfo>;

#[test]
fn test_read_write_keystore() {
    let named = tempfile::NamedTempFile::new().unwrap();
    let keystore_location = named.path();

    let mut ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(keystore_location)).unwrap();

    let key_info = EvmKeyInfo {
        private_key: vec![0, 1, 2],
    };
    let addr = key_info.as_address();

    ks.put(key_info.as_address(), key_info.clone()).unwrap();

    let key_from_store = ks.get(&addr).unwrap();
    assert_eq!(key_from_store, key_info);

    // Create the key store again
    let ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(keystore_location)).unwrap();
    let key_from_store = ks.get(&addr).unwrap();
    assert_eq!(key_from_store, key_info);
}

#[test]
fn test_default() {
    let keystore_folder = tempfile::tempdir().unwrap().into_path();
    let keystore_location = keystore_folder.join("eth_keystore");

    let mut ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(&keystore_location)).unwrap();

    let key_info = EvmKeyInfo {
        private_key: vec![0, 1, 2],
    };
    let addr = key_info.as_address();

    // can't set default if the key hasn't been put yet.
    assert!(ks.set_default(&addr).is_err());
    let _ = ks.put(addr.clone(), key_info.clone());
    ks.set_default(&addr).unwrap();
    assert_eq!(ks.get_default().unwrap(), Some(addr.clone()));

    // set other default
    let new_key = EvmKeyInfo {
        private_key: vec![0, 1, 3],
    };
    let new_addr: Key = new_key.as_address();
    ks.put(new_addr.clone(), new_key.clone()).unwrap();
    ks.set_default(&new_addr).unwrap();
    assert_eq!(ks.get_default().unwrap(), Some(new_addr.clone()));

    // Create the key store again

    let ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(&keystore_location)).unwrap();
    let key_from_store = ks.get(&addr).unwrap();
    assert_eq!(key_from_store, key_info);
    let _key_from_store = ks.get(&Key::default_key()).unwrap();
    // the default is also recovered from persistent storage
    assert_eq!(ks.get_default().unwrap().unwrap(), new_addr);
}
