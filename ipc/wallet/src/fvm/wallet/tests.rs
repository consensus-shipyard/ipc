// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::blake2b_256;
use assert_matches::assert_matches;
use libsecp256k1::{Message as SecpMessage, SecretKey as SecpPrivate};

use super::*;
use crate::CrownJewels;
use crate::{generate, KeyStoreConfig};

fn construct_priv_keys() -> Vec<FullKey> {
    let mut secp_keys = Vec::new();
    let mut bls_keys = Vec::new();
    for _ in 1..5 {
        let secp_priv_key = generate(SignatureType::Secp256k1).unwrap();
        let secp_key_info = FvmKeyInfo::new(SignatureType::Secp256k1, secp_priv_key);
        let secp_key = FullKey::try_from(secp_key_info).unwrap();
        secp_keys.push(secp_key);

        let bls_priv_key = generate(SignatureType::BLS).unwrap();
        let bls_key_info = FvmKeyInfo::new(SignatureType::BLS, bls_priv_key);
        let bls_key = FullKey::try_from(bls_key_info).unwrap();
        bls_keys.push(bls_key);
    }

    secp_keys.append(bls_keys.as_mut());
    secp_keys
}

fn generate_wallet() -> Wallet {
    let key_vec = construct_priv_keys();
    Wallet::new_from_keys(CrownJewels::new(KeyStoreConfig::InMemory).unwrap(), key_vec)
}

#[test]
fn contains_key() {
    let key_vec = construct_priv_keys();
    let found_key = key_vec[0].clone();
    let addr = key_vec[0].address;

    let mut wallet =
        Wallet::new_from_keys(CrownJewels::new(KeyStoreConfig::InMemory).unwrap(), key_vec);

    // make sure that this address resolves to the right key
    assert_eq!(wallet.find_key(&addr).unwrap(), found_key);
    // make sure that has_key returns true as well
    assert!(wallet.has_key(&addr));

    let new_priv_key = generate(SignatureType::BLS).unwrap();
    let pub_key = helpers::to_public(SignatureType::BLS, new_priv_key.as_slice()).unwrap();
    let address = Address::new_bls(pub_key.as_slice()).unwrap();

    // test to see if the new key has been created and added to the wallet
    assert!(!wallet.has_key(&address));
    // test to make sure that the newly made key cannot be added to the wallet
    // because it is not found in the keystore
    assert_matches!(
        wallet.find_key(&address).unwrap_err(),
        WalletErr::KeyNotFound { .. }
    );
    // sanity check to make sure that the key has not been added to the wallet
    assert!(!wallet.has_key(&address));
}

#[test]
fn sign() {
    let key_vec = construct_priv_keys();
    let priv_key_bytes = key_vec[2].key_info.private_key();
    let addr = key_vec[2].address;

    let keystore = CrownJewels::new(KeyStoreConfig::InMemory).unwrap();
    let mut wallet = Wallet::new_from_keys(keystore, key_vec.clone());
    let msg = [0u8; 64];

    let msg_sig = wallet.sign(&addr, &msg).unwrap();

    let msg_complete = blake2b_256(&msg);
    let message = SecpMessage::parse(&msg_complete);
    let priv_key = SecpPrivate::parse_slice(priv_key_bytes).unwrap();
    let (sig, recovery_id) = libsecp256k1::sign(&message, &priv_key);
    let mut new_bytes = [0; 65];
    new_bytes[..64].copy_from_slice(&sig.serialize());
    new_bytes[64] = recovery_id.serialize();
    let actual = Signature::new_secp256k1(new_bytes.to_vec());
    assert_eq!(msg_sig, actual)
}

#[test]
fn import_export() -> anyhow::Result<()> {
    let key_vec = construct_priv_keys();
    let key = key_vec[0].clone();
    let keystore = CrownJewels::new(KeyStoreConfig::InMemory).unwrap();
    let mut wallet = Wallet::new_from_keys(keystore, key_vec);

    let key_info = wallet.export(&key.address).unwrap();
    // test to see if export returns the correct key_info
    assert_eq!(key_info, key.key_info);

    let new_priv_key = generate(SignatureType::Secp256k1).unwrap();
    let pub_key = helpers::to_public(SignatureType::Secp256k1, new_priv_key.as_slice()).unwrap();
    let test_addr = Address::new_secp256k1(pub_key.as_slice()).unwrap();
    let key_info_err = wallet.export(&test_addr).unwrap_err();
    // test to make sure that an error is raised when an incorrect address is added
    assert_matches!(key_info_err, WalletErr::KeyNotFound { .. });

    let test_key_info = FvmKeyInfo::new(SignatureType::Secp256k1, new_priv_key);
    // make sure that key_info has been imported to wallet
    assert!(wallet.import(test_key_info.clone()).is_ok());

    // XXX behaviour change, we do not error on import for duplicate keys anymore!
    assert_matches!(wallet.import(test_key_info), Ok(_));
    Ok(())
}

