// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::state::{PowerEntryValue, State};
use crate::types::{ConstructorParams, GetStateResponse, PowerEntry, UpdateStateParams};
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch, actor_error, ActorError};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub mod state;
pub mod types;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(F3LightClientActor);

pub const F3_LIGHT_CLIENT_ACTOR_NAME: &str = "f3_light_client";

pub struct F3LightClientActor;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    UpdateState = frc42_dispatch::method_hash!("UpdateState"),
    GetState = frc42_dispatch::method_hash!("GetState"),
}

trait F3LightClient {
    /// Update light client state
    fn update_state(rt: &impl Runtime, params: UpdateStateParams) -> Result<(), ActorError>;

    /// Get current light client state
    fn get_state(rt: &impl Runtime) -> Result<GetStateResponse, ActorError>;
}

impl F3LightClientActor {
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store(), params.processed_instance_id, params.power_table)?;

        rt.create(&state)?;
        Ok(())
    }
}

impl F3LightClient for F3LightClientActor {
    fn update_state(rt: &impl Runtime, params: UpdateStateParams) -> Result<(), ActorError> {
        // Only allow system actor to update state
        // In practice, this will be called by the consensus layer when executing ParentFinality messages
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            // Basic monotonicity checks to prevent accidental rewinds or no-op updates.
            //
            // Note: multiple epochs can be proven under the same certificate instance, so
            // `processed_instance_id` may stay the same across updates, but it must never go
            // backwards.
            //
            // We intentionally allow forward jumps: intermediate F3 instances may be "base-only"
            // (empty suffix), meaning there is no epoch proof/execution point at which to update
            // the actor. In that case, the executor may update the actor directly to a later
            // instance in a single atomic state transition.
            //
            // Also, we allow re-applying the same update (idempotency) by permitting equality.
            if params.processed_instance_id < st.light_client_state.processed_instance_id {
                return Err(actor_error!(
                    illegal_argument,
                    "processed_instance_id went backwards: {} < {}",
                    params.processed_instance_id,
                    st.light_client_state.processed_instance_id
                ));
            }

            st.update_state(rt, params.processed_instance_id, params.power_table)?;
            Ok(())
        })
    }

    fn get_state(rt: &impl Runtime) -> Result<GetStateResponse, ActorError> {
        // Allow any caller to read the state
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        let lc = &state.light_client_state;

        // Materialize the current power table for convenience.
        let power_table = {
            let m = fil_actors_runtime::Map2::<_, u64, PowerEntryValue>::load(
                rt.store(),
                &lc.power_table_root,
                fil_actors_runtime::DEFAULT_HAMT_CONFIG,
                "f3_power_table",
            )?;
            let mut out = Vec::new();
            m.for_each(|id, v| {
                out.push(PowerEntry {
                    id,
                    public_key: v.public_key.clone(),
                    power_be: v.power_be.clone(),
                });
                Ok(())
            })?;
            out.sort_by_key(|e| e.id);
            out
        };

        Ok(GetStateResponse {
            processed_instance_id: lc.processed_instance_id,
            power_table_root: lc.power_table_root,
            power_table,
        })
    }
}

impl ActorCode for F3LightClientActor {
    type Methods = Method;

    fn name() -> &'static str {
        F3_LIGHT_CLIENT_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        UpdateState => update_state,
        GetState => get_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PowerEntry;
    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::error::ExitCode;

    /// Helper function to create test power entries
    fn create_test_power_entries() -> Vec<PowerEntry> {
        fn u64_to_power_be(x: u64) -> Vec<u8> {
            if x == 0 {
                return Vec::new();
            }
            let bytes = x.to_be_bytes();
            let first = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
            bytes[first..].to_vec()
        }

        vec![
            PowerEntry {
                id: 1,
                public_key: vec![1, 2, 3],
                power_be: u64_to_power_be(100),
            },
            PowerEntry {
                id: 2,
                public_key: vec![4, 5, 6],
                power_be: u64_to_power_be(200),
            },
        ]
    }

    /// Construct the actor and verify initialization
    pub fn construct_and_verify(
        current_instance_id: u64,
        power_table: Vec<PowerEntry>,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        // Set caller to system actor (required for constructor)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let constructor_params = ConstructorParams {
            processed_instance_id: current_instance_id,
            power_table,
        };

