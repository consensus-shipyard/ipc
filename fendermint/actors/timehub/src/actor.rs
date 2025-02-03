// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_blobs_shared::get_credit_approval;
use fendermint_actor_machine::MachineActor;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use tracing::debug;

use crate::{Leaf, Method, PushParams, PushReturn, State, TIMEHUB_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

// Raw type persisted in the store.
// This avoids using CID so that the store does not try to validate or resolve it.
type RawLeaf = (u64, Vec<u8>);

impl Actor {
    fn push(rt: &impl Runtime, params: PushParams) -> Result<PushReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        // Check access control.
        // Either the caller needs to be the Timehub owner, or the owner needs to have given a
        // credit approval to the caller.
        let state = rt.state::<State>()?;
        let owner = state.owner;
        let origin = rt.message().origin();
        let actor_address = state.address.get()?;
        if origin != owner {
            let approved = get_credit_approval(rt, owner, origin)?.is_some();
            if !approved {
                return Err(actor_error!(
                    forbidden;
                    format!("Unauthorized: missing credit approval from Timehub owner {} to origin {} for Timehub {}", owner, origin, actor_address)));
            }
        }

        // Decode the raw bytes as a Cid and report any errors.
        // However, we pass opaque bytes to the store as it tries to validate and resolve any CIDs
        // it stores.
        let _cid = Cid::try_from(params.0.as_slice()).map_err(|_err| {
            actor_error!(illegal_argument;
                    "data must be valid CID bytes")
        })?;
        let timestamp = rt.tipset_timestamp();
        let data: RawLeaf = (timestamp, params.0);

        rt.transaction(|st: &mut State, rt| st.push(rt.store(), data))
    }

    fn get_leaf_at(rt: &impl Runtime, index: u64) -> Result<Option<Leaf>, ActorError> {
        debug!(index, "get_leaf_at");
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        // Decode leaf as timestamp and raw bytes. Then decode as a CID
        let leaf: Option<RawLeaf> = st.get_leaf_at(rt.store(), index)?;
        leaf.map(|(timestamp, bytes)| -> Result<Leaf, ActorError> {
            Ok(Leaf {
                timestamp,
                witnessed: Cid::try_from(bytes).map_err(
                    |_err| actor_error!(illegal_argument; "internal bytes are not a valid CID"),
                )?,
            })
        })
        .transpose()
    }

    fn get_root(rt: &impl Runtime) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_root(rt.store())
    }

    fn get_peaks(rt: &impl Runtime) -> Result<Vec<Cid>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_peaks(rt.store())
    }

    fn get_count(rt: &impl Runtime) -> Result<u64, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        Ok(st.leaf_count)
    }
}

impl MachineActor for Actor {
    type State = State;
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        TIMEHUB_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        Init => init,
        GetAddress => get_address,
        GetMetadata => get_metadata,
        Push => push,
        Get => get_leaf_at,
        Root => get_root,
        Peaks => get_peaks,
        Count => get_count,
        _ => fallback,
    }
}