#[test]
fn list_addr() {
    let key_vec = construct_priv_keys();
    let mut addr_string_vec = Vec::new();

    let mut key_store = CrownJewels::new(KeyStoreConfig::InMemory).unwrap();

    for i in &key_vec {
        addr_string_vec.push(i.address.to_string());

        let wrapped = WrappedFvmAddress::from(&i.address);
        key_store.put(wrapped, i.key_info.clone()).unwrap();
    }

    addr_string_vec.sort();

    let mut addr_vec = Vec::new();

    for addr in addr_string_vec {
        addr_vec.push(Address::from_str(addr.as_str()).unwrap())
    }

    let wallet = Wallet::new(key_store);

    let test_addr_vec = wallet.list_addrs().unwrap();

    // check to see if the addrs in wallet are the same as the key_vec before it was
    // added to the wallet
    assert_eq!(test_addr_vec, addr_vec);
}

#[test]
fn generate_new_key() {
    let mut wallet = generate_wallet();
    let addr = wallet.generate_addr(SignatureType::BLS).unwrap();
    let key = wallet
        .keystore
        .get(&WrappedFvmAddress::default_key())
        .unwrap();
    // make sure that the newly generated key is the default key - checking by key
    // type
    assert_matches!(key.key_type(), SignatureType::BLS);

    let wrapped = WrappedFvmAddress::from(&addr);

    let key_info = wallet.keystore.get(&wrapped).unwrap();
    let key = wallet.in_memory_cache.get(&addr).unwrap();

    // these assertions will make sure that the key has actually been added to the
    // wallet
    assert_matches!(key_info.key_type(), SignatureType::BLS);
    assert_eq!(key.address, addr);
}

#[test]
fn get_set_default() {
    let key_store = CrownJewels::new(KeyStoreConfig::InMemory).unwrap();
    let mut wallet = Wallet::new(key_store);
    // check to make sure that there is no default
    assert_matches!(
        wallet.get_default().unwrap_err(),
        WalletErr::KeyNotFound { .. }
    );

    let new_priv_key = generate(SignatureType::Secp256k1).unwrap();
    let pub_key = helpers::to_public(SignatureType::Secp256k1, new_priv_key.as_slice()).unwrap();
    let test_addr = Address::new_secp256k1(pub_key.as_slice()).unwrap();

    let key_info = FvmKeyInfo::new(SignatureType::Secp256k1, new_priv_key);
    let test_addr_string = WrappedFvmAddress::from(&test_addr);

    wallet.keystore.put(test_addr_string, key_info).unwrap();

    // check to make sure that the set_default function completed without error
    assert!(wallet.set_default(test_addr).is_ok());

    // check to make sure that the test_addr is actually the default addr for the
    // wallet
    assert_eq!(wallet.get_default().unwrap(), test_addr);
}

#[test]
fn secp_verify() {
    let secp_priv_key = generate(SignatureType::Secp256k1).unwrap();
    let secp_key_info = FvmKeyInfo::new(SignatureType::Secp256k1, secp_priv_key);
    let secp_key = FullKey::try_from(secp_key_info).unwrap();
    let addr = secp_key.address;
    let key_store = CrownJewels::new(KeyStoreConfig::InMemory).unwrap();
    let mut wallet = Wallet::new_from_keys(key_store, vec![secp_key]);

    let msg = [0u8; 64];

    let sig = wallet.sign(&addr, &msg).unwrap();
    sig.verify(&msg, &addr).unwrap();

    // invalid verify check
    let invalid_addr = wallet.generate_addr(SignatureType::Secp256k1).unwrap();
    assert!(sig.verify(&msg, &invalid_addr).is_err())
}

#[test]
fn bls_verify_test() {
    let bls_priv_key = generate(SignatureType::BLS).unwrap();
    let bls_key_info = FvmKeyInfo::new(SignatureType::BLS, bls_priv_key);
    let bls_key = FullKey::try_from(bls_key_info).unwrap();
    let addr = bls_key.address;
    let key_store = CrownJewels::new(KeyStoreConfig::InMemory).unwrap();
    let mut wallet = Wallet::new_from_keys(key_store, vec![bls_key]);

    let msg = [0u8; 64];

    let sig = wallet.sign(&addr, &msg).unwrap();
    sig.verify(&msg, &addr).unwrap();

    // invalid verify check
    let invalid_addr = wallet.generate_addr(SignatureType::BLS).unwrap();
    assert!(sig.verify(&msg, &invalid_addr).is_err())
}
