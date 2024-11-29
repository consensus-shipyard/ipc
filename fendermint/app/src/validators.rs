// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the validator ID from Tendermint to their corresponding public key.

use anyhow::{anyhow, Ok, Result};
use fendermint_crypto::PublicKey;
use fendermint_vm_genesis::{Power, Validator};
use fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller;
use fendermint_vm_interpreter::fvm::state::FvmExecState;
use std::collections::HashMap;

use tendermint::account::Id as TendermintId;
use tendermint::PublicKey as TendermintPubKey;

use fvm_ipld_blockstore::Blockstore;

#[derive(Clone)]
pub(crate) struct ValidatorCache {
    map: HashMap<TendermintId, PublicKey>,
}

impl ValidatorCache {
    pub fn new_from_state<SS>(state: &mut FvmExecState<SS>) -> Result<Self>
    where
        SS: Blockstore + Clone + 'static,
    {
        let gateway = GatewayCaller::default();
        let (_, validators) = gateway.current_power_table(state)?;

        let map = validators
            .iter()
            .map(validator_to_map_entry)
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Self { map })
    }

    pub fn get_validator(&self, id: &tendermint::account::Id) -> Result<PublicKey> {
        self.map
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("validator not found"))
    }
}

fn validator_to_map_entry(v: &Validator<Power>) -> Result<(TendermintId, PublicKey)> {
    let tendermint_pub_key: TendermintPubKey = TendermintPubKey::try_from(v.public_key.clone())?;
    let id = TendermintId::from(tendermint_pub_key);
    let key = *v.public_key.public_key();
    Ok((id, key))
}
