// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorError;
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(IPCEamActor);

pub const IPC_GAS_ACTOR_NAME: &str = "gas";
pub type Gas = u64;

pub struct IPCGasActor;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    block_gas_limit: Gas,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    SetBlockGasLimit = frc42_dispatch::method_hash!("SetBlockGasLimit"),
}

impl IPCGasActor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, args: ConstructorParams) -> Result<(), ActorError> {
        let st = State {
            block_gas_limit: args.block_gas_limit,
        };
        rt.create(&st)?;

        Ok(())
    }

    fn set_block_gas_limit(rt: &impl Runtime, limit: Gas) -> Result<(), ActorError> {
        if rt.message().caller() != SYSTEM_ACTOR_ADDR {
            return Err(ActorError::forbidden("not system actor".into()));
        }

        rt.transaction(|st: &mut State, _rt| {
            st.block_gas_limit = limit;
            Ok(())
        })?;

        Ok(())
    }
}

impl ActorCode for IPCGasActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_GAS_ACTOR_NAME
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
        } else if method == Method::SetBlockGasLimit as u64 {
            fil_actors_runtime::dispatch(rt, method, Self::set_block_gas_limit, params)
        } else {
            Err(ActorError::not_found("method not found".into()))
        }
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    block_gas_limit: Gas,
}

impl State {
    pub fn block_gas_limit(&self) -> Gas {
        self.block_gas_limit
    }
}

#[cfg(test)]
mod tests {
    use crate::{ConstructorParams, IPCGasActor, Method, State};
    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::error::ExitCode;

    pub fn construct_and_verify() -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        let result = rt
            .call::<IPCGasActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    block_gas_limit: 100,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();

        rt
    }

    #[test]
    fn test_set_ok() {
        let rt = construct_and_verify();
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);

        let r = rt.call::<IPCGasActor>(
            Method::SetBlockGasLimit as u64,
            IpldBlock::serialize_cbor(&20).unwrap(),
        );
        assert!(r.is_ok());

        let s = rt.get_state::<State>();
        assert_eq!(s.block_gas_limit, 20);
    }

    #[test]
    fn test_not_allowed() {
        let rt = construct_and_verify();
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, Address::new_id(1000));

        let code = rt
            .call::<IPCGasActor>(
                Method::SetBlockGasLimit as u64,
                IpldBlock::serialize_cbor(&20).unwrap(),
            )
            .unwrap_err()
            .exit_code();
        assert_eq!(code, ExitCode::USR_FORBIDDEN)
    }
}
