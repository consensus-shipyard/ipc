// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use crate::state::State;
pub use fil_actor_eam::Method;
use fil_actor_eam::{
    Create2Params, Create2Return, CreateExternalParams, CreateExternalReturn, CreateParams,
    CreateReturn, EamActor,
};
use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch_unrestricted, ActorError};
use fvm_shared::address::Address;

mod state;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(IPCEamActor);

pub const IPC_EAM_ACTOR_NAME: &str = "eam";

pub struct IPCEamActor;

impl IPCEamActor {
    /// Creates the actor. If the `whitelisted_deployers` is empty, that means there is no restriction
    /// for deployment, i.e any address can deploy.
    pub fn constructor(
        rt: &impl Runtime,
        whitelisted_deployers: Vec<Address>,
    ) -> Result<(), ActorError> {
        EamActor::constructor(rt)?;

        let st = State::new(rt.store(), whitelisted_deployers)?;
        rt.create(&st)?;

        Ok(())
    }

    /// Create a new contract per the EVM's CREATE rules.
    ///
    /// Permissions: May be called by the EVM.
    pub fn create(rt: &impl Runtime, params: CreateParams) -> Result<CreateReturn, ActorError> {
        Self::ensure_deployer_allowed(rt)?;
        EamActor::create(rt, params)
    }

    /// Create a new contract per the EVM's CREATE2 rules.
    ///
    /// Permissions: May be called by the EVM.
    pub fn create2(rt: &impl Runtime, params: Create2Params) -> Result<Create2Return, ActorError> {
        Self::ensure_deployer_allowed(rt)?;
        EamActor::create2(rt, params)
    }

    /// Create a new contract from off-chain.
    ///
    /// When called by an EthAccount, this method will compute the new actor's address according to
    /// the `CREATE` rules. When called by a "native" Account, this method will derive the address
    /// from the _hash_ of the caller's key address.
    ///
    /// Permissions: May be called by builtin or eth accounts.
    pub fn create_external(
        rt: &impl Runtime,
        params: CreateExternalParams,
    ) -> Result<CreateExternalReturn, ActorError> {
        Self::ensure_deployer_allowed(rt)?;
        EamActor::create_external(rt, params)
    }

    fn ensure_deployer_allowed(rt: &impl Runtime) -> Result<(), ActorError> {
        let caller = rt.message().caller();

        let state: State = rt.state()?;
        if !state.can_deploy(rt.store(), &caller)? {
            return Err(ActorError::forbidden(String::from(
                "sender not allowed to deploy contracts",
            )));
        }

        Ok(())
    }
}

impl ActorCode for IPCEamActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_EAM_ACTOR_NAME
    }

    actor_dispatch_unrestricted! {
        Constructor => constructor,
        Create => create,
        Create2 => create2,
        CreateExternal => create_external,
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPCEamActor, Method};
    use fil_actor_eam::ext::evm::ConstructorParams;
    use fil_actor_eam::ext::init::{Exec4Params, Exec4Return, EXEC4_METHOD};
    use fil_actor_eam::{compute_address_create, CreateParams, Return};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::runtime::builtins::Type;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, EVM_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::INIT_ACTOR_ADDR;
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_ipld_encoding::RawBytes;
    use fvm_shared::address::Address;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::error::ExitCode;

    pub fn construct_and_verify(deployers: Vec<Address>) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        // construct EAM singleton actor
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);

        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let result = rt
            .call::<IPCEamActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&deployers).unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();

        rt
    }

    #[test]
    fn test_create_not_allowed() {
        let deployers = vec![Address::new_id(1000), Address::new_id(2000)];
        let rt = construct_and_verify(deployers);

        let id_addr = Address::new_id(10000);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*EVM_ACTOR_CODE_ID, id_addr);

        let create_params = CreateParams {
            initcode: vec![0xff],
            nonce: 0,
        };

        let exit_code = rt
            .call::<IPCEamActor>(
                Method::Create as u64,
                IpldBlock::serialize_cbor(&create_params).unwrap(),
            )
            .unwrap_err()
            .exit_code();
        assert_eq!(exit_code, ExitCode::USR_FORBIDDEN)
    }

    #[test]
    fn test_create_no_restriction() {
        let deployers = vec![];
        let rt = construct_and_verify(deployers);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*EVM_ACTOR_CODE_ID, id_addr);
        rt.expect_validate_caller_type(vec![Type::EVM]);

        let initcode = vec![0xff];

        let create_params = CreateParams {
            initcode: initcode.clone(),
            nonce: 0,
        };

        let evm_params = ConstructorParams {
            creator: eth_addr,
            initcode: initcode.into(),
        };

        let new_eth_addr = compute_address_create(&rt, &eth_addr, 0);
        let params = Exec4Params {
            code_cid: *EVM_ACTOR_CODE_ID,
            constructor_params: RawBytes::serialize(evm_params).unwrap(),
            subaddress: new_eth_addr.0[..].to_owned().into(),
        };

        let send_return = IpldBlock::serialize_cbor(&Exec4Return {
            id_address: Address::new_id(111),
            robust_address: Address::new_id(0), // not a robust address but im hacking here and nobody checks
        })
        .unwrap();

        rt.expect_send_simple(
            INIT_ACTOR_ADDR,
            EXEC4_METHOD,
            IpldBlock::serialize_cbor(&params).unwrap(),
            TokenAmount::from_atto(0),
            send_return,
            ExitCode::OK,
        );

        let result = rt
            .call::<IPCEamActor>(
                Method::Create as u64,
                IpldBlock::serialize_cbor(&create_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Return>()
            .unwrap();

        let expected_return = Return {
            actor_id: 111,
            robust_address: Some(Address::new_id(0)),
            eth_address: new_eth_addr,
        };

        assert_eq!(result, expected_return);
        rt.verify();
    }

    #[test]
    fn test_create_allowed() {
        let deployers = vec![Address::new_id(1000), Address::new_id(2000)];
        let rt = construct_and_verify(deployers);

        let id_addr = Address::new_id(1000);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);

        rt.set_caller(*EVM_ACTOR_CODE_ID, id_addr);

        rt.expect_validate_caller_type(vec![Type::EVM]);

        let initcode = vec![0xff];

        let create_params = CreateParams {
            initcode: initcode.clone(),
            nonce: 0,
        };

        let evm_params = ConstructorParams {
            creator: eth_addr,
            initcode: initcode.into(),
        };

        let new_eth_addr = compute_address_create(&rt, &eth_addr, 0);
        let params = Exec4Params {
            code_cid: *EVM_ACTOR_CODE_ID,
            constructor_params: RawBytes::serialize(evm_params).unwrap(),
            subaddress: new_eth_addr.0[..].to_owned().into(),
        };

        let send_return = IpldBlock::serialize_cbor(&Exec4Return {
            id_address: Address::new_id(111),
            robust_address: Address::new_id(0), // not a robust address but im hacking here and nobody checks
        })
        .unwrap();

        rt.expect_send_simple(
            INIT_ACTOR_ADDR,
            EXEC4_METHOD,
            IpldBlock::serialize_cbor(&params).unwrap(),
            TokenAmount::from_atto(0),
            send_return,
            ExitCode::OK,
        );

        let result = rt
            .call::<IPCEamActor>(
                Method::Create as u64,
                IpldBlock::serialize_cbor(&create_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Return>()
            .unwrap();

        let expected_return = Return {
            actor_id: 111,
            robust_address: Some(Address::new_id(0)),
            eth_address: new_eth_addr,
        };

        assert_eq!(result, expected_return);
        rt.verify();
    }
}
