use crate::evm::EvmKeyInfo;
use crate::{AddressDerivator, CrownJewels, DefaultKey, KeyStoreConfig};
use std::convert::Infallible;
use std::default;
use std::fmt::{Display, Formatter};

use super::key::EvmPersistentKeyInfo;

#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct Key {
    data: String,
}
impl DefaultKey for Key {
    fn default_key() -> Self {
        <Self as Default>::default()
    }
}
impl AddressDerivator for EvmKeyInfo {
    type Key = Key;
    fn as_address(&self) -> Self::Key {
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
        EvmKeyInfo { private_key: sk  }
    }
}


impl Default for Key {
    fn default() -> Self {
        Self {
            data: String::from("default-key"),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

type EvmCrownJewels = CrownJewels<String, EvmKeyInfo, EvmPersistentKeyInfo>;
type EvmCrownJewelsTest = CrownJewels<Key, EvmKeyInfo, EvmPersistentKeyInfo>;

#[test]
fn test_read_write_keystore() {
    let keystore_folder = tempfile::tempdir().unwrap().into_path();
    let keystore_location = keystore_folder.join("eth_keystore");

    let mut ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(&keystore_location)).unwrap();

    let key_info = EvmKeyInfo {
        private_key: vec![0, 1, 2],
    };
    let addr = key_info.as_address();

    ks.put(key_info.as_address(), key_info.clone()).unwrap();

    let key_from_store = ks.get(&addr).unwrap();
    assert_eq!(key_from_store, key_info);

    // Create the key store again
    let ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(&keystore_location)).unwrap();
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
    let addr = Key::try_from(&key_info).unwrap();

    // can't set default if the key hasn't been put yet.
    assert!(ks.set_default(&addr).is_err());
    ks.put(addr.clone(), key_info.clone()).unwrap();
    ks.set_default(&addr).unwrap();
    assert_eq!(ks.get_default().unwrap(), Some(addr.clone()));

    // set other default
    let new_key = EvmKeyInfo {
        private_key: vec![0, 1, 3],
    };
    let new_addr = Key::try_from(&new_key).unwrap();
    ks.put(new_addr.clone(), new_key.clone()).unwrap();
    ks.set_default(&new_addr).unwrap();
    assert_eq!(ks.get_default().unwrap(), Some(new_addr.clone()));

    // Create the key store again

    let mut ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(&keystore_location)).unwrap();
    let key_from_store = ks.get(&addr).unwrap();
    assert_eq!(key_from_store, key_info);
    let key_from_store = ks.get(&Key::default()).unwrap();
    // the default is also recovered from persistent storage
    assert_eq!(ks.get_default().unwrap().unwrap(), new_addr);
}
