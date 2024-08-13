// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::econ::TokenAmount;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use num_traits::Zero;
use std::ops::Mul;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(EIP1559GasMarketActor);

/// Base fee max change denominator as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
const BASE_FEE_MAX_CHANGE_DENOMINATOR: u64 = 8;
/// Elasticity multiplier as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
const ELASTICITY_MULTIPLIER: u64 = 2;
/// Initial base fee as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
pub const INITIAL_BASE_FEE: u64 = 1_000_000_000;
pub const IPC_GAS_MARKET_ACTOR_NAME: &str = "gas_market";
pub type Gas = u64;
pub type GasMarketReading = EIP1559GasState;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559GasState {
    pub block_gas_limit: Gas,
    pub base_fee: TokenAmount,
}

pub struct EIP1559GasMarketActor {}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    CurrentGasReading = frc42_dispatch::method_hash!("CurrentGasReading"),
    SetBlockGasLimit = frc42_dispatch::method_hash!("SetBlockGasLimit"),
    UpdateBlockGasConsumption = frc42_dispatch::method_hash!("UpdateBlockGasConsumption"),
}

impl EIP1559GasMarketActor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, st: EIP1559GasState) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.create(&st)?;

        Ok(())
    }

    fn set_block_gas_limit(rt: &impl Runtime, block_gas_limit: Gas) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            st.block_gas_limit = block_gas_limit;
            Ok(())
        })?;

        Ok(())
    }

    fn current_gas_reading(rt: &impl Runtime) -> Result<GasMarketReading, ActorError> {
        rt.state()
    }

    fn update_block_gas_consumption(
        rt: &impl Runtime,
        block_gas_used: Gas,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            st.base_fee = update_base_fee(st.block_gas_limit, block_gas_used, st.base_fee.clone());
            Ok(())
        })
    }
}

fn update_base_fee(gas_limit: Gas, gas_used: Gas, base_fee: TokenAmount) -> TokenAmount {
    let gas_target = gas_limit / ELASTICITY_MULTIPLIER;

    if gas_used == gas_target {
        return base_fee;
    }

    if gas_used > gas_target {
        let gas_used_delta = gas_used - gas_target;
        let base_fee_delta = base_fee
            .clone()
            .mul(gas_used_delta)
            .div_floor(gas_target)
            .div_floor(BASE_FEE_MAX_CHANGE_DENOMINATOR)
            .max(TokenAmount::from_atto(1));
        base_fee + base_fee_delta
    } else {
        let gas_used_delta = gas_target - gas_used;
        let base_fee_per_gas_delta = base_fee
            .clone()
            .mul(gas_used_delta)
            .div_floor(gas_target)
            .div_floor(BASE_FEE_MAX_CHANGE_DENOMINATOR);
        if base_fee_per_gas_delta > base_fee {
            TokenAmount::zero()
        } else {
            base_fee - base_fee_per_gas_delta
        }
    }
}

impl ActorCode for EIP1559GasMarketActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_GAS_MARKET_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        SetBlockGasLimit => set_block_gas_limit,
        CurrentGasReading => get_state,
        UpdateBlockGasConsumption => update_block_gas_consumption,
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
                    base_fee: Default::default(),
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
            Method::SetBlockGasLimit as u64,
            IpldBlock::serialize_cbor(&20).unwrap(),
        );
        assert!(r.is_ok());

        let r = rt
            .call::<EIP1559GasMarketActor>(
                Method::CurrentGasReading as u64,
                IpldBlock::serialize_cbor(&()).unwrap(),
            )
            .unwrap();
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
                Method::SetBlockGasLimit as u64,
                IpldBlock::serialize_cbor(&20).unwrap(),
            )
            .unwrap_err()
            .exit_code();
        assert_eq!(code, ExitCode::USR_FORBIDDEN)
    }
}
