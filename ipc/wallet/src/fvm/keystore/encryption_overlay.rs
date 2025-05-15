// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use super::*;

use anyhow::{bail, Result};

/// Encrypted overlay for the `KeyStore` in [`ENCRYPTED_KEYSTORE_LOCATION`]
///
/// Uses `Argon2id` as hash key derivation
/// and algorithm `XSalsa20Poly1305` authenticated encryption
/// and CBOR as data format encoding.
#[derive(Clone, PartialEq, Debug, Eq)]
pub(crate) struct EncryptionOverlay {
    pub(crate) salt: SaltByteArray,
    // pre-hashed key
    pub(crate) encryption_key: Vec<u8>,
}

impl EncryptionOverlay {
    /// Derive the actual encryption key utilizing a random salt value
    pub fn with_random_salt(passphrase: &str) -> Result<Self> {
        Self::derive_key(passphrase.as_ref(), None)
    }

    /// Derive the actual encryption key utlizing the provided "salt" value
    pub fn from_salt(passphrase: impl AsRef<str>, salt: impl Into<SaltByteArray>) -> Result<Self> {
        Self::derive_key(passphrase.as_ref(), Some(salt.into()))
    }

    fn derive_key(passphrase: &str, prev_salt: Option<SaltByteArray>) -> Result<Self> {
        let salt = match prev_salt {
            Some(prev_salt) => prev_salt,
            None => {
                let mut salt = [0; RECOMMENDED_SALT_LEN];
                OsRng.fill_bytes(&mut salt);
                salt
            }
        };

        let mut param_builder = ParamsBuilder::new();
        // #define crypto_pwhash_argon2id_MEMLIMIT_INTERACTIVE 67108864U
        // see <https://github.com/jedisct1/libsodium/blob/089f850608737f9d969157092988cb274fe7f8d4/src/libsodium/include/sodium/crypto_pwhash_argon2id.h#L70>
        const CRYPTO_PWHASH_ARGON2ID_MEMLIMIT_INTERACTIVE: u32 = 67108864;
        // #define crypto_pwhash_argon2id_OPSLIMIT_INTERACTIVE 2U
        // see <https://github.com/jedisct1/libsodium/blob/089f850608737f9d969157092988cb274fe7f8d4/src/libsodium/include/sodium/crypto_pwhash_argon2id.h#L66>
        const CRYPTO_PWHASH_ARGON2ID_OPSLIMIT_INTERACTIVE: u32 = 2;
        param_builder
            .m_cost(CRYPTO_PWHASH_ARGON2ID_MEMLIMIT_INTERACTIVE / 1024)
            .t_cost(CRYPTO_PWHASH_ARGON2ID_OPSLIMIT_INTERACTIVE);
        // https://docs.rs/sodiumoxide/latest/sodiumoxide/crypto/secretbox/xsalsa20poly1305/constant.KEYBYTES.html
        // KEYBYTES = 0x20
        // param_builder.output_len(32)?;
        let hasher = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            param_builder.build().map_err(map_err_to_anyhow)?,
        );
        let salt_string = SaltString::encode_b64(&salt).map_err(map_err_to_anyhow)?;
        let pw_hash = hasher
            .hash_password(passphrase.as_bytes(), &salt_string)
            .map_err(map_err_to_anyhow)?;
        if let Some(hash) = pw_hash.hash {
            Ok(Self {
                salt,
                encryption_key: hash.as_bytes().to_vec(),
            })
        } else {
            bail!(EncryptedKeyStoreError::EncryptionError)
        }
    }

    pub(crate) fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let mut nonce = [0; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        let nonce = GenericArray::from_slice(&nonce);
        let key = GenericArray::from_slice(&self.encryption_key);
        let cipher = XSalsa20Poly1305::new(key);
        let mut ciphertext = cipher.encrypt(nonce, msg).map_err(map_err_to_anyhow)?;
        ciphertext.extend(nonce.iter());
        Ok(ciphertext)
    }

    pub(crate) fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let cyphertext_len = msg.len() - NONCE_SIZE;
        let ciphertext = &msg[..cyphertext_len];
        let nonce = GenericArray::from_slice(&msg[cyphertext_len..]);
        let key = GenericArray::from_slice(&self.encryption_key);
        let cipher = XSalsa20Poly1305::new(key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(map_err_to_anyhow)?;
        Ok(plaintext)
    }
}
