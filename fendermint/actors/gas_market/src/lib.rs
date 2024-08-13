// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::econ::TokenAmount;
use fvm_shared::METHOD_CONSTRUCTOR;
use lazy_static::lazy_static;
use num_derive::FromPrimitive;
use std::cmp::Ordering;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(EIP1559GasMarketActor);

/// Base fee max change denominator as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
const BASE_FEE_MAX_CHANGE_DENOMINATOR: u64 = 8;
/// Elasticity multiplier as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
const ELASTICITY_MULTIPLIER: u64 = 2;
lazy_static! {
    /// Initial base fee as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
    static ref INITIAL_BASE_FEE: TokenAmount = TokenAmount::from_atto(1_000_000_000);
    static ref MINIMAL_BASE_FEE: TokenAmount = TokenAmount::from_atto(100);
}
pub const IPC_GAS_MARKET_ACTOR_NAME: &str = "gas_market";
pub type Gas = u64;
pub type GasMarketReading = EIP1559GasState;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559GasState {
    pub block_gas_limit: Gas,
    pub base_fee: TokenAmount,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct BlockGasUtilization {
    pub block_gas_used: Gas,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct SetConstants {
    pub block_gas_limit: Gas,
}

pub struct EIP1559GasMarketActor {}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    CurrentGasReading = frc42_dispatch::method_hash!("CurrentGasReading"),
    SetConstants = frc42_dispatch::method_hash!("SetConstants"),
    UpdateUtilization = frc42_dispatch::method_hash!("UpdateUtilization"),
}

impl EIP1559GasMarketActor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, st: EIP1559GasState) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.create(&st)
    }

    fn set_constants(rt: &impl Runtime, constants: SetConstants) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            st.block_gas_limit = constants.block_gas_limit;
            Ok(())
        })?;

        Ok(())
    }

    fn current_gas_reading(rt: &impl Runtime) -> Result<GasMarketReading, ActorError> {
        rt.state()
    }

    fn update_utilization(
        rt: &impl Runtime,
        utilization: BlockGasUtilization,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            st.base_fee = st.next_base_fee(utilization.block_gas_used);
            Ok(())
        })
    }
}

impl EIP1559GasState {
    fn next_base_fee(&self, gas_used: Gas) -> TokenAmount {
        let base_fee = self.base_fee.clone();
        let gas_target = self.block_gas_limit / ELASTICITY_MULTIPLIER;

        match gas_used.cmp(&gas_target) {
            Ordering::Equal => base_fee,
            Ordering::Less => {
                let base_fee_delta = base_fee.atto() * (gas_target - gas_used)
                    / gas_target
                    / BASE_FEE_MAX_CHANGE_DENOMINATOR;
                let base_fee_delta = TokenAmount::from_atto(base_fee_delta);
                if base_fee_delta >= base_fee {
                    MINIMAL_BASE_FEE.clone()
                } else {
                    base_fee - base_fee_delta
                }
            }
            Ordering::Greater => {
                let gas_used_delta = gas_used - gas_target;
                let delta =
                    base_fee.atto() * gas_used_delta / gas_target / BASE_FEE_MAX_CHANGE_DENOMINATOR;
                base_fee + TokenAmount::from_atto(delta).max(TokenAmount::from_atto(1))
            }
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
        SetConstants => set_constants,
        CurrentGasReading => current_gas_reading,
        UpdateUtilization => update_utilization,
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
