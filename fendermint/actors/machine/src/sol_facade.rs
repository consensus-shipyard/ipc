// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actor_adm::Kind;
use fvm_shared::address::Address;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::machine as sol;
use recall_sol_facade::types::H160;
use std::collections::HashMap;

pub struct MachineCreated<'a> {
    kind: Kind,
    owner: Address,
    metadata: &'a HashMap<String, String>,
}
impl<'a> MachineCreated<'a> {
    pub fn new(kind: Kind, owner: Address, metadata: &'a HashMap<String, String>) -> Self {
        Self {
            kind,
            owner,
            metadata,
        }
    }
}
impl TryIntoEVMEvent for MachineCreated<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let owner: H160 = self.owner.try_into()?;
        let metadata = fvm_ipld_encoding::to_vec(self.metadata)?;
        Ok(sol::Events::MachineCreated(sol::MachineCreated {
            kind: self.kind as u8,
            owner: owner.into(),
            metadata: metadata.into(),
        }))
    }
}

pub struct MachineInitialized {
    kind: Kind,
    machine_address: Address,
}
impl MachineInitialized {
    pub fn new(kind: Kind, machine_address: Address) -> Self {
        Self {
            kind,
            machine_address,
        }
    }
}
impl TryIntoEVMEvent for MachineInitialized {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let machine_address: H160 = self.machine_address.try_into()?;
        Ok(sol::Events::MachineInitialized(sol::MachineInitialized {
            kind: self.kind as u8,
            machineAddress: machine_address.into(),
        }))
    }
}
