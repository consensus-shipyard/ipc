// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::ipc::subnet::SubnetActorErrors;
use fendermint_vm_genesis::{Collateral, Validator};
use fendermint_vm_interpreter::fvm::state::fevm::{ContractCaller, MockProvider, NoRevert};
use fendermint_vm_interpreter::fvm::state::FvmExecState;
use fendermint_vm_message::conv::from_fvm;
use fvm_ipld_blockstore::Blockstore;
use ipc_actors_abis::subnet_actor_getter_facet::SubnetActorGetterFacet;
use ipc_actors_abis::subnet_actor_manager_facet::SubnetActorManagerFacet;

pub use ipc_actors_abis::subnet_registry::ConstructorParams as SubnetConstructorParams;

#[derive(Clone)]
pub struct SubnetCaller<DB> {
    addr: EthAddress,
    _getter: ContractCaller<DB, SubnetActorGetterFacet<MockProvider>, NoRevert>,
    manager: ContractCaller<DB, SubnetActorManagerFacet<MockProvider>, SubnetActorErrors>,
}

impl<DB> SubnetCaller<DB> {
    pub fn new(addr: EthAddress) -> Self {
        Self {
            addr,
            _getter: ContractCaller::new(addr, SubnetActorGetterFacet::new),
            manager: ContractCaller::new(addr, SubnetActorManagerFacet::new),
        }
    }

    pub fn addr(&self) -> EthAddress {
        self.addr
    }
}

impl<DB: Blockstore> SubnetCaller<DB> {
    /// Join a subnet as a validator.
    pub fn join(
        &self,
        state: &mut FvmExecState<DB>,
        validator: &Validator<Collateral>,
    ) -> anyhow::Result<()> {
        let public_key = validator.public_key.0.serialize();
        let addr = EthAddress::new_secp256k1(&public_key)?;
        let deposit = from_fvm::to_eth_tokens(&validator.power.0)?;

        // We need to send in the name of the address as a sender, not the system account.
        self.manager.call(state, |c| {
            c.join(public_key.into()).from(addr).value(deposit)
        })
    }
}
