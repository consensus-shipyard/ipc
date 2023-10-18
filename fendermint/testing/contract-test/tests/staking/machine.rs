// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::sync::Arc;

use arbitrary::{Arbitrary, Unstructured};
use contract_test::ipc::{registry::RegistryCaller, subnet::SubnetCaller};
use ethers::types as et;
use fendermint_testing::smt::StateMachine;
use fendermint_vm_actor_interface::ipc::subnet_id_to_eth;
use fendermint_vm_interpreter::fvm::{
    state::{ipc::GatewayCaller, FvmExecState},
    store::memory::MemoryBlockstore,
};
use fendermint_vm_message::conv::from_fvm;
use fvm::engine::MultiEngine;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::bigint::Integer;
use fvm_shared::econ::TokenAmount;

use super::state::StakingState;
use contract_test::ipc::registry::SubnetConstructorParams;

/// System Under Test for staking.
pub struct StakingSystem {
    /// FVM state initialized with the parent genesis, and a subnet created for the child.
    _exec_state: FvmExecState<MemoryBlockstore>,
    _gateway: GatewayCaller<MemoryBlockstore>,
    _registry: RegistryCaller<MemoryBlockstore>,
    _subnet: SubnetCaller<MemoryBlockstore>,
}

pub enum StakingCommand {
    /// Bottom-up checkpoint; confirms all staking operations up to the configuration number.
    Checkpoint { next_configuration_number: u64 },
    /// Increase the collateral of a validator; when it goes from 0 this means joining the subnet.
    Stake(Address, TokenAmount),
    /// Decrease the collateral of a validator; if it goes to 0 it means leaving the subnet.
    Unstake(Address, TokenAmount),
}

#[derive(Default)]
pub struct StakingMachine {
    multi_engine: Arc<MultiEngine>,
}

impl StateMachine for StakingMachine {
    type System = StakingSystem;

    type State = StakingState;

    type Command = StakingCommand;

    type Result = ();

    fn gen_state(&self, u: &mut Unstructured) -> arbitrary::Result<Self::State> {
        StakingState::arbitrary(u)
    }

    fn new_system(&self, state: &Self::State) -> Self::System {
        let rt = tokio::runtime::Runtime::new().expect("create tokio runtime for init");

        let (mut exec_state, _) = rt
            .block_on(contract_test::init_exec_state(
                self.multi_engine.clone(),
                state.parent_genesis.clone(),
            ))
            .expect("failed to init parent");

        let gateway = GatewayCaller::default();
        let registry = RegistryCaller::default();

        // Deploy a new subnet based on `state.child_genesis`
        let parent_ipc = state.parent_genesis.ipc.as_ref().unwrap();
        let child_ipc = state.child_genesis.ipc.as_ref().unwrap();

        let (root, route) =
            subnet_id_to_eth(&parent_ipc.gateway.subnet_id).expect("subnet ID is valid");

        let params = SubnetConstructorParams {
            parent_id: ipc_actors_abis::subnet_registry::SubnetID { root, route },
            ipc_gateway_addr: gateway.addr().into(),
            consensus: 0, // TODO: What are the options?
            bottom_up_check_period: child_ipc.gateway.bottom_up_check_period,
            majority_percentage: child_ipc.gateway.majority_percentage,
            active_validators_limit: child_ipc.gateway.active_validators_limit,
            power_scale: state.child_genesis.power_scale,
            // The `min_activation_collateral` has to be at least as high as the parent gateway's `min_collateral`,
            // otherwise it will refuse the subnet trying to register itself.
            min_activation_collateral: from_fvm::to_eth_tokens(&parent_ipc.gateway.min_collateral)
                .unwrap(),
            min_validators: 1,
            min_cross_msg_fee: et::U256::zero(),
        };

        //eprintln!("\n> CREATING SUBNET: {params:?}");

        let subnet_addr = registry
            .new_subnet(&mut exec_state, params)
            .expect("failed to create subnet");

        let subnet = SubnetCaller::new(subnet_addr);

        // Make all the validators join the subnet by putting down collateral according to their power.
        for v in state.child_genesis.validators.iter() {
            //eprintln!("\n> JOINING SUBNET: {v:?}");
            subnet
                .join(&mut exec_state, v)
                .expect("failed to join subnet");
        }

        StakingSystem {
            _exec_state: exec_state,
            _gateway: gateway,
            _registry: registry,
            _subnet: subnet,
        }
    }

