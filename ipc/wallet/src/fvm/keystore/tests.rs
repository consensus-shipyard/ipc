use anyhow::*;
use base64::{prelude::BASE64_STANDARD, Engine};
use quickcheck_macros::quickcheck;

use super::*;
use crate::{
    json::{KeyInfoJson, KeyInfoJsonRef},
    wallet,
};

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
fn test_encrypt_message() -> Result<()> {
    let (_, private_key) = EncryptionOverlay::derive_key(PASSPHRASE, None)?;
    let message = "foo is coming";
    let ciphertext = EncryptionOverlay::encrypt(&private_key, message.as_bytes())?;
    let second_pass = EncryptionOverlay::encrypt(&private_key, message.as_bytes())?;
    ensure!(
        ciphertext != second_pass,
        "Ciphertexts use secure initialization vectors"
    );
    Ok(())
}

#[test]
fn test_decrypt_message() -> Result<()> {
    let (_, private_key) = EncryptionOverlay::derive_key(PASSPHRASE, None)?;
    let message = "foo is coming";
    let ciphertext = EncryptionOverlay::encrypt(&private_key, message.as_bytes())?;
    let plaintext = EncryptionOverlay::decrypt(&private_key, &ciphertext)?;
    ensure!(plaintext == message.as_bytes());
    Ok(())
}

#[test]
fn test_read_old_encrypted_keystore() -> Result<()> {
    let dir: PathBuf = "tests/keystore_encrypted_old".into();
    ensure!(dir.exists());
    let ks = KeyStore::new(KeyStoreConfig::encrypted(dir, PASSPHRASE))?;
    ensure!(ks.plain.is_some());
    Ok(())
}

#[test]
fn test_read_write_encrypted_keystore() -> Result<()> {
    let keystore_location = tempfile::tempdir()?.into_path();
    let ks = KeyStore::new(KeyStoreConfig::encrypted(
        &keystore_location,
        PASSPHRASE,
    ))?;
    ks.flush()?;

    let ks_read = KeyStore::new(KeyStoreConfig::encrypted(
        &keystore_location,
        PASSPHRASE,
    ))?;

    ensure!(ks == ks_read);

    Ok(())
}

#[test]
fn test_read_write_keystore() -> Result<()> {
    let keystore_location = tempfile::tempdir()?.into_path();
    let mut ks = KeyStore::new(KeyStoreConfig::plain(&keystore_location))?;

    let key = wallet::generate_key(SignatureType::BLS)?;

    let addr = format!("wallet-{}", key.address);
    ks.put(addr.clone(), key.key_info)?;
    ks.flush().unwrap();

    let default = ks.get(&addr).unwrap();

    // Manually parse keystore.json
    let keystore_file = keystore_location.join(PLAIN_JSON_KEYSTORE_NAME);
    let reader = BufReader::new(File::open(keystore_file)?);
    let persisted_keystore: HashMap<String, PersistentKeyInfo> =
        serde_json::from_reader(reader)?;

    let default_key_info = persisted_keystore.get(&addr).unwrap();
    let actual = BASE64_STANDARD.decode(default_key_info.private_key.clone())?;

    assert_eq!(
        default.private_key, actual,
        "persisted key matches key from key store"
    );

    // Read existing keystore.json
    let ks_read = KeyStore::new(KeyStoreConfig::plain(keystore_location))?;
    ensure!(ks == ks_read);

    Ok(())
}

impl quickcheck::Arbitrary for KeyInfo {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let sigtype = g
            .choose(&[
                fvm_shared::crypto::signature::SignatureType::BLS,
                fvm_shared::crypto::signature::SignatureType::Secp256k1,
            ])
            .unwrap();
        KeyInfo {
            key_type: *sigtype,
            private_key: Vec::arbitrary(g),
        }
    }
}

#[quickcheck]
fn keyinfo_roundtrip(keyinfo: KeyInfo) {
    let serialized: String = serde_json::to_string(&KeyInfoJsonRef(&keyinfo)).unwrap();
    let parsed: KeyInfoJson = serde_json::from_str(&serialized).unwrap();
    assert_eq!(keyinfo, parsed.0);
}
