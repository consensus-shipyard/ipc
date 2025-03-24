// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::TokenCreditRate;
use fendermint_actor_recall_config_shared::{
    Method, RecallConfig, SetAdminParams, SetConfigParams,
};
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::bigint::BigUint;
use fvm_shared::{address::Address, clock::ChainEpoch};
use num_traits::Zero;
use recall_actor_sdk::{emit_evm_event, to_delegated_address, to_id_and_delegated_address};

use crate::sol_facade::{ConfigAdminSet, ConfigSet};

mod sol_facade;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub const ACTOR_NAME: &str = "recall_config";

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    /// The admin address that is allowed to update the config.
    pub admin: Option<Address>,
    /// The Recall network configuration.
    pub config: RecallConfig,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct ConstructorParams {
    initial_blob_capacity: u64,
    initial_token_credit_rate: TokenCreditRate,
    initial_blob_credit_debit_interval: ChainEpoch,
    initial_blob_min_ttl: ChainEpoch,
    initial_blob_default_ttl: ChainEpoch,
    initial_blob_delete_batch_size: u64,
    initial_account_debit_batch_size: u64,
}

pub struct Actor {}

impl Actor {
    /// Creates the actor
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let st = State {
            admin: None,
            config: RecallConfig {
                blob_capacity: params.initial_blob_capacity,
                token_credit_rate: params.initial_token_credit_rate,
                blob_credit_debit_interval: params.initial_blob_credit_debit_interval,
                blob_min_ttl: params.initial_blob_min_ttl,
                blob_default_ttl: params.initial_blob_default_ttl,
                blob_delete_batch_size: params.initial_blob_delete_batch_size,
                account_debit_batch_size: params.initial_account_debit_batch_size,
            },
        };
        rt.create(&st)
    }

    fn set_admin(rt: &impl Runtime, params: SetAdminParams) -> Result<(), ActorError> {
        Self::ensure_update_allowed(rt)?;

        let (admin_id_addr, admin_delegated_addr) = to_id_and_delegated_address(rt, params.0)?;

        rt.transaction(|st: &mut State, _rt| {
            st.admin = Some(admin_id_addr);
            Ok(())
        })?;

        emit_evm_event(rt, ConfigAdminSet::new(admin_delegated_addr))?;

        Ok(())
    }

    fn get_admin(rt: &impl Runtime) -> Result<Option<Address>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        match rt.state::<State>().map(|s| s.admin)? {
            Some(admin) => {
                let admin = to_delegated_address(rt, admin)?;
                Ok(Some(admin))
            }
            None => Ok(None),
        }
    }

    fn set_config(rt: &impl Runtime, params: SetConfigParams) -> Result<(), ActorError> {
        let admin_exists = Self::ensure_update_allowed(rt)?;

        if params.token_credit_rate.rate() <= &BigUint::zero() {
            return Err(actor_error!(
                illegal_argument,
                "token credit rate must be positive"
            ));
        }
        if params.blob_capacity == 0 {
            return Err(actor_error!(
                illegal_argument,
                "blob capacity must be positive"
            ));
        }
        if params.blob_credit_debit_interval <= 0 {
            return Err(actor_error!(
                illegal_argument,
                "credit debit interval must be positive"
            ));
        }
        if params.blob_min_ttl <= 0 {
            return Err(actor_error!(
                illegal_argument,
                "minimum TTL must be positive"
            ));
        }
        if params.blob_default_ttl <= 0 {
            return Err(actor_error!(
                illegal_argument,
                "default TTL must be positive"
            ));
        }
        if params.blob_default_ttl < params.blob_min_ttl {
            return Err(actor_error!(
                illegal_argument,
                "default TTL must be greater than or equal to minimum TTL"
            ));
        }
        if params.blob_delete_batch_size == 0 {
            return Err(actor_error!(
                illegal_argument,
                "blob delete batch size must be positive"
            ));
        }
        if params.account_debit_batch_size == 0 {
            return Err(actor_error!(
                illegal_argument,
                "account debit batch size must be positive"
            ));
        }

        let (admin_id_addr, admin_delegated_addr) = if !admin_exists {
            // The first caller becomes admin
            let addrs = to_id_and_delegated_address(rt, rt.message().caller())?;
            (Some(addrs.0), Some(addrs.1))
        } else {
            (None, None)
        };

        rt.transaction(|st: &mut State, _rt| {
            if let Some(admin) = admin_id_addr {
                st.admin = Some(admin);
            }
            st.config = params.clone();
            Ok(())
        })?;

        if let Some(admin) = admin_delegated_addr {
            emit_evm_event(rt, ConfigAdminSet::new(admin))?;
        }
        emit_evm_event(
            rt,
            ConfigSet {
                blob_capacity: params.blob_capacity,
                token_credit_rate: params.token_credit_rate,
                blob_credit_debit_interval: params.blob_credit_debit_interval,
                blob_min_ttl: params.blob_min_ttl,
                blob_default_ttl: params.blob_default_ttl,
                blob_delete_batch_size: params.blob_delete_batch_size,
                account_debit_batch_size: params.account_debit_batch_size,
            },
        )?;

        Ok(())
    }

    fn get_config(rt: &impl Runtime) -> Result<RecallConfig, ActorError> {
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
    use super::*;

    use fendermint_actor_recall_config_shared::{RecallConfig, RECALL_CONFIG_ACTOR_ID};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::error::ExitCode;
    use recall_actor_sdk::to_actor_event;

    pub fn construct_and_verify(
        blob_capacity: u64,
        token_credit_rate: TokenCreditRate,
        blob_credit_debit_interval: i32,
        initial_blob_min_ttl: ChainEpoch,
        initial_blob_default_ttl: ChainEpoch,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(RECALL_CONFIG_ACTOR_ID),
            ..Default::default()
        };

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let result = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    initial_blob_capacity: blob_capacity,
                    initial_token_credit_rate: token_credit_rate,
                    initial_blob_credit_debit_interval: ChainEpoch::from(
                        blob_credit_debit_interval,
                    ),
                    initial_blob_min_ttl,
                    initial_blob_default_ttl,
                    initial_blob_delete_batch_size: 100,
                    initial_account_debit_batch_size: 100,
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
    fn test_get_initial_admin() {
        let rt = construct_and_verify(1024, TokenCreditRate::from(5usize), 3600, 3600, 3600);

        rt.expect_validate_caller_any();
        let admin = rt
            .call::<Actor>(Method::GetAdmin as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<Option<Address>>()
            .unwrap();
        rt.verify();

        assert!(admin.is_none());
    }

    #[test]
    fn test_set_admin() {
        let rt = construct_and_verify(1024, TokenCreditRate::from(5usize), 3600, 3600, 3600);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);

        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.expect_validate_caller_any();
        let event = to_actor_event(ConfigAdminSet::new(f4_eth_addr)).unwrap();
        rt.expect_emitted_event(event);
        let result = rt.call::<Actor>(
            Method::SetAdmin as u64,
            IpldBlock::serialize_cbor(&SetAdminParams(f4_eth_addr)).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        rt.expect_validate_caller_any();
        let admin = rt
            .call::<Actor>(Method::GetAdmin as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<Option<Address>>()
            .unwrap();
        rt.verify();

        assert_eq!(admin, Some(f4_eth_addr));

        // Reset admin
        let new_id_addr = Address::new_id(111);
        let new_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let new_f4_eth_addr = Address::new_delegated(10, &new_eth_addr.0).unwrap();
        rt.set_delegated_address(new_id_addr.id().unwrap(), new_f4_eth_addr);

        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr); // current admin
        rt.expect_validate_caller_addr(vec![id_addr]);
        let event = to_actor_event(ConfigAdminSet::new(new_f4_eth_addr)).unwrap();
        rt.expect_emitted_event(event);
        let result = rt.call::<Actor>(
            Method::SetAdmin as u64,
            IpldBlock::serialize_cbor(&SetAdminParams(new_f4_eth_addr)).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        rt.expect_validate_caller_any();
        let admin = rt
            .call::<Actor>(Method::GetAdmin as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<Option<Address>>()
            .unwrap();
        rt.verify();

        assert_eq!(admin, Some(new_f4_eth_addr));
    }

    #[test]
    fn test_set_admin_unauthorized() {
        let rt = construct_and_verify(1024, TokenCreditRate::from(5usize), 3600, 3600, 3600);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);

        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.expect_validate_caller_any();
        let event = to_actor_event(ConfigAdminSet::new(f4_eth_addr)).unwrap();
        rt.expect_emitted_event(event);
        let result = rt.call::<Actor>(
            Method::SetAdmin as u64,
            IpldBlock::serialize_cbor(&SetAdminParams(f4_eth_addr)).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Try to set again with a different caller
        let unauthorized_id_addr = Address::new_id(111);
        let unauthorized_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let unauthorized_f4_eth_addr =
            Address::new_delegated(10, &unauthorized_eth_addr.0).unwrap();
        rt.set_delegated_address(unauthorized_id_addr.id().unwrap(), unauthorized_f4_eth_addr);

        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, unauthorized_id_addr); // unauthorized caller
        rt.expect_validate_caller_addr(vec![id_addr]); // expect current admin
        let result = rt.call::<Actor>(
            Method::SetAdmin as u64,
            IpldBlock::serialize_cbor(&SetAdminParams(unauthorized_f4_eth_addr)).unwrap(),
        );
        rt.verify();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().exit_code(), ExitCode::USR_FORBIDDEN);
    }

    #[test]
    fn test_set_config() {
        let rt = construct_and_verify(1024, TokenCreditRate::from(5usize), 3600, 3600, 3600);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);

        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.expect_validate_caller_any();

        let admin_event = to_actor_event(ConfigAdminSet::new(f4_eth_addr)).unwrap();
        rt.expect_emitted_event(admin_event);

        let config = RecallConfig {
            blob_capacity: 2048,
            token_credit_rate: TokenCreditRate::from(10usize),
            blob_credit_debit_interval: ChainEpoch::from(1800),
            blob_min_ttl: ChainEpoch::from(2 * 60 * 60),
            blob_default_ttl: ChainEpoch::from(24 * 60 * 60),
            blob_delete_batch_size: 100,
            account_debit_batch_size: 100,
        };
        let config_event = to_actor_event(ConfigSet {
            blob_capacity: config.blob_capacity,
            token_credit_rate: config.token_credit_rate.clone(),
            blob_credit_debit_interval: config.blob_credit_debit_interval,
            blob_min_ttl: config.blob_min_ttl,
            blob_default_ttl: config.blob_default_ttl,
            blob_delete_batch_size: config.blob_delete_batch_size,
            account_debit_batch_size: config.account_debit_batch_size,
        })
        .unwrap();
        rt.expect_emitted_event(config_event);

        let result = rt.call::<Actor>(
            Method::SetConfig as u64,
            IpldBlock::serialize_cbor(&config).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        rt.expect_validate_caller_any();
        let recall_config = rt
            .call::<Actor>(Method::GetConfig as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<RecallConfig>()
            .unwrap();
        rt.verify();

        assert_eq!(recall_config.blob_capacity, 2048);
        assert_eq!(
            recall_config.token_credit_rate,
            TokenCreditRate::from(10usize)
        );
        assert_eq!(recall_config.blob_credit_debit_interval, 1800);
        assert_eq!(recall_config.blob_min_ttl, ChainEpoch::from(2 * 60 * 60));
        assert_eq!(
            recall_config.blob_default_ttl,
            ChainEpoch::from(24 * 60 * 60)
        );

        rt.expect_validate_caller_any();
        let admin = rt
            .call::<Actor>(Method::GetAdmin as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<Option<Address>>()
            .unwrap();
        rt.verify();

        assert_eq!(admin, Some(f4_eth_addr));
    }

    #[test]
    fn test_set_invalid_config() {
        struct TestCase {
            name: &'static str,
            config: RecallConfig,
        }

        let valid_config = RecallConfig {
            blob_capacity: 2048,
            token_credit_rate: TokenCreditRate::from(10usize),
            blob_credit_debit_interval: ChainEpoch::from(1800),
            blob_min_ttl: ChainEpoch::from(2 * 60 * 60),
            blob_default_ttl: ChainEpoch::from(24 * 60 * 60),
            blob_delete_batch_size: 100,
            account_debit_batch_size: 100,
        };

        let test_cases = vec![
            // Token credit rate validation
            TestCase {
                name: "token credit rate cannot be zero",
                config: RecallConfig {
                    token_credit_rate: TokenCreditRate::from(0usize),
                    ..valid_config.clone()
                },
            },
            // Blob capacity validation
            TestCase {
                name: "blob capacity cannot be zero",
                config: RecallConfig {
                    blob_capacity: 0,
                    ..valid_config.clone()
                },
            },
            // Credit debit interval validation
            TestCase {
                name: "blob credit debit interval cannot be zero",
                config: RecallConfig {
                    blob_credit_debit_interval: 0,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob credit debit interval cannot be negative",
                config: RecallConfig {
                    blob_credit_debit_interval: -1,
                    ..valid_config.clone()
                },
            },
            // TTL validations
            TestCase {
                name: "blob min ttl cannot be negative",
                config: RecallConfig {
                    blob_min_ttl: -1,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob min ttl cannot be zero",
                config: RecallConfig {
                    blob_min_ttl: 0,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob default ttl must be greater than or equal to min ttl",
                config: RecallConfig {
                    blob_min_ttl: 4 * 60 * 60,
                    blob_default_ttl: 2 * 60 * 60,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob default ttl cannot be zero",
                config: RecallConfig {
                    blob_default_ttl: 0,
                    ..valid_config.clone()
                },
            },
            TestCase {
                name: "blob default ttl cannot be negative",
                config: RecallConfig {
                    blob_default_ttl: -1,
                    ..valid_config.clone()
                },
            },
        ];

        let rt = construct_and_verify(1024, TokenCreditRate::from(5usize), 3600, 3600, 3600);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);

        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);

        // Now test all invalid configurations
        for test_case in test_cases {
            rt.expect_validate_caller_any();
            let result = rt.call::<Actor>(
                Method::SetConfig as u64,
                IpldBlock::serialize_cbor(&test_case.config).unwrap(),
            );
            rt.verify();
            assert!(
                result.is_err(),
                "expected case \"{}\" to fail but it succeeded",
                test_case.name
            );
        }
    }

    #[test]
    fn test_get_config() {
        let rt = construct_and_verify(1024, TokenCreditRate::from(5usize), 3600, 3600, 3600);

        rt.expect_validate_caller_any();
        let recall_config = rt
            .call::<Actor>(Method::GetConfig as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<RecallConfig>()
            .unwrap();
        rt.verify();

        assert_eq!(recall_config.blob_capacity, 1024);
        assert_eq!(
            recall_config.token_credit_rate,
            TokenCreditRate::from(5usize)
        );
        assert_eq!(recall_config.blob_credit_debit_interval, 3600);
        assert_eq!(recall_config.blob_min_ttl, 3600);
        assert_eq!(recall_config.blob_default_ttl, 3600);
    }
}
