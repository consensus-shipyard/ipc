// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use fendermint_crypto::{from_b64, to_b64, PublicKey, SecretKey};
use fs_err as fs;
use std::path::Path;

pub fn secret_to_b64(sk: &SecretKey) -> String {
    to_b64(sk.serialize().as_ref())
}

pub fn public_to_b64(pk: &PublicKey) -> String {
    to_b64(&pk.serialize_compressed())
}

pub fn b64_to_public(b64: &str) -> anyhow::Result<PublicKey> {
    let json = serde_json::json!(b64);
    let pk: PublicKey = serde_json::from_value(json)?;
    Ok(pk)
}

pub fn b64_to_secret(b64: &str) -> anyhow::Result<SecretKey> {
    let bz = from_b64(b64)?;
    let sk = SecretKey::try_from(bz)?;
    Ok(sk)
}

pub fn read_public_key(public_key: &Path) -> anyhow::Result<PublicKey> {
    let b64 = fs::read_to_string(public_key).context("failed to read public key")?;
    let pk = b64_to_public(&b64).context("failed to parse public key")?;
    Ok(pk)
}

pub fn read_secret_key_hex(private_key: &Path) -> anyhow::Result<SecretKey> {
    let hex_str = fs::read_to_string(private_key).context("failed to read private key")?;
    let mut hex_str = hex_str.trim();
    if hex_str.starts_with("0x") {
        hex_str = &hex_str[2..];
    }
    let raw_secret = hex::decode(hex_str).context("cannot decode hex private key")?;
    let sk = SecretKey::try_from(raw_secret).context("failed to parse secret key")?;
    Ok(sk)
}

pub fn read_secret_key(secret_key: &Path) -> anyhow::Result<SecretKey> {
    let b64 = fs::read_to_string(secret_key).context("failed to read secret key")?;
    let sk = b64_to_secret(&b64).context("failed to parse secret key")?;
    Ok(sk)
}

#[cfg(test)]
mod tests {
    use fendermint_vm_genesis::ValidatorKey;
    use quickcheck_macros::quickcheck;

    use super::{b64_to_public, public_to_b64};

    #[quickcheck]
    fn prop_public_key_deserialize_to_genesis(vk: ValidatorKey) {
        let b64 = public_to_b64(&vk.0);
        let pk = b64_to_public(&b64).unwrap();
        assert_eq!(pk, vk.0)
    }
}
