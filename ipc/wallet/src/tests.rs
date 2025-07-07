// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::*;
use base64::{prelude::BASE64_STANDARD, Engine};
use fvm_shared::crypto::signature::SignatureType;
use quickcheck_macros::quickcheck;

use super::*;

use fs_err as fs;
use std::collections::HashMap;
use std::io::BufReader;

mod encryption {
    use std::path::Path;

    use super::*;
    const PASSPHRASE: &str = "foobarbaz";

    #[test]
    fn test_generate_key() {
        let (salt, encryption_key) = EncryptionOverlay::derive_key(PASSPHRASE, None).unwrap();
        let (second_salt, second_key) =
            EncryptionOverlay::derive_key(PASSPHRASE, Some(salt)).unwrap();

        assert_eq!(
            encryption_key, second_key,
            "Derived key must be deterministic"
        );
        assert_eq!(salt, second_salt, "Salts must match");
    }

    #[test]
    fn iv_vector_works() -> Result<()> {
        let overlay = EncryptionOverlay::new(PASSPHRASE)?;
        let message = "foo is coming";
        let ciphertext = overlay.encrypt(message.as_bytes())?;
        let second_pass = overlay.encrypt(message.as_bytes())?;
        ensure!(
            ciphertext != second_pass,
            "Ciphertexts use secure initialization vectors"
        );
        Ok(())
    }

    #[test]
    fn encrypt_decrypt_message_works() -> Result<()> {
        let overlay = EncryptionOverlay::new(PASSPHRASE)?;
        let message = "foo is coming";
        let ciphertext = overlay.encrypt(message.as_bytes())?;
        let plaintext = overlay.decrypt(&ciphertext)?;
        ensure!(plaintext == message.as_bytes());
        Ok(())
    }

    #[test]
    fn fvm_regression_read_old_encrypted_keystore() -> Result<()> {
        let loco = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/keystore_encrypted_old/keystore"
        ));
        ensure!(loco.exists());
        let ks = FvmCrownJewels::new(KeyStoreConfig::encrypted(loco, PASSPHRASE))?;
        ensure!(ks.plain.is_some());
        Ok(())
    }

    #[test]
    fn fvm_read_write_encrypted_keystore() -> Result<()> {
        let keystore_location = tempfile::NamedTempFile::new()?;
        let keystore_location = keystore_location.path();

        // create a new one, it doesn't exist yet
        let ks = FvmCrownJewels::new(KeyStoreConfig::encrypted(keystore_location, PASSPHRASE))?;
        ks.flush()?;

        // read the existing one
        let ks_read =
            FvmCrownJewels::new(KeyStoreConfig::encrypted(keystore_location, PASSPHRASE))?;

        ensure!(ks == ks_read);

        Ok(())
    }

    #[test]
    fn fvm_read_write_keystore() -> Result<()> {
        let named = tempfile::NamedTempFile::new()?;
        let keystore_location = dbg!(named.path());
        // let (_keystore_f, ref keystore_location) = named.keep()?;

        let mut ks = FvmCrownJewels::new(KeyStoreConfig::plain(keystore_location))?;

        let key = wallet::generate_key(SignatureType::Secp256k1)?;

        let addr = format!("wallet-{}", key.address);
        ks.put(addr.clone(), key.key_info)?;
        ks.flush().unwrap();

        let default = ks.get(&addr).unwrap();

        // Manually parse keystore.json
        let reader = BufReader::new(fs::File::open(keystore_location)?);
        let persisted_keystore: HashMap<String, PersistentKeyInfo> =
            dbg!(serde_json::from_reader(reader))?;

        let default_key_info = persisted_keystore.get(dbg!(&addr)).unwrap();
        let actual = BASE64_STANDARD.decode(dbg!(default_key_info.private_key.clone()))?;

        assert_eq!(
            default.private_key, actual,
            "persisted key matches key from key store"
        );

        // Read existing keystore.json
        let ks_read = CrownJewels::new(KeyStoreConfig::plain(keystore_location))?;
        ensure!(ks == ks_read);

        let _ = keystore_location;
        Ok(())
    }

    impl quickcheck::Arbitrary for FvmKeyInfo {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let sigtype = g
                .choose(&[fvm_shared::crypto::signature::SignatureType::Secp256k1])
                .unwrap();
            FvmKeyInfo {
                key_type: *sigtype,
                private_key: Vec::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn keyinfo_roundtrip(keyinfo: FvmKeyInfo) {
        let serialized: String = serde_json::to_string(&fvm::KeyInfoJsonRef(&keyinfo)).unwrap();
        let parsed: fvm::KeyInfoJson = serde_json::from_str(&serialized).unwrap();
        assert_eq!(keyinfo, parsed.0);
    }
}
