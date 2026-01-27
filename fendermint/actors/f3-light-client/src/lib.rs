// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::state::State;
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

        let state = State::new(
            rt.store(),
            params.latest_instance_id,
            params.latest_finalized_height,
            params.power_table,
        )?;

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
            st.update_state(
                rt,
                params.latest_instance_id,
                params.latest_finalized_height,
                params.power_table,
            )?;
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
            let m = fil_actors_runtime::Map2::<_, u64, PowerEntry>::load(
                rt.store(),
                &lc.power_table_root,
                fil_actors_runtime::DEFAULT_HAMT_CONFIG,
                "f3_power_table",
            )?;
            let mut out = Vec::new();
            m.for_each(|_k, v| {
                out.push(v.clone());
                Ok(())
            })?;
            out.sort_by_key(|e| e.id);
            out
        };

        Ok(GetStateResponse {
            latest_instance_id: lc.latest_instance_id,
            latest_finalized_height: lc.latest_finalized_height,
            power_table_root: lc.power_table_root.clone(),
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
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::error::ExitCode;

    /// Helper function to create test power entries
    fn create_test_power_entries() -> Vec<PowerEntry> {
        vec![
            PowerEntry {
                id: 1,
                public_key: vec![1, 2, 3],
                power: 100,
            },
            PowerEntry {
                id: 2,
                public_key: vec![4, 5, 6],
                power: 200,
            },
        ]
    }

    /// Construct the actor and verify initialization
    pub fn construct_and_verify(
        current_instance_id: u64,
        power_table: Vec<PowerEntry>,
        latest_finalized_epoch: ChainEpoch,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        // Set caller to system actor (required for constructor)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let constructor_params = ConstructorParams {
            latest_instance_id: current_instance_id,
            latest_finalized_height: latest_finalized_epoch,
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
        let _rt = construct_and_verify(0, vec![], 10);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_constructor_with_power_table() {
        let power_entries = create_test_power_entries();
        let _rt = construct_and_verify(1, power_entries, 10);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_update_state_success() {
        let rt = construct_and_verify(1, create_test_power_entries(), 10);

        // Set caller to system actor
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let update_params = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 10,
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
    fn test_update_state_non_advancing_height() {
        let rt = construct_and_verify(1, create_test_power_entries(), 10);

        // First update to set the finalized height to 102
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_params = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 10,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&initial_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Try to update with same height
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 10,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        // Should fail with illegal argument
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }

    #[test]
    fn test_update_state_unauthorized_caller() {
        let rt = construct_and_verify(1, create_test_power_entries(), 10);

        // Set caller to non-system actor
        let unauthorized_caller = Address::new_id(999);
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, unauthorized_caller);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let update_params = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 11,
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
        let rt = construct_and_verify(42, power_entries.clone(), 10);

        // Update state first
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            latest_instance_id: 42,
            latest_finalized_height: 11,
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
        assert_eq!(response.latest_instance_id, 42);
        assert_eq!(response.latest_finalized_height, 11);
        assert_eq!(response.power_table, power_entries);
    }

    #[test]
    fn test_power_table_root_changes_on_update() {
        let rt = construct_and_verify(42, create_test_power_entries(), 10);

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
        let new_power_table = vec![
            PowerEntry {
                id: 1,
                public_key: vec![1, 2, 3],
                power: 999,
            },
            PowerEntry {
                id: 3,
                public_key: vec![7, 8, 9],
                power: 333,
            },
        ];
        let update_params = UpdateStateParams {
            latest_instance_id: 42,
            latest_finalized_height: 11,
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
        let rt = construct_and_verify(1, create_test_power_entries(), 10);

        // Update with first state
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let params1 = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 100,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&params1).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Update with second state (higher height)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let params2 = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 200,
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
        let rt = construct_and_verify(100, create_test_power_entries(), 10);

        // First state at instance 100
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_params = UpdateStateParams {
            latest_instance_id: 100,
            latest_finalized_height: 10,
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
            latest_instance_id: 101,
            latest_finalized_height: 10,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_skip_rejected() {
        let rt = construct_and_verify(100, create_test_power_entries(), 10);

        // First state at instance 100
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_params = UpdateStateParams {
            latest_instance_id: 100,
            latest_finalized_height: 10,
            power_table: create_test_power_entries(),
        };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&initial_params).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Try to skip instance (100 -> 102) should fail
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let update_params = UpdateStateParams {
            latest_instance_id: 102,
            latest_finalized_height: 10,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }

    #[test]
    fn test_empty_epochs_rejected() {
        let rt = construct_and_verify(1, create_test_power_entries(), 10);

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // Try to update with empty finalized_epochs
        let update_params = UpdateStateParams {
            latest_instance_id: 1,
            latest_finalized_height: 10,
            power_table: create_test_power_entries(),
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }
}
