// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ethers::types::U256;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::ipc::subnet::SubnetActorErrors;
use fendermint_vm_genesis::{Collateral, Validator};
use fendermint_vm_interpreter::fvm::state::fevm::{
    ContractCaller, ContractResult, MockProvider, NoRevert,
};
use fendermint_vm_interpreter::fvm::state::FvmExecState;
use fendermint_vm_message::conv::{from_eth, from_fvm};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::econ::TokenAmount;
use ipc_actors_abis::subnet_actor_getter_facet::{self as getter, SubnetActorGetterFacet};
use ipc_actors_abis::subnet_actor_manager_facet::SubnetActorManagerFacet;

pub use ipc_actors_abis::register_subnet_facet::ConstructorParams as SubnetConstructorParams;
use ipc_actors_abis::subnet_actor_checkpoint_facet_mock::SubnetActorCheckpointFacetMock;
use ipc_actors_abis::subnet_actor_reward_facet::SubnetActorRewardFacet;
use ipc_api::subnet_id::SubnetID;

#[derive(Clone)]
pub struct SubnetCaller<DB> {
    addr: EthAddress,
    getter: ContractCaller<DB, SubnetActorGetterFacet<MockProvider>, NoRevert>,
    manager: ContractCaller<DB, SubnetActorManagerFacet<MockProvider>, SubnetActorErrors>,
    // use the SubnetActorCheckpointFacetMock instead of SubnetActorCheckpointingFacet to simulate
    // subnet validator logic instead of focusing on cometbft app hash generation/validation
    checkpoint: ContractCaller<DB, SubnetActorCheckpointFacetMock<MockProvider>, SubnetActorErrors>,
    rewarder: ContractCaller<DB, SubnetActorRewardFacet<MockProvider>, SubnetActorErrors>,
}

impl<DB> SubnetCaller<DB> {
    pub fn new(addr: EthAddress) -> Self {
        Self {
            addr,
            getter: ContractCaller::new(addr, SubnetActorGetterFacet::new),
            manager: ContractCaller::new(addr, SubnetActorManagerFacet::new),
            checkpoint: ContractCaller::new(addr, SubnetActorCheckpointFacetMock::new),
            rewarder: ContractCaller::new(addr, SubnetActorRewardFacet::new),
        }
    }

    pub fn addr(&self) -> EthAddress {
        self.addr
    }
}

type TryCallResult<T> = anyhow::Result<ContractResult<T, SubnetActorErrors>>;

impl<DB: Blockstore + Clone> SubnetCaller<DB> {
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
            c.join(public_key.into(), deposit).from(addr).value(deposit)
        })
    }

    /// Try to join the subnet as a validator.
    pub fn try_join(
        &self,
        state: &mut FvmExecState<DB>,
        validator: &Validator<Collateral>,
    ) -> TryCallResult<()> {
        let public_key = validator.public_key.0.serialize();
        let addr = EthAddress::new_secp256k1(&public_key)?;
        let deposit = from_fvm::to_eth_tokens(&validator.power.0)?;
        self.manager.try_call(state, |c| {
            c.join(public_key.into(), deposit).from(addr).value(deposit)
        })
    }

    /// Try to increase the stake of a validator.
    pub fn try_stake(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
        value: &TokenAmount,
    ) -> TryCallResult<()> {
        let deposit = from_fvm::to_eth_tokens(value)?;
        self.manager
            .try_call(state, |c| c.stake(deposit).from(addr).value(deposit))
    }

    /// Try to decrease the stake of a validator.
    pub fn try_unstake(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
        value: &TokenAmount,
    ) -> TryCallResult<()> {
        let withdraw = from_fvm::to_eth_tokens(value)?;
        self.manager
            .try_call(state, |c| c.unstake(withdraw).from(addr))
    }

    /// Try to remove all stake of a validator.
    pub fn try_leave(&self, state: &mut FvmExecState<DB>, addr: &EthAddress) -> TryCallResult<()> {
        self.manager.try_call(state, |c| c.leave().from(addr))
    }

    /// Claim any refunds.
    pub fn try_claim(&self, state: &mut FvmExecState<DB>, addr: &EthAddress) -> TryCallResult<()> {
        self.rewarder.try_call(state, |c| c.claim().from(addr))
    }

    /// Get information about the validator's current and total collateral.
    pub fn get_validator(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
    ) -> anyhow::Result<getter::ValidatorInfo> {
        self.getter.call(state, |c| c.get_validator(addr.into()))
    }

    /// Get the confirmed collateral of a validator.
    pub fn confirmed_collateral(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
    ) -> anyhow::Result<TokenAmount> {
        self.get_validator(state, addr)
            .map(|i| from_eth::to_fvm_tokens(&i.current_power))
    }

    /// Get the total (unconfirmed) collateral of a validator.
    pub fn total_collateral(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
    ) -> anyhow::Result<TokenAmount> {
        self.get_validator(state, addr)
            .map(|i| from_eth::to_fvm_tokens(&i.next_power))
    }

    /// Get the `(next, start)` configuration number pair.
    ///
    /// * `next` is the next expected one
    /// * `start` is the first unapplied one
    pub fn get_configuration_numbers(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<(u64, u64)> {
        self.getter.call(state, |c| c.get_configuration_numbers())
    }

    /// Check if minimum collateral has been met.
    pub fn bootstrapped(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<bool> {
        self.getter.call(state, |c| c.bootstrapped())
    }

    /// Check if a validator is active, ie. they are in the top N.
    pub fn is_active(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
    ) -> anyhow::Result<bool> {
        self.getter
            .call(state, |c| c.is_active_validator(addr.into()))
    }

    /// Check if a validator is wating, ie. they have deposited but are not in the top N.
    pub fn is_waiting(
        &self,
        state: &mut FvmExecState<DB>,
        addr: &EthAddress,
    ) -> anyhow::Result<bool> {
        self.getter
            .call(state, |c| c.is_waiting_validator(addr.into()))
    }

    /// This is purely for testing, although we could use it in production to avoid having to match Rust and Solidity semantics.
    pub fn cross_msgs_hash(
        &self,
        state: &mut FvmExecState<DB>,
        cross_msgs: Vec<getter::IpcEnvelope>,
    ) -> anyhow::Result<[u8; 32]> {
        self.getter.call(state, |c| c.cross_msgs_hash(cross_msgs))
    }

    pub fn drive_validator_change(
        &self,
        state: &mut FvmExecState<DB>,
        _: &SubnetID,
        height: u64,
        next_config_number: u64,
    ) -> TryCallResult<()> {
        self.checkpoint.try_call(state, |c| {
            c.commit_side_effects(
                U256::from(height),
                Default::default(),
                Default::default(),
                Default::default(),
                next_config_number,
            )
        })
    }
}