    fn gen_command(
        &self,
        u: &mut Unstructured,
        state: &Self::State,
    ) -> arbitrary::Result<Self::Command> {
        let cmd = match u.choose(&["checkpoint", "stake", "unstake"]).unwrap() {
            &"checkpoint" => {
                let cn = match state.pending_updates.len() {
                    0 => state.configuration_number,
                    n => {
                        let idx = u.choose_index(n).expect("non-zero");
                        state.pending_updates[idx].configuration_number
                    }
                };
                StakingCommand::Checkpoint {
                    next_configuration_number: cn,
                }
            }
            &"stake" => {
                let a = u.choose(&state.addrs).expect("accounts not empty");
                let a = state.accounts.get(a).expect("account exists");
                // Limit ourselves to the outstanding balance - the user would not be able to send more value to the contract.
                let b = BigInt::arbitrary(u)?.mod_floor(a.current_balance.atto());
                let b = TokenAmount::from_atto(b);
                StakingCommand::Stake(a.addr, b)
            }
            &"unstake" => {
                let a = u.choose(&state.addrs).expect("accounts not empty");
                let a = state.accounts.get(a).expect("account exists");
                // We can try sending requests to unbond arbitrarily large amounts of collateral - the system should catch any attempt to steal.
                // Only limiting it to be under the initial balance so that it's comparable to what the deposits could have been.
                let b = BigInt::arbitrary(u)?.mod_floor(a.initial_balance.atto());
                let b = TokenAmount::from_atto(b);
                StakingCommand::Unstake(a.addr, b)
            }
            other => unimplemented!("unknown command: {other}"),
        };
        Ok(cmd)
    }

    fn run_command(&self, _system: &mut Self::System, _cmd: &Self::Command) -> Self::Result {
        // TODO: Execute the command against the contract.
    }

    fn check_result(&self, _cmd: &Self::Command, _pre_state: &Self::State, _result: &Self::Result) {
        // TODO: Check that events emitted by the system are as expected.
    }

    fn next_state(&self, cmd: &Self::Command, state: Self::State) -> Self::State {
        match cmd {
            StakingCommand::Checkpoint {
                next_configuration_number,
            } => state.checkpoint(*next_configuration_number),
            StakingCommand::Stake(addr, value) => state.stake(*addr, value.clone()),
            StakingCommand::Unstake(addr, value) => state.unstake(*addr, value.clone()),
        }
    }

    fn check_system(
        &self,
        cmd: &Self::Command,
        post_state: &Self::State,
        _post_system: &Self::System,
    ) {
        match cmd {
            StakingCommand::Checkpoint { .. } => {
                // Sanity check the reference state while we have no contract to compare with.
                debug_assert!(
                    post_state
                        .accounts
                        .iter()
                        .all(|(_, a)| a.current_balance <= a.initial_balance),
                    "no account goes over initial balance"
                );

                debug_assert!(
                    post_state
                        .child_validators
                        .iter()
                        .all(|(_, p)| !p.0.is_zero()),
                    "all child validators have non-zero collateral"
                );
            }
            StakingCommand::Stake(addr, _) | StakingCommand::Unstake(addr, _) => {
                let a = post_state.accounts.get(addr).unwrap();
                debug_assert!(a.current_balance <= a.initial_balance);
            }
        }

        // TODO: Compare the system with the state:
        // * check that balances match
        // * check that active powers match
    }
}