        let result = rt
            .call::<F3LightClientActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&constructor_params).unwrap(),
            )
            .unwrap();

        expect_empty(result);
        rt.verify();
        rt.reset();

        rt
    }

    #[test]
    fn test_constructor_empty_power_table() {
        let _rt = construct_and_verify(0, vec![]);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_constructor_with_power_table() {
        let power_entries = create_test_power_entries();
        let _rt = construct_and_verify(1, power_entries);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_update_state_success() {
        let rt = construct_and_verify(1, create_test_power_entries());

        // Set caller to system actor
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let update_params = UpdateStateParams {
            processed_instance_id: 1,
            power_table: create_test_power_entries(),
        };

        let result = rt
            .call::<F3LightClientActor>(
                Method::UpdateState as u64,
                IpldBlock::serialize_cbor(&update_params).unwrap(),
            )
            .unwrap();

        expect_empty(result);
        rt.verify();
    }

    #[test]
    fn test_update_state_idempotent_allowed() {
        let rt = construct_and_verify(1, create_test_power_entries());

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_params = UpdateStateParams {
            processed_instance_id: 1,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&initial_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Try to update with same instance
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            processed_instance_id: 1,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        // Allowed (idempotency): equality is ok, only rewinds are rejected.
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_state_unauthorized_caller() {
        let rt = construct_and_verify(1, create_test_power_entries());

        // Set caller to non-system actor
        let unauthorized_caller = Address::new_id(999);
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, unauthorized_caller);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let update_params = UpdateStateParams {
            processed_instance_id: 1,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        // Should fail with forbidden
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_FORBIDDEN);
    }

    #[test]
    fn test_get_state() {
        let power_entries = create_test_power_entries();
        let rt = construct_and_verify(42, power_entries.clone());

        // Update state first
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            processed_instance_id: 42,
            power_table: power_entries.clone(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Query state
        rt.expect_validate_caller_any();
        let result = rt
            .call::<F3LightClientActor>(Method::GetState as u64, None)
            .unwrap()
            .unwrap();

        let response = result.deserialize::<GetStateResponse>().unwrap();
        assert_eq!(response.processed_instance_id, 42);
        assert_eq!(response.power_table, power_entries);
    }

    #[test]
    fn test_power_table_root_changes_on_update() {
        let rt = construct_and_verify(42, create_test_power_entries());

        // Read initial state.
        rt.expect_validate_caller_any();
        let initial = rt
            .call::<F3LightClientActor>(Method::GetState as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<GetStateResponse>()
            .unwrap();

        // Update with a different power table.
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        fn u64_to_power_be(x: u64) -> Vec<u8> {
            if x == 0 {
                return Vec::new();
            }
            let bytes = x.to_be_bytes();
            let first = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
            bytes[first..].to_vec()
        }
        let new_power_table = vec![
            PowerEntry {
                id: 1,
                public_key: vec![1, 2, 3],
                power_be: u64_to_power_be(999),
            },
            PowerEntry {
                id: 3,
                public_key: vec![7, 8, 9],
                power_be: u64_to_power_be(333),
            },
        ];
        let update_params = UpdateStateParams {
            processed_instance_id: 42,
            power_table: new_power_table.clone(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Read updated state.
        rt.expect_validate_caller_any();
        let updated = rt
            .call::<F3LightClientActor>(Method::GetState as u64, None)
            .unwrap()
            .unwrap()
            .deserialize::<GetStateResponse>()
            .unwrap();

        assert_ne!(
            initial.power_table_root, updated.power_table_root,
            "power table root CID should change when table changes"
        );
        assert_eq!(updated.power_table, new_power_table);
    }

    #[test]
    fn test_state_progression() {
        let rt = construct_and_verify(1, create_test_power_entries());

        // Update with first state
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let params1 = UpdateStateParams {
            processed_instance_id: 1,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&params1).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Update with same instance again (idempotent allowed)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let params2 = UpdateStateParams {
            processed_instance_id: 1,
            power_table: create_test_power_entries(),
        };
        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&params2).unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_progression_next_instance() {
        let rt = construct_and_verify(100, create_test_power_entries());

        // First state at instance 100
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_params = UpdateStateParams {
            processed_instance_id: 100,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&initial_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Update to next instance (100 -> 101) should succeed
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            processed_instance_id: 101,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_skip_allowed_but_rewind_rejected() {
        let rt = construct_and_verify(100, create_test_power_entries());

        // First state at instance 100
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_params = UpdateStateParams {
            processed_instance_id: 100,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&initial_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Skipping forward instances is allowed (base-only instances may have no epoch execution point).
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            processed_instance_id: 102,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.reset();

        // Rewinding is still forbidden.
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let rewind_params = UpdateStateParams {
            processed_instance_id: 101,
            power_table: create_test_power_entries(),
        };
        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&rewind_params).unwrap(),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }
}
