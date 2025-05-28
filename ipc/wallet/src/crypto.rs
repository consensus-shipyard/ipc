// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT
// use super::*;


pub const ENCRYPTED_KEYSTORE_NAME: &str = "keystore";

use argon2::password_hash::SaltString;
pub use argon2::RECOMMENDED_SALT_LEN;
use rand::rngs::OsRng;
use rand::RngCore;
use xsalsa20poly1305::{aead::generic_array::GenericArray, KeyInit, XSalsa20Poly1305, NONCE_SIZE};
use ahash::HashMapExt;
use argon2::PasswordHasher;
use xsalsa20poly1305::aead::Aead;

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Error decrypting data, is the password correct?")]
    DecryptionError,
    
    #[error("Error encrypting data")]
    EncryptionError,
    
    #[error("Error with forest configuration, ensure it's enabled in your `config.toml` with `encrypted_keystore` being set")]
    ConfigurationError,
    
    #[error(transparent)]
    Argon2(#[from] argon2::Error),

    // XXX TODO make variants per use, not goto catch-all-wrappers
    #[error(transparent)]
    PasswdHashing(#[from] argon2::password_hash::Error),
    
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[error(transparent)]
    Base64Encode(#[from] base64::EncodeSliceError),

    #[error(transparent)]
    Xsalsa(#[from] xsalsa20poly1305::Error),
}

pub(crate) type SaltByteArray = [u8; RECOMMENDED_SALT_LEN];

/// Encrypted overlay for the `KeyStore` in [`ENCRYPTED_KEYSTORE_LOCATION`]
/// 
/// Uses `Argon2id` as hash key derivation
/// and algorithm `XSalsa20Poly1305` authenticated encryption
/// and CBOR as data format encoding.
#[derive(Clone, PartialEq, Debug, Eq)]
pub(crate) struct EncryptionOverlay {
    pub(crate) salt: SaltByteArray,
    // pre-hashed key
    encryption_key: Vec<u8>,
}


impl EncryptionOverlay {
    pub(crate) fn new(passphrase: &str) -> Result<Self, CryptoError> {
        let (salt, encryption_key) = Self::derive_key(passphrase, None)?;
        Ok(Self {
            salt,
            encryption_key,
        })        
    }
    
    pub(crate) fn new_with_salt(passphrase: &str, salt: SaltByteArray) -> Result<Self, CryptoError> {
        let (salt, encryption_key) = Self::derive_key(passphrase, Some(salt))?;
        Ok(Self {
            salt,
            encryption_key,
        })
    }
    
    /// Use given salt and passphrase to derive the salt bytes array (cc) and actual encryption key
    /// 
    /// If the `prev_salt` is `None`, a new one will be generated using the OS provided RNG. 
    pub(crate) fn derive_key(
        passphrase: &str,
        prev_salt: Option<SaltByteArray>,
    ) -> Result<(SaltByteArray, Vec<u8>), CryptoError> {
        let salt = match prev_salt {
            Some(prev_salt) => prev_salt,
            None => {
                let mut salt = [0; RECOMMENDED_SALT_LEN];
                OsRng.fill_bytes(&mut salt);
                salt
            }
        };

        let mut param_builder = argon2::ParamsBuilder::new();
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
        let hasher = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            param_builder.build()?,
        );
        let salt_string = SaltString::encode_b64(&salt)?;
        let pw_hash = hasher
            .hash_password(passphrase.as_bytes(), &salt_string)?;
        if let Some(hash) = pw_hash.hash {
            Ok((salt, hash.as_bytes().to_vec()))
        } else {
            Err(CryptoError::EncryptionError)
        }
    }

    pub(crate) fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce = [0; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        let nonce = GenericArray::from_slice(&nonce);
        let key = GenericArray::from_slice(&self.encryption_key);
        let cipher = XSalsa20Poly1305::new(key);
        let mut ciphertext = cipher.encrypt(nonce, msg)?;
        ciphertext.extend(nonce.iter());
        Ok(ciphertext)
    }

    pub(crate) fn decrypt(&self, msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let cyphertext_len = msg.len() - NONCE_SIZE;
        let ciphertext = &msg[..cyphertext_len];
        let nonce = GenericArray::from_slice(&msg[cyphertext_len..]);
        let key = GenericArray::from_slice(&self.encryption_key);
        let cipher = XSalsa20Poly1305::new(key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}
