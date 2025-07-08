// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::evm::key::EvmKeyInfo;
use crate::evm::WrappedEthAddress;
use crate::{AddressDerivator, CrownJewels, DefaultKey, KeyStoreConfig};

use super::key::EvmPersistentKeyInfo;

type EvmCrownJewelsTest = CrownJewels<WrappedEthAddress, EvmKeyInfo, EvmPersistentKeyInfo>;

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
    let new_addr: WrappedEthAddress = new_key.as_address();
    ks.put(new_addr.clone(), new_key.clone()).unwrap();
    ks.set_default(&new_addr).unwrap();
    assert_eq!(ks.get_default().unwrap(), Some(new_addr.clone()));

    // Create the key store again

    let ks = EvmCrownJewelsTest::new(KeyStoreConfig::plain(&keystore_location)).unwrap();
    let key_from_store = ks.get(&addr).unwrap();
    assert_eq!(key_from_store, key_info);
    let _key_from_store = ks.get(&WrappedEthAddress::default_key()).unwrap();
    // the default is also recovered from persistent storage
    assert_eq!(ks.get_default().unwrap().unwrap(), new_addr);
}
