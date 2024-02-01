// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::{ActorError, Map2, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;

pub type DeployerMap<BS> = Map2<BS, Address, ()>;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    /// The allowed deployers of contract
    pub deployers: Cid, // HAMT[Address]()
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<State, ActorError> {
        let empty_deployers = DeployerMap::empty(store, DEFAULT_HAMT_CONFIG, "empty").flush()?;

        Ok(State {
            deployers: empty_deployers,
        })
    }

    /// Adds a deployer, i.e. allows the deployer to deploy contracts in IPC subnets
    pub fn add_deployer(
        &mut self,
        store: &impl Blockstore,
        deployer: &Address,
    ) -> Result<(), ActorError> {
        let mut deployers = self.load_deployers(store)?;
        deployers.set(deployer, ())?;
        self.deployers = deployers.flush()?;
        Ok(())
    }

    pub fn can_deploy(
        &self,
        store: &impl Blockstore,
        deployer: &Address,
    ) -> Result<bool, ActorError> {
        let deployers = self.load_deployers(store)?;
        deployers.contains_key(deployer)
    }

    pub fn load_deployers<BS: Blockstore>(&self, store: BS) -> Result<DeployerMap<BS>, ActorError> {
        DeployerMap::load(store, &self.deployers, DEFAULT_HAMT_CONFIG, "verifiers")
    }
}
