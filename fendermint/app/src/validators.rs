// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the validator ID from Tendermint to their corresponding public key.

use anyhow::{anyhow, Result};
use fendermint_crypto::PublicKey;
use fendermint_vm_genesis::ValidatorKey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub(crate) struct ValidatorTracker {
    validator_mapping: Arc<RwLock<HashMap<tendermint::account::Id, PublicKey>>>,
}

impl ValidatorTracker {
    pub fn new() -> Self {
        Self {
            validator_mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the public key of the validator by ID.
    /// Note that the ID is expected to be a validator.
    pub fn get_public_key(&self, id: &tendermint::account::Id) -> Result<PublicKey> {
        let keys = self
            .validator_mapping
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        keys.get(id)
            .copied()
            .ok_or_else(|| anyhow!("Validator not found: {:?}", id))
    }

    /// Sets the validator keys mapping.
    pub fn set_validators(&self, validators: Vec<ValidatorKey>) -> Result<()> {
        let mut cache = self
            .validator_mapping
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock to update validators"))?;

        cache.clear();

        validators.into_iter().try_for_each(|validator_key| {
            let tendermint_pub_key = tendermint::PublicKey::try_from(validator_key.clone())
                .map_err(|_| anyhow!("Failed to convert ValidatorKey to Tendermint public key"))?;

            let tendermint_id = tendermint::account::Id::from(tendermint_pub_key);
            cache.insert(tendermint_id, *validator_key.public_key());
            Ok(())
        })
    }
}
