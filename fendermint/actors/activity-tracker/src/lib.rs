// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::State;
pub use crate::state::ValidatorSummary;

mod state;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(ActivityTrackerActor);

pub const IPC_ACTIVITY_TRACKER_ACTOR_NAME: &str = "activity_tracker";

pub struct ActivityTrackerActor;

#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone)]
pub struct BlockedMinedParams {
    pub validator: Address,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetActivitiesResult {
    pub activities: Vec<ValidatorSummary>,
    pub start_height: ChainEpoch,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetActivitySummaryResult {
    pub commitment: [u8; 32],
    /// Total number validators that have mined blocks
    pub total_active_validators: u64,
    /// The validator details
    pub activities: Vec<ValidatorSummary>,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    BlockMined = frc42_dispatch::method_hash!("BlockMined"),
    GetActivities = frc42_dispatch::method_hash!("GetActivities"),
    PurgeActivities = frc42_dispatch::method_hash!("PurgeActivities"),
    GetSummary = frc42_dispatch::method_hash!("GetSummary"),
}

impl ActivityTrackerActor {
    pub fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        let st = State::new(rt.store())?;
        rt.create(&st)?;

        Ok(())
    }

    pub fn block_mined(rt: &impl Runtime, block: BlockedMinedParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.incr_validator_block_committed(rt, &block.validator)
        })?;

        Ok(())
    }

    pub fn purge_activities(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.purge_validator_block_committed(rt)?;
            st.reset_start_height(rt)
        })?;

        Ok(())
    }

    pub fn get_summary(_rt: &impl Runtime) -> Result<GetActivitySummaryResult, ActorError> {
        // todo
        let dummy = GetActivitySummaryResult{ commitment: [0; 32], total_active_validators: 10, activities: vec![] };
        Ok(dummy)
    }

    pub fn get_activities(rt: &impl Runtime) -> Result<GetActivitiesResult, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state: State = rt.state()?;
        let activities = state.validator_activities(rt)?;
        Ok(GetActivitiesResult {
            activities,
            start_height: state.start_height,
        })
    }
}

impl ActorCode for ActivityTrackerActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_ACTIVITY_TRACKER_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        BlockMined => block_mined,
        GetActivities => get_activities,
        PurgeActivities => purge_activities,
        GetSummary => get_summary,
    }
}
