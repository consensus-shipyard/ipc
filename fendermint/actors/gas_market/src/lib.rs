// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(EIP1559GasMarketActor);

pub const IPC_GAS_ACTOR_NAME: &str = "gas";

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559GasState {
    pub block_gas_limit: u64,
}

pub struct EIP1559GasMarketActor {}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    UpdateGasMarketState = frc42_dispatch::method_hash!("UpdateGasMarketState"),
}

impl EIP1559GasMarketActor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, st: EIP1559GasState) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.create(&st)?;

        Ok(())
    }

    fn update_gas_market_state(
        rt: &impl Runtime,
        new_state: EIP1559GasState,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            *st = new_state;
            Ok(())
        })?;

        Ok(())
    }
}

impl ActorCode for EIP1559GasMarketActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_GAS_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        UpdateGasMarketState => update_gas_market_state,
    }
}

#[cfg(test)]
mod tests {
    use crate::{EIP1559GasMarketActor, EIP1559GasState, Method};
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

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let result = rt
            .call::<EIP1559GasMarketActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&EIP1559GasState {
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
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let r = rt.call::<EIP1559GasMarketActor>(
            Method::UpdateGasMarketState as u64,
            IpldBlock::serialize_cbor(&EIP1559GasState {
                block_gas_limit: 20,
            })
            .unwrap(),
        );
        assert!(r.is_ok());

        let s = rt.get_state::<EIP1559GasState>();
        assert_eq!(s.block_gas_limit, 20);
    }

    #[test]
    fn test_not_allowed() {
        let rt = construct_and_verify();
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, Address::new_id(1000));
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let code = rt
            .call::<EIP1559GasMarketActor>(
                Method::UpdateGasMarketState as u64,
                IpldBlock::serialize_cbor(&EIP1559GasState {
                    block_gas_limit: 20,
                })
                .unwrap(),
            )
            .unwrap_err()
            .exit_code();
        assert_eq!(code, ExitCode::USR_FORBIDDEN)
    }
}
