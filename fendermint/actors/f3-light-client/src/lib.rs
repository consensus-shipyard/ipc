// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::state::State;
use crate::types::{ConstructorParams, GetStateResponse, UpdateStateParams};
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
            params.instance_id,
            params.power_table,
            params.finalized_epochs,
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
            st.update_state(rt, params.state)?;
            Ok(())
        })
    }

    fn get_state(rt: &impl Runtime) -> Result<GetStateResponse, ActorError> {
        // Allow any caller to read the state
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        let lc = &state.light_client_state;

        Ok(GetStateResponse {
            instance_id: lc.instance_id,
            finalized_epochs: lc.finalized_epochs.clone(),
            power_table: lc.power_table.clone(),
            latest_finalized_height: lc.finalized_epochs.iter().max().copied().unwrap_or(0),
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
    use crate::types::{LightClientState, PowerEntry};
    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::error::ExitCode;

    /// Helper function to create test light client state
    fn create_test_state(
        instance_id: u64,
        finalized_epochs: Vec<i64>,
        power_table: Vec<PowerEntry>,
    ) -> LightClientState {
        LightClientState {
            instance_id,
            finalized_epochs,
            power_table,
        }
    }

    /// Helper function to create test power entries
    fn create_test_power_entries() -> Vec<PowerEntry> {
        vec![
            PowerEntry {
                public_key: vec![1, 2, 3],
                power: 100,
            },
            PowerEntry {
                public_key: vec![4, 5, 6],
                power: 200,
            },
        ]
    }

    /// Construct the actor and verify initialization
    pub fn construct_and_verify(
        instance_id: u64,
        power_table: Vec<PowerEntry>,
        finalized_epochs: Vec<i64>,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        // Set caller to system actor (required for constructor)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let constructor_params = ConstructorParams {
            instance_id,
            power_table,
            finalized_epochs,
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
        let _rt = construct_and_verify(0, vec![], vec![]);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_constructor_with_power_table() {
        let power_entries = create_test_power_entries();
        let _rt = construct_and_verify(1, power_entries, vec![]);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_constructor_with_finalized_epochs() {
        let power_entries = create_test_power_entries();
        let _rt = construct_and_verify(1, power_entries, vec![100, 101, 102]);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_update_state_success() {
        let rt = construct_and_verify(1, create_test_power_entries(), vec![]);

        // Set caller to system actor
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let new_state = create_test_state(1, vec![100, 101, 102], create_test_power_entries());
        let update_params = UpdateStateParams {
            state: new_state.clone(),
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
        let rt = construct_and_verify(1, create_test_power_entries(), vec![]);

        // First update to set the finalized height to 102
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_state = create_test_state(1, vec![100, 101, 102], create_test_power_entries());
        let initial_params = UpdateStateParams {
            state: initial_state,
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
        let same_height_state =
            create_test_state(1, vec![100, 101, 102], create_test_power_entries());
        let update_params = UpdateStateParams {
            state: same_height_state,
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
        let rt = construct_and_verify(1, create_test_power_entries(), vec![]);

        // Set caller to non-system actor
        let unauthorized_caller = Address::new_id(999);
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, unauthorized_caller);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let new_state = create_test_state(1, vec![100, 101, 102], create_test_power_entries());
        let update_params = UpdateStateParams { state: new_state };

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
        let rt = construct_and_verify(42, power_entries.clone(), vec![]);

        // Update state first
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let new_state = create_test_state(42, vec![100, 101, 102], power_entries.clone());
        let update_params = UpdateStateParams { state: new_state };
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
        assert_eq!(response.instance_id, 42);
        assert_eq!(response.finalized_epochs, vec![100, 101, 102]);
        assert_eq!(response.power_table, power_entries);
        assert_eq!(response.latest_finalized_height, 102);
    }

    #[test]
    fn test_state_progression() {
        let rt = construct_and_verify(1, create_test_power_entries(), vec![]);

        // Update with first state
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let state1 = create_test_state(1, vec![100, 101, 102], create_test_power_entries());
        let params1 = UpdateStateParams { state: state1 };
        rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&params1).unwrap(),
        )
        .unwrap();
        rt.reset();

        // Update with second state (higher height)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let state2 = create_test_state(1, vec![200, 201, 202], create_test_power_entries());
        let params2 = UpdateStateParams { state: state2 };
        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&params2).unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_progression_next_instance() {
        let rt = construct_and_verify(100, create_test_power_entries(), vec![]);

        // First state at instance 100
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_state = create_test_state(100, vec![50, 51, 52], create_test_power_entries());
        let initial_params = UpdateStateParams {
            state: initial_state,
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
        let next_instance_state =
            create_test_state(101, vec![10, 11, 12], create_test_power_entries());
        let update_params = UpdateStateParams {
            state: next_instance_state,
        };

        let result = rt.call::<F3LightClientActor>(
            Method::UpdateState as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_skip_rejected() {
        let rt = construct_and_verify(100, create_test_power_entries(), vec![]);

        // First state at instance 100
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let initial_state = create_test_state(100, vec![50, 51, 52], create_test_power_entries());
        let initial_params = UpdateStateParams {
            state: initial_state,
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
        let skipped_state =
            create_test_state(102, vec![100, 101, 102], create_test_power_entries());
        let update_params = UpdateStateParams {
            state: skipped_state,
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
        let rt = construct_and_verify(1, create_test_power_entries(), vec![]);

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // Try to update with empty finalized_epochs
        let invalid_state = create_test_state(1, vec![], create_test_power_entries());
        let update_params = UpdateStateParams {
            state: invalid_state,
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
