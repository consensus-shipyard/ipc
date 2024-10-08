// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch, ActorError};
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::{ActorID, MethodNum};
use num_derive::FromPrimitive;

pub use crate::state::{ValidatorSummary};
use crate::state::State;

mod state;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(ActivityTrackerActor);

pub const IPC_ACTIVITY_TRACKER_ACTOR_NAME: &str = "activity";

pub struct ActivityTrackerActor;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockedMinedParams {
    validator: Address,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    BlockMined = frc42_dispatch::method_hash!("BlockMined"),
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

    pub fn purge_activities(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.purge_validator_block_committed(rt)?;
            st.reset_start_height(rt)
        })?;

        Ok(())
    }

    pub fn get_activities(rt: &impl Runtime) -> Result<Vec<ValidatorSummary>, ActorError> {
        let state: State = rt.state()?;
        state.validator_activities(rt)
    }

}

impl ActorCode for ActivityTrackerActor {
    type Methods = Method;

    fn name() -> &'static str {
        CHAINMETADATA_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        PushBlockHash => push_block_hash,
        LookbackLen => lookback_len,
        GetBlockHash => get_block_hash,
    }
}
