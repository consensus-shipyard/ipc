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
use std::cmp::Ordering;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(EIP1559GasMarketActor);

pub const IPC_GAS_MARKET_ACTOR_NAME: &str = "gas_market";
pub type Gas = u64;

/// Constant params used by EIP1559
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559Constants {
    /// The minimal base fee when gas utilization is low
    minimal_base_fee: TokenAmount,
    /// Elasticity multiplier as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
    elasticity_multiplier: u64,
    /// Base fee max change denominator as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
    base_fee_max_change_denominator: u64,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559GasState {
    block_gas_limit: Gas,
    base_fee: TokenAmount,
    constants: EIP1559Constants,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct GasActorConstructorParams {
    block_gas_limit: Gas,
    base_fee: TokenAmount,
    constants: Option<EIP1559Constants>,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct GasMarketReading {
    pub block_gas_limit: Gas,
    pub base_fee: TokenAmount,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct BlockGasUtilization {
    pub block_gas_used: Gas,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct SetConstants {
    pub block_gas_limit: Option<Gas>,
    pub base_fee: Option<TokenAmount>,
    pub constants: Option<EIP1559Constants>,
}

pub struct EIP1559GasMarketActor {}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    CurrentReading = frc42_dispatch::method_hash!("CurrentReading"),
    SetConstants = frc42_dispatch::method_hash!("SetConstants"),
    UpdateUtilization = frc42_dispatch::method_hash!("UpdateUtilization"),
}

impl EIP1559GasMarketActor {
    /// Creates the actor
    pub fn constructor(
        rt: &impl Runtime,
        params: GasActorConstructorParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let st = EIP1559GasState::from(params);
        rt.create(&st)
    }

    fn set_constants(rt: &impl Runtime, constants: SetConstants) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            st.set_constants(constants);
            Ok(())
        })?;

        Ok(())
    }

    fn current_reading(rt: &impl Runtime) -> Result<GasMarketReading, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st = rt.state::<EIP1559GasState>()?;
        Ok(GasMarketReading {
            block_gas_limit: st.block_gas_limit,
            base_fee: st.base_fee,
        })
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

impl Default for EIP1559Constants {
    fn default() -> Self {
        Self {
            // Take from filecoin setting
            minimal_base_fee: TokenAmount::from_atto(100),
            // Elasticity multiplier as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
            elasticity_multiplier: 2,
            // Base fee max change denominator as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
            base_fee_max_change_denominator: 8,
        }
    }
}

impl From<GasActorConstructorParams> for EIP1559GasState {
    fn from(params: GasActorConstructorParams) -> Self {
        Self {
            block_gas_limit: params.block_gas_limit,
            base_fee: params.base_fee,
            constants: params.constants.unwrap_or_default(),
        }
    }
}

impl GasActorConstructorParams {
    pub fn new(block_gas_limit: Gas, base_fee: TokenAmount) -> Self {
        Self {
            block_gas_limit,
            base_fee,
            constants: None,
        }
    }

    pub fn with_constants(mut self, constants: EIP1559Constants) -> Self {
        self.constants = Some(constants);
        self
    }
}

impl EIP1559GasState {
    #[inline]
    fn update_if_some<T>(maybe_some: Option<T>, to_change: &mut T) {
        if let Some(v) = maybe_some {
            *to_change = v;
        }
    }

    fn set_constants(&mut self, constants: SetConstants) {
        Self::update_if_some(constants.constants, &mut self.constants);
        Self::update_if_some(constants.base_fee, &mut self.base_fee);
        Self::update_if_some(constants.block_gas_limit, &mut self.block_gas_limit);
    }

    fn next_base_fee(&self, gas_used: Gas) -> TokenAmount {
        let base_fee = self.base_fee.clone();
        let gas_target = self.block_gas_limit / self.constants.elasticity_multiplier;

        match gas_used.cmp(&gas_target) {
            Ordering::Equal => base_fee,
            Ordering::Less => {
                let base_fee_delta = base_fee.atto() * (gas_target - gas_used)
                    / gas_target
                    / self.constants.base_fee_max_change_denominator;
                let base_fee_delta = TokenAmount::from_atto(base_fee_delta);
                if base_fee_delta >= base_fee {
                    self.constants.minimal_base_fee.clone()
                } else {
                    base_fee - base_fee_delta
                }
            }
            Ordering::Greater => {
                let gas_used_delta = gas_used - gas_target;
                let delta = base_fee.atto() * gas_used_delta
                    / gas_target
                    / self.constants.base_fee_max_change_denominator;
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
        CurrentReading => current_reading,
        UpdateUtilization => update_utilization,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EIP1559GasMarketActor, EIP1559GasState, GasActorConstructorParams, Method, SetConstants,
    };
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
                IpldBlock::serialize_cbor(&GasActorConstructorParams {
                    block_gas_limit: 100,
                    base_fee: Default::default(),
                    constants: None,
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
            Method::SetConstants as u64,
            IpldBlock::serialize_cbor(&SetConstants {
                block_gas_limit: Some(20),
                base_fee: None,
                constants: None,
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
                Method::SetConstants as u64,
                IpldBlock::serialize_cbor(&SetConstants {
                    block_gas_limit: Some(20),
                    base_fee: None,
                    constants: None,
                })
                .unwrap(),
            )
            .unwrap_err()
            .exit_code();
        assert_eq!(code, ExitCode::USR_FORBIDDEN)
    }
}
