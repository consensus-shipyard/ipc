// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_hoku_config_shared::{HokuConfig, Method, SetAdminParams, SetConfigParams};
use fendermint_actor_machine::resolve_external;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub const ACTOR_NAME: &str = "hoku_config";

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    /// The admin address that is allowed to update the config.
    pub admin: Option<Address>,
    /// The Hoku network configuration.
    pub config: HokuConfig,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct ConstructorParams {
    initial_blob_capacity: u64,
    initial_token_credit_rate: u64,
    initial_blob_credit_debit_interval: ChainEpoch,
}

pub struct Actor {}

impl Actor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let st = State {
            admin: None,
            config: HokuConfig {
                blob_capacity: params.initial_blob_capacity,
                token_credit_rate: params.initial_token_credit_rate,
                blob_credit_debit_interval: params.initial_blob_credit_debit_interval,
            },
        };
        rt.create(&st)
    }

    fn set_admin(rt: &impl Runtime, params: SetAdminParams) -> Result<(), ActorError> {
        Self::ensure_update_allowed(rt)?;
        let (new_admin, _) = resolve_external(rt, params.0)?;
        rt.transaction(|st: &mut State, _rt| {
            st.admin = Some(new_admin);
            Ok(())
        })
    }

    fn get_admin(rt: &impl Runtime) -> Result<Option<Address>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>().map(|s| s.admin)
    }

    fn set_config(rt: &impl Runtime, params: SetConfigParams) -> Result<(), ActorError> {
        let admin_exists = Self::ensure_update_allowed(rt)?;
        let new_admin = if !admin_exists {
            // The first caller becomes admin
            let (new_admin, _) = resolve_external(rt, rt.message().caller())?;
            Some(new_admin)
        } else {
            None
        };
        rt.transaction(|st: &mut State, _rt| {
            if let Some(new_admin) = new_admin {
                st.admin = Some(new_admin);
            }
            st.config = params;
            Ok(())
        })?;

        Ok(())
    }

    fn get_config(rt: &impl Runtime) -> Result<HokuConfig, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>().map(|s| s.config)
    }

    /// Ensures that immediate caller is allowed to update the config.
    /// Returns whether the admin exists.
    fn ensure_update_allowed(rt: &impl Runtime) -> Result<bool, ActorError> {
        let st = rt.state::<State>()?;
        let admin_exists = if let Some(admin) = st.admin {
            if let Some(admin_id) = rt.resolve_address(&admin) {
                rt.validate_immediate_caller_is(std::iter::once(&Address::new_id(admin_id)))?
            } else {
                // This should not happen.
                return Err(ActorError::forbidden(String::from(
                    "failed to resolve config admin id",
                )));
            }
            true
        } else {
            // The first caller becomes the admin
            rt.validate_immediate_caller_accept_any()?;
            false
        };
        Ok(admin_exists)
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        SetAdmin => set_admin,
        GetAdmin => get_admin,
        SetConfig => set_config,
        GetConfig => get_config,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Actor, ConstructorParams, Method};
    use fendermint_actor_hoku_config_shared::{HokuConfig, HOKU_CONFIG_ACTOR_ID};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::clock::ChainEpoch;

    pub fn construct_and_verify(
        token_credit_rate: u64,
        blob_capacity: u64,
        blob_credit_debit_interval: i32,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(HOKU_CONFIG_ACTOR_ID),
            ..Default::default()
        };

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let result = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    initial_token_credit_rate: token_credit_rate,
                    initial_blob_capacity: blob_capacity,
                    initial_blob_credit_debit_interval: ChainEpoch::from(
                        blob_credit_debit_interval,
                    ),
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
    fn test_get_config() {
        let rt = construct_and_verify(5, 1024, 3600);

        rt.expect_validate_caller_any();
        let hoku_config = rt
            .call::<Actor>(Method::GetConfig as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<HokuConfig>()
            .unwrap();

        assert_eq!(hoku_config.token_credit_rate, 5);
        assert_eq!(hoku_config.blob_capacity, 1024);
        assert_eq!(hoku_config.blob_credit_debit_interval, 3600);
    }

    #[test]
    fn test_set_config() {
        let rt = construct_and_verify(5, 1024, 3600);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);

        rt.expect_validate_caller_any();
        let result = rt.call::<Actor>(
            Method::SetConfig as u64,
            IpldBlock::serialize_cbor(&HokuConfig {
                blob_capacity: 2048,
                token_credit_rate: 10,
                blob_credit_debit_interval: ChainEpoch::from(1800),
            })
            .unwrap(),
        );
        assert!(result.is_ok());

        rt.expect_validate_caller_any();
        let hoku_config = rt
            .call::<Actor>(Method::GetConfig as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<HokuConfig>()
            .unwrap();

        assert_eq!(hoku_config.token_credit_rate, 10);
        assert_eq!(hoku_config.blob_capacity, 2048);
        assert_eq!(hoku_config.blob_credit_debit_interval, 1800);
    }
}
