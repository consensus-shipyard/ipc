// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::state::ConsensusData;
pub use crate::state::State;
use anyhow::anyhow;
use ethers::abi::AbiEncode;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch, ActorError, EAM_ACTOR_ID};
use fil_actors_runtime::{actor_error, WithCodec, DEFAULT_HAMT_CONFIG};
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::{RawBytes, IPLD_RAW};
use fvm_shared::address::{Address, Payload};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_CONSTRUCTOR;
use ipc_actors_abis::checkpointing_facet::{AggregatedStats, FullActivityRollup, FullSummary};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

mod state;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(ActivityTrackerActor);

pub fn always_fail(_: &mut [u8]) -> Result<(), getrandom::Error> { unimplemented!() }

getrandom::register_custom_getrandom!(always_fail);

pub const IPC_ACTIVITY_TRACKER_ACTOR_NAME: &str = "activity_tracker";

pub struct ActivityTrackerActor;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    RecordBlockCommitted = frc42_dispatch::method_hash!("RecordBlockCommitted"),
    CommitActivity = frc42_dispatch::method_hash!("CommitActivity"),
    PendingActivity = frc42_dispatch::method_hash!("PendingActivity"),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct AbiEncodedBytes<T>
where
    T: ethers::abi::AbiEncode,
{
    pub bytes: Vec<u8>,
    #[serde(skip)]
    pub _marker: PhantomData<T>,
}

impl<T> From<T> for AbiEncodedBytes<T>
where
    T: ethers::abi::AbiEncode,
{
    fn from(value: T) -> Self {
        let encoded = value.encode();
        Self {
            bytes: encoded,
            _marker: PhantomData,
        }
    }
}

trait ActivityTracker {
    /// Hook for the consensus layer to report that the validator committed a new block.
    fn record_block_committed(rt: &impl Runtime, validator: Address) -> Result<(), ActorError>;

    /// Commits the pending activity into an activity rollup.
    /// Currently, this constructs an activity rollup from the internal state, and then resets the internal state.
    /// In the future, this might actually write the activity rollup to the gateway directly, instead of relying on the client to move it around.
    /// Returns the activity rollup as a Solidity ABI-encoded type, in raw byte form.
    fn commit_activity(
        rt: &impl Runtime,
    ) -> Result<
        WithCodec<AbiEncodedBytes<FullActivityRollup>, IPLD_RAW>,
        ActorError,
    >;

    /// Queries the activity that has been accumulated since the last commit, and is pending a flush.
    fn pending_activity(
        rt: &impl Runtime,
    ) -> Result<
        WithCodec<AbiEncodedBytes<FullActivityRollup>, IPLD_RAW>,
        ActorError,
    >;
}

impl ActivityTrackerActor {
    pub fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        let st = State::new(rt.store())?;
        rt.create(&st)?;
        Ok(())
    }
}

impl ActivityTracker for ActivityTrackerActor {
    fn record_block_committed(rt: &impl Runtime, validator: Address) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        // Reject non-f410 addresses.
        if !matches!(validator, Payload::Delegated(d) if d.namespace() == EAM_ACTOR_ID && d.subaddress().len() == 20)
        {
            return Err(
                actor_error!(illegal_argument; "validator address must be a valid f410 address"),
            );
        }

        rt.transaction(|st: &mut State, rt| {
            let mut consensus =
                ConsensusData::load(rt.store(), &st.consensus, DEFAULT_HAMT_CONFIG, "consensus")?;

            let mut v = consensus.get(&validator)?.cloned().unwrap_or_default();
            v.blocks_committed += 1;
            consensus.set(&validator, v)?;

            st.consensus = consensus.flush()?;

            Ok(())
        })
    }

    fn commit_activity(
        rt: &impl Runtime,
    ) -> Result<WithCodec<AbiEncodedBytes<FullActivityRollup>, IPLD_RAW>, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        // Obtain the pending rollup from state.
        let rollup = rt.state::<State>()?.pending_activity_rollup(rt)?;

        rt.transaction(|st: &mut State, rt| {
            st.consensus = ConsensusData::flush_empty(rt.store(), DEFAULT_HAMT_CONFIG)?;
            st.tracking_since = rt.curr_epoch();
            Ok(())
        })?;

        Ok(WithCodec(AbiEncodedBytes::from(rollup)))
    }

    fn pending_activity(
        rt: &impl Runtime,
    ) -> Result<WithCodec<AbiEncodedBytes<FullActivityRollup>, IPLD_RAW>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.state::<State>()?
            .pending_activity_rollup(rt)
            .map(AbiEncodedBytes::from)
            .map(WithCodec)
    }
}

impl ActorCode for ActivityTrackerActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_ACTIVITY_TRACKER_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        RecordBlockCommitted => record_block_committed,
        CommitActivity => commit_activity,
        PendingActivity => pending_activity,
    }
}
