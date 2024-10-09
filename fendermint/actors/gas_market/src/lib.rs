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
pub type SetConstants = EIP1559Constants;

/// Constant params used by EIP1559
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559Constants {
    pub block_gas_limit: Gas,
    /// The minimal base fee when gas utilization is low
    pub minimal_base_fee: TokenAmount,
    /// Elasticity multiplier as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
    pub elasticity_multiplier: u64,
    /// Base fee max change denominator as defined in [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559)
    pub base_fee_max_change_denominator: u64,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct EIP1559GasState {
    base_fee: TokenAmount,
    constants: EIP1559Constants,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct GasActorConstructorParams {
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
pub struct BlockGasUtilizationRet {
    pub base_fee: TokenAmount,
}

pub struct EIP1559GasMarketActor {}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    CurrentReading = frc42_dispatch::method_hash!("CurrentReading"),
    GetConstants = frc42_dispatch::method_hash!("GetConstants"),
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
            st.constants = constants;
            Ok(())
        })?;

        Ok(())
    }

    fn current_reading(rt: &impl Runtime) -> Result<GasMarketReading, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st = rt.state::<EIP1559GasState>()?;
        Ok(GasMarketReading {
            block_gas_limit: st.constants.block_gas_limit,
            base_fee: st.base_fee,
        })
    }

    fn get_constants(rt: &impl Runtime) -> Result<EIP1559Constants, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st = rt.state::<EIP1559GasState>()?;
        Ok(st.constants)
    }

    fn update_utilization(
        rt: &impl Runtime,
        utilization: BlockGasUtilization,
    ) -> Result<BlockGasUtilizationRet, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut EIP1559GasState, _rt| {
            st.base_fee = st.next_base_fee(utilization.block_gas_used);
            Ok(BlockGasUtilizationRet {
                base_fee: st.base_fee.clone(),
            })
        })
    }
}

impl Default for EIP1559Constants {
    fn default() -> Self {
        Self {
            // Take from filecoin setting, fvm_shared::BLOCK_GAS_LIMIT
            block_gas_limit: 10_000_000_000,
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
            base_fee: params.base_fee,
            constants: params.constants.unwrap_or_default(),
        }
    }
}

impl GasActorConstructorParams {
    pub fn new(base_fee: TokenAmount) -> Self {
        Self {
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
    fn next_base_fee(&self, gas_used: Gas) -> TokenAmount {
        let base_fee = self.base_fee.clone();
        let gas_target = self.constants.block_gas_limit / self.constants.elasticity_multiplier;

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
        GetConstants => get_constants,
        UpdateUtilization => update_utilization,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BlockGasUtilization, EIP1559Constants, EIP1559GasMarketActor, EIP1559GasState,
        GasActorConstructorParams, GasMarketReading, Method,
    };
    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::econ::TokenAmount;
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
                    base_fee: TokenAmount::from_atto(100),
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
            IpldBlock::serialize_cbor(&EIP1559Constants {
                minimal_base_fee: Default::default(),
                elasticity_multiplier: 0,
                base_fee_max_change_denominator: 0,
                block_gas_limit: 20,
            })
            .unwrap(),
        );
        assert!(r.is_ok());

        let s = rt.get_state::<EIP1559GasState>();
        assert_eq!(s.constants.block_gas_limit, 20);
    }

    #[test]
    fn test_update_utilization_full_usage() {
        let rt = construct_and_verify();

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let r = rt.call::<EIP1559GasMarketActor>(
            Method::UpdateUtilization as u64,
            IpldBlock::serialize_cbor(&BlockGasUtilization {
                // full block usage
                block_gas_used: 10_000_000_000,
            })
            .unwrap(),
        );
        assert!(r.is_ok());

        rt.expect_validate_caller_any();
        let r = rt
            .call::<EIP1559GasMarketActor>(Method::CurrentReading as u64, None)
            .unwrap()
            .unwrap();
        let reading = r.deserialize::<GasMarketReading>().unwrap();
        assert_eq!(reading.base_fee, TokenAmount::from_atto(112));
    }

    #[test]
    fn test_update_utilization_equal_usage() {
        let rt = construct_and_verify();

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let r = rt.call::<EIP1559GasMarketActor>(
            Method::UpdateUtilization as u64,
            IpldBlock::serialize_cbor(&BlockGasUtilization {
                // full block usage
                block_gas_used: 5_000_000_000,
            })
            .unwrap(),
        );
        assert!(r.is_ok());

        rt.expect_validate_caller_any();
        let r = rt
            .call::<EIP1559GasMarketActor>(Method::CurrentReading as u64, None)
            .unwrap()
            .unwrap();
        let reading = r.deserialize::<GasMarketReading>().unwrap();
        assert_eq!(reading.base_fee, TokenAmount::from_atto(100));
    }

    #[test]
    fn test_update_utilization_under_usage() {
        let rt = construct_and_verify();

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let r = rt.call::<EIP1559GasMarketActor>(
            Method::UpdateUtilization as u64,
            IpldBlock::serialize_cbor(&BlockGasUtilization {
                // full block usage
                block_gas_used: 100_000_000,
            })
            .unwrap(),
        );
        assert!(r.is_ok());

        rt.expect_validate_caller_any();
        let r = rt
            .call::<EIP1559GasMarketActor>(Method::CurrentReading as u64, None)
            .unwrap()
            .unwrap();
        let reading = r.deserialize::<GasMarketReading>().unwrap();
        assert_eq!(reading.base_fee, TokenAmount::from_atto(88));
    }

    #[test]
    fn test_not_allowed() {
        let rt = construct_and_verify();
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, Address::new_id(1000));
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let code = rt
            .call::<EIP1559GasMarketActor>(
                Method::SetConstants as u64,
                IpldBlock::serialize_cbor(&EIP1559Constants {
                    minimal_base_fee: TokenAmount::from_atto(10000),
                    elasticity_multiplier: 0,
                    base_fee_max_change_denominator: 0,
                    block_gas_limit: 20,
                })
                .unwrap(),
            )
            .unwrap_err()
            .exit_code();
        assert_eq!(code, ExitCode::USR_FORBIDDEN)
    }
}
