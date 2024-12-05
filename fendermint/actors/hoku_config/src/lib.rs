// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub const ACTOR_NAME: &str = "hoku_config";

pub type SetConfigParams = Config;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    pub admin: Option<Address>,
    pub config: Config,
}

/// The updatable config.
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct Config {
    pub blob_capacity: u64,
    pub blob_credit_debit_rate: u64,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct ConstructorParams {
    initial_blob_capacity: u64,
    initial_blob_credit_debit_rate: u64,
}

pub struct Actor {}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    SetConfig = frc42_dispatch::method_hash!("SetConfig"),
    GetConfig = frc42_dispatch::method_hash!("GetConfig"),
}

impl Actor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let st = State {
            admin: None,
            config: Config {
                blob_capacity: params.initial_blob_capacity,
                blob_credit_debit_rate: params.initial_blob_credit_debit_rate,
            },
        };

        rt.create(&st)
    }

    fn set_config(rt: &impl Runtime, params: SetConfigParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, _rt| {
            st.config = params;
            Ok(())
        })?;

        Ok(())
    }

    fn get_config(rt: &impl Runtime) -> Result<Config, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>().map(|s| s.config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blob_capacity: 0,
            blob_credit_debit_rate: 1,
        }
    }
}

impl State {
    // fn next_base_fee(&self, gas_used: Gas) -> TokenAmount {
    //     let base_fee = self.base_fee.clone();
    //     let gas_target = self.constants.block_gas_limit / self.constants.elasticity_multiplier;
    //
    //     match gas_used.cmp(&gas_target) {
    //         Ordering::Equal => base_fee,
    //         Ordering::Less => {
    //             let base_fee_delta = base_fee.atto() * (gas_target - gas_used)
    //                 / gas_target
    //                 / self.constants.base_fee_max_change_denominator;
    //             let base_fee_delta = TokenAmount::from_atto(base_fee_delta);
    //             if base_fee_delta >= base_fee {
    //                 self.constants.minimal_base_fee.clone()
    //             } else {
    //                 base_fee - base_fee_delta
    //             }
    //         }
    //         Ordering::Greater => {
    //             let gas_used_delta = gas_used - gas_target;
    //             let delta = base_fee.atto() * gas_used_delta
    //                 / gas_target
    //                 / self.constants.base_fee_max_change_denominator;
    //             base_fee + TokenAmount::from_atto(delta).max(TokenAmount::from_atto(1))
    //         }
    //     }
    // }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        SetConfig => set_config,
        GetConfig => get_config,
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{Actor, Constants, ConstructorParams, Method, State};
//     use fendermint_actors_api::gas_market::{Reading, Utilization};
//     use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
//     use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
//     use fvm_ipld_encoding::ipld_block::IpldBlock;
//     use fvm_shared::address::Address;
//     use fvm_shared::econ::TokenAmount;
//     use fvm_shared::error::ExitCode;
//
//     pub fn construct_and_verify() -> MockRuntime {
//         let rt = MockRuntime {
//             receiver: Address::new_id(10),
//             ..Default::default()
//         };
//
//         rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
//         rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
//
//         let result = rt
//             .call::<Actor>(
//                 Method::Constructor as u64,
//                 IpldBlock::serialize_cbor(&ConstructorParams {
//                     initial_base_fee: TokenAmount::from_atto(100),
//                     constants: Constants::default(),
//                 })
//                 .unwrap(),
//             )
//             .unwrap();
//         expect_empty(result);
//         rt.verify();
//         rt.reset();
//
//         rt
//     }
//
//     #[test]
//     fn test_set_ok() {
//         let rt = construct_and_verify();
//
//         rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
//         rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
//
//         let r = rt.call::<Actor>(
//             Method::SetConstants as u64,
//             IpldBlock::serialize_cbor(&Constants {
//                 minimal_base_fee: Default::default(),
//                 elasticity_multiplier: 0,
//                 base_fee_max_change_denominator: 0,
//                 block_gas_limit: 20,
//             })
//             .unwrap(),
//         );
//         assert!(r.is_ok());
//
//         let s = rt.get_state::<State>();
//         assert_eq!(s.constants.block_gas_limit, 20);
//     }
//
//     #[test]
//     fn test_update_utilization_full_usage() {
//         let rt = construct_and_verify();
//
//         rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
//         rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
//
//         let r = rt.call::<Actor>(
//             Method::UpdateUtilization as u64,
//             IpldBlock::serialize_cbor(&Utilization {
//                 // full block usage
//                 block_gas_used: 10_000_000_000,
//             })
//             .unwrap(),
//         );
//         assert!(r.is_ok());
//
//         rt.expect_validate_caller_any();
//         let r = rt
//             .call::<Actor>(Method::CurrentReading as u64, None)
//             .unwrap()
//             .unwrap();
//         let reading = r.deserialize::<Reading>().unwrap();
//         assert_eq!(reading.base_fee, TokenAmount::from_atto(112));
//     }
//
//     #[test]
//     fn test_update_utilization_equal_usage() {
//         let rt = construct_and_verify();
//
//         rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
//         rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
//
//         let r = rt.call::<Actor>(
//             Method::UpdateUtilization as u64,
//             IpldBlock::serialize_cbor(&Utilization {
//                 // full block usage
//                 block_gas_used: 5_000_000_000,
//             })
//             .unwrap(),
//         );
//         assert!(r.is_ok());
//
//         rt.expect_validate_caller_any();
//         let r = rt
//             .call::<Actor>(Method::CurrentReading as u64, None)
//             .unwrap()
//             .unwrap();
//         let reading = r.deserialize::<Reading>().unwrap();
//         assert_eq!(reading.base_fee, TokenAmount::from_atto(100));
//     }
//
//     #[test]
//     fn test_update_utilization_under_usage() {
//         let rt = construct_and_verify();
//
//         rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
//         rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
//
//         let r = rt.call::<Actor>(
//             Method::UpdateUtilization as u64,
//             IpldBlock::serialize_cbor(&Utilization {
//                 // full block usage
//                 block_gas_used: 100_000_000,
//             })
//             .unwrap(),
//         );
//         assert!(r.is_ok());
//
//         rt.expect_validate_caller_any();
//         let r = rt
//             .call::<Actor>(Method::CurrentReading as u64, None)
//             .unwrap()
//             .unwrap();
//         let reading = r.deserialize::<Reading>().unwrap();
//         assert_eq!(reading.base_fee, TokenAmount::from_atto(88));
//     }
//
//     #[test]
//     fn test_not_allowed() {
//         let rt = construct_and_verify();
//         rt.set_caller(*SYSTEM_ACTOR_CODE_ID, Address::new_id(1000));
//         rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
//
//         let code = rt
//             .call::<Actor>(
//                 Method::SetConstants as u64,
//                 IpldBlock::serialize_cbor(&Constants {
//                     minimal_base_fee: TokenAmount::from_atto(10000),
//                     elasticity_multiplier: 0,
//                     base_fee_max_change_denominator: 0,
//                     block_gas_limit: 20,
//                 })
//                 .unwrap(),
//             )
//             .unwrap_err()
//             .exit_code();
//         assert_eq!(code, ExitCode::USR_FORBIDDEN)
//     }
// }
