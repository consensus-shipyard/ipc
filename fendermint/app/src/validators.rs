// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the validator id from tendermint to their corresponding public key.

use anyhow::anyhow;
use fendermint_crypto::PublicKey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub(crate) struct ValidatorTracker {
    public_keys: Arc<RwLock<HashMap<tendermint::account::Id, PublicKey>>>,
}

impl ValidatorTracker {
    pub fn new() -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ValidatorTracker {
    /// Get the public key of the validator by id. Note that the id is expected to be a validator.
    pub fn get_validator(&self, id: &tendermint::account::Id) -> anyhow::Result<PublicKey> {
        let keys = self.public_keys.read().unwrap();
        keys.get(id)
            .copied()
            .ok_or_else(|| anyhow!("validator not found: {:?}", id))
    }

    pub fn set_validators(
        &self,
        public_keys: Vec<(tendermint::account::Id, tendermint::public_key::PublicKey)>,
    ) {
        let mut cache = self.public_keys.write().unwrap();

        for (id, key) in public_keys {
            if let Ok(fendermint_key) = fendermint_pub_key_from_tendermint_pub_key(&key) {
                cache.insert(id, fendermint_key);
            }
        }
    }
}

fn fendermint_pub_key_from_tendermint_pub_key(
    key: &tendermint::public_key::PublicKey,
) -> anyhow::Result<PublicKey> {
    let p = key.secp256k1().unwrap();
    let compressed = p.to_encoded_point(true);
    let b = compressed.as_bytes();
    let key = PublicKey::parse_slice(b, None)?;
    Ok(key)
}
