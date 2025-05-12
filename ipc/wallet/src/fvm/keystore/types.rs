use super::*;

pub(crate) type SaltByteArray = [u8; RECOMMENDED_SALT_LEN];


#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum EncryptedKeyStoreError {
    /// Possibly indicates incorrect passphrase
    #[error("Error decrypting data, is the password correct?")]
    DecryptionError,
    /// An error occurred while encrypting keys
    #[error("Error encrypting data")]
    EncryptionError,
    /// Unlock called without `encrypted_keystore` being enabled in
    /// `config.toml`
    #[error("Error with forest configuration")]
    ConfigurationError,
}

// TODO need to update keyinfo to not use SignatureType, use string instead to
// save keys like jwt secret
/// `KeyInfo` structure, this contains the type of key (stored as a string) and
/// the private key. Note how the private key is stored as a byte vector
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct KeyInfo {
    pub(crate) key_type: SignatureType,
    // Vec<u8> is used because The private keys for BLS and SECP256K1 are not of the same type
    pub(crate) private_key: Vec<u8>,
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct PersistentKeyInfo {
    pub(crate) key_type: SignatureType,
    pub(crate) private_key: String,
}

impl KeyInfo {
    /// Return a new `KeyInfo` given the key type and private key
    pub fn new(key_type: SignatureType, private_key: Vec<u8>) -> Self {
        KeyInfo {
            key_type,
            private_key,
        }
    }

    /// Return a reference to the key's signature type
    pub fn key_type(&self) -> &SignatureType {
        &self.key_type
    }

    /// Return a reference to the private key
    pub fn private_key(&self) -> &Vec<u8> {
        &self.private_key
    }
}
