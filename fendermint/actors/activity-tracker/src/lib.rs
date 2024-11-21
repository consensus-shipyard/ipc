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
pub use crate::state::{ValidatorData, ValidatorStats};

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
    pub activities: Vec<ValidatorData>,
    pub cycle_start: ChainEpoch,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    // REVIEW(raulk): Rename to "ReportBlockCommitted" (methods should always be actions, and we use Committed, not Mined).
    BlockMined = frc42_dispatch::method_hash!("BlockMined"),
    // REVIEW(raulk): Merge these two methods into a "PullActivity" method that returns the full activity summary _and_ resets the internal state atomically.
    GetActivities = frc42_dispatch::method_hash!("GetActivities"),
    PurgeActivities = frc42_dispatch::method_hash!("PurgeActivities"),
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

    /// Purges all activities to start a new tracking cycle.
    pub fn purge_activities(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.purge_validator_block_committed(rt)?;
            st.reset_cycle_height(rt)
        })?;

        Ok(())
    }

    pub fn get_activities(rt: &impl Runtime) -> Result<GetActivitiesResult, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state: State = rt.state()?;
        let activities = state.validator_activities(rt)?;
        Ok(GetActivitiesResult {
            activities,
            cycle_start: state.cycle_start,
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
    }
}
