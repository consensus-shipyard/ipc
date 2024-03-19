// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actor_eam::{EamActor, Method};
use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorError;
use fil_actors_runtime::EAM_ACTOR_ID;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{ActorID, MethodNum};

pub use crate::state::PermissionModeParams;
pub use crate::state::State;

mod state;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(IPCEamActor);

pub const IPC_EAM_ACTOR_NAME: &str = "eam";
pub const IPC_EAM_ACTOR_ID: ActorID = EAM_ACTOR_ID;

pub struct IPCEamActor;

impl IPCEamActor {
    /// Creates the actor. If the `whitelisted_deployers` is empty, that means there is no restriction
    /// for deployment, i.e any address can deploy.
    pub fn constructor(rt: &impl Runtime, args: ConstructorParams) -> Result<(), ActorError> {
        EamActor::constructor(rt)?;

        let st = State::new(rt.store(), args.permission_mode)?;
        rt.create(&st)?;

        Ok(())
    }

    fn ensure_deployer_allowed(rt: &impl Runtime) -> Result<(), ActorError> {
        // The caller is guaranteed to be an ID address.
        let caller_id = rt.message().caller().id().unwrap();

        // Check if the caller is a contract. If it is, and we're in permissioned mode,
        // then the contract was either there in genesis or has been deployed by a whitelisted
        // account; in both cases it's been known up front whether it creates other contracts,
        // and if that was undesireable it would not have been deployed as it is.
        let code_cid = rt.get_actor_code_cid(&caller_id).expect("caller has code");
        if rt.resolve_builtin_actor_type(&code_cid) == Some(Type::EVM) {
            return Ok(());
        }

        // Check if the caller is whitelisted.
        let state: State = rt.state()?;
        if !state.can_deploy(rt, caller_id)? {
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

    fn invoke_method<RT>(
        rt: &RT,
        method: MethodNum,
        params: Option<IpldBlock>,
    ) -> Result<Option<IpldBlock>, ActorError>
    where
        RT: Runtime,
        RT::Blockstore: Blockstore + Clone,
    {
        if method == Method::Constructor as u64 {
            fil_actors_runtime::dispatch(rt, method, Self::constructor, params)
        } else {
            Self::ensure_deployer_allowed(rt)?;
            EamActor::invoke_method(rt, method, params)
        }
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    permission_mode: PermissionModeParams,
}

#[cfg(test)]
mod tests {
    use fil_actor_eam::ext::evm::ConstructorParams;
    use fil_actor_eam::ext::init::{Exec4Params, Exec4Return, EXEC4_METHOD};
    use fil_actor_eam::{compute_address_create, CreateExternalParams, CreateParams, Return};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::runtime::builtins::Type;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, EVM_ACTOR_CODE_ID,
        SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::INIT_ACTOR_ADDR;
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_ipld_encoding::RawBytes;
    use fvm_shared::address::Address;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::error::ExitCode;

    use crate::state::PermissionModeParams;
    use crate::{ConstructorParams as IPCConstructorParams, IPCEamActor, Method};

    pub fn construct_and_verify(deployers: Vec<Address>) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        // construct EAM singleton actor
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);

        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let permission_mode = if deployers.is_empty() {
            PermissionModeParams::Unrestricted
        } else {
            PermissionModeParams::AllowList(deployers)
        };

        let result = rt
            .call::<IPCEamActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&IPCConstructorParams { permission_mode }).unwrap(),
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
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);

        let create_params = CreateExternalParams(vec![0xff]);

        let exit_code = rt
            .call::<IPCEamActor>(
                Method::CreateExternal as u64,
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
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.expect_validate_caller_addr(vec![id_addr]);

        let initcode = vec![0xff];

        let create_params = CreateExternalParams(initcode.clone());

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
                Method::CreateExternal as u64,
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
    fn test_create_by_whitelisted_allowed() {
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let deployers = vec![Address::new_id(2000), f4_eth_addr];
        let rt = construct_and_verify(deployers);

        let id_addr = Address::new_id(1000);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.expect_validate_caller_addr(vec![id_addr]);

        let initcode = vec![0xff];

        let create_params = CreateExternalParams(initcode.clone());

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
                Method::CreateExternal as u64,
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
    fn test_create_by_contract_allowed() {
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let deployers = vec![Address::new_id(2000), f4_eth_addr];
        let rt = construct_and_verify(deployers);

        let id_addr = Address::new_id(1000);
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
