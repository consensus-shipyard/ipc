// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use cid::Cid;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, ActorID};

use crate::{Kind, Metadata};

/// ADM actor state
///
/// NOTE: This is a simplified implementation for now. A production implementation
/// would use HAMTs for efficient storage and indexing of machines.
#[derive(Serialize_tuple, Deserialize_tuple, Clone)]
pub struct State {
    /// Addresses allowed to deploy machines
    pub deployers: Vec<Address>,
    /// List of all machines (for now, kept in memory)
    pub machines: Vec<MachineInfo>,
    /// Code CID for bucket actor
    pub bucket_code_cid: Option<Cid>,
    /// Code CID for timehub actor
    pub timehub_code_cid: Option<Cid>,
}

/// Information about a created machine
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct MachineInfo {
    /// Actor ID of the machine
    pub actor_id: ActorID,
    /// Machine type
    pub kind: Kind,
    /// Owner address (ID address)
    pub owner: Address,
    /// Robust address (delegated or ID)
    pub address: Address,
    /// User-defined metadata
    pub metadata: HashMap<String, String>,
}

impl State {
    /// Create new ADM state
    pub fn new<BS: Blockstore>(
        _store: &BS,
        deployers: Vec<Address>,
    ) -> Result<Self, ActorError> {
        Ok(State {
            deployers,
            machines: Vec::new(),
            bucket_code_cid: None,
            timehub_code_cid: None,
        })
    }

    /// Set the code CID for a machine type
    pub fn set_code_cid(&mut self, kind: &Kind, cid: Cid) {
        match kind {
            Kind::Bucket => self.bucket_code_cid = Some(cid),
            Kind::Timehub => self.timehub_code_cid = Some(cid),
        }
    }

    /// Get the code CID for a machine type
    pub fn get_code_cid(&self, kind: &Kind) -> Result<Cid, ActorError> {
        match kind {
            Kind::Bucket => self.bucket_code_cid.ok_or_else(|| {
                ActorError::illegal_state("Bucket actor code CID not registered".to_string())
            }),
            Kind::Timehub => self.timehub_code_cid.ok_or_else(|| {
                ActorError::illegal_state("Timehub actor code CID not registered".to_string())
            }),
        }
    }

    /// Add a machine to the registry
    pub fn add_machine(
        &mut self,
        actor_id: ActorID,
        owner: Address,
        kind: Kind,
        address: &Address,
    ) -> Result<(), ActorError> {
        self.machines.push(MachineInfo {
            actor_id,
            kind,
            owner,
            address: *address,
            metadata: HashMap::new(),
        });
        Ok(())
    }

    /// Update deployer list
    pub fn update_deployers(
        &mut self,
        deployers: Vec<Address>,
    ) -> Result<(), ActorError> {
        self.deployers = deployers;
        Ok(())
    }

    /// List all machines owned by an address
    pub fn list_machines_by_owner<BS: Blockstore>(
        &self,
        _store: &BS,
        owner: &Address,
    ) -> Result<Vec<Metadata>, ActorError> {
        let result = self
            .machines
            .iter()
            .filter(|m| &m.owner == owner)
            .map(|m| Metadata {
                kind: m.kind.clone(),
                address: m.address,
                metadata: m.metadata.clone(),
            })
            .collect();
        Ok(result)
    }
}

