// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the validator id from tendermint to their corresponding public key.

use anyhow::anyhow;
use fendermint_crypto::PublicKey;
use fvm_shared::clock::ChainEpoch;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tendermint::block::Height;
use tendermint_rpc::{Client, Paging};

#[derive(Clone)]
pub(crate) struct ValidatorTracker<C> {
    client: C,
    public_keys: Arc<RwLock<HashMap<tendermint::account::Id, PublicKey>>>,
}

impl<C: Client> ValidatorTracker<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<C: Client + Sync> ValidatorTracker<C> {
    /// Get the public key of the validator by id. Note that the id is expected to be a validator.
    pub async fn get_validator(
        &self,
        id: &tendermint::account::Id,
        height: ChainEpoch,
    ) -> anyhow::Result<PublicKey> {
        if let Some(key) = self.get_from_cache(id) {
            return Ok(key);
        }

        // this means validators have changed, re-pull all validators
        let height = Height::try_from(height)?;
        let response = self.client.validators(height, Paging::All).await?;

        let mut new_validators = HashMap::new();
        let mut pubkey = None;
        for validator in response.validators {
            let key = fendermint_pub_key_from_tendermint_pub_key(&validator.pub_key)?;

            if *id == validator.address {
                pubkey = Some(key);
            }

            new_validators.insert(validator.address, key);
        }

        *self.public_keys.write().unwrap() = new_validators;

        // cannot find the validator, this should not have happened usually
        pubkey.ok_or_else(|| anyhow!("{} not validator", id))
    }

    fn get_from_cache(&self, id: &tendermint::account::Id) -> Option<PublicKey> {
        let keys = self.public_keys.read().unwrap();
        keys.get(id).copied()
    }

    pub fn hydrate_cache(
        &self,
        public_keys: Vec<(tendermint::account::Id, tendermint::public_key::PublicKey)>,
    ) {
        let public_keys = public_keys
            .into_iter()
            .filter_map(|(id, key)| {
                fendermint_pub_key_from_tendermint_pub_key(&key)
                    .map(|key| (id, key))
                    .ok()
            })
            .collect();

        *self.public_keys.write().unwrap() = public_keys;
    }
}

// TODO Karel - consider using From trait
fn fendermint_pub_key_from_tendermint_pub_key(
    key: &tendermint::public_key::PublicKey,
) -> anyhow::Result<PublicKey> {
    let p = key.secp256k1().unwrap();
    let compressed = p.to_encoded_point(true);
    let b = compressed.as_bytes();
    let key = PublicKey::parse_slice(b, None)?;
    Ok(key)
}
