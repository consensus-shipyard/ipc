// Copyright 2022-2024 Protocol Labs
// Copyright 2025 Recall Contributors
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

//! Custom Init actor for IPC that allows the ADM actor to spawn any actor type.

use cid::Cid;
use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{
    actor_dispatch_unrestricted, actor_error, extract_send_result, ActorContext, ActorError,
    AsActorError, SYSTEM_ACTOR_ADDR,
};
use fvm_shared::address::Address;
use fvm_shared::{ActorID, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

pub use fil_actors_runtime::INIT_ACTOR_ADDR;

mod state;
mod types;

pub use state::State;
pub use types::*;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(IPCInitActor);

/// ADM Actor ID - hardcoded to match fendermint_vm_actor_interface::adm::ADM_ACTOR_ID
pub const ADM_ACTOR_ID: ActorID = 17;

/// Custom Init actor name for the manifest
pub const IPC_INIT_ACTOR_NAME: &str = "init";

/// Init actor methods
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Exec = 2,
    Exec4 = 3,
}

/// IPC Init actor with ADM support
pub struct IPCInitActor;

impl IPCInitActor {
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(rt.store(), params.network_name)?;
        rt.create(&state)?;
        Ok(())
    }

    pub fn exec(rt: &impl Runtime, params: ExecParams) -> Result<ExecReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller_code = rt
            .get_actor_code_cid(&rt.message().caller().id().unwrap())
            .ok_or_else(|| {
                actor_error!(
                    illegal_state,
                    "no code for caller as {}",
                    rt.message().caller()
                )
            })?;

        if !can_exec(rt, &caller_code, &params.code_cid) {
            return Err(actor_error!(forbidden;
                    "caller type {} cannot exec actor type {}",
                    &caller_code, &params.code_cid
            ));
        }

        let robust_address = rt.new_actor_address()?;

        let (id_address, existing): (ActorID, bool) = rt.transaction(|s: &mut State, rt| {
            s.map_addresses_to_id(rt.store(), &robust_address, None)
                .context("failed to allocate ID address")
        })?;

        if existing {
            return Err(actor_error!(
                forbidden,
                "cannot exec over existing actor {}",
                id_address
            ));
        }

        rt.create_actor(params.code_cid, id_address, None)?;

        extract_send_result(rt.send_simple(
            &Address::new_id(id_address),
            METHOD_CONSTRUCTOR,
            params.constructor_params.into(),
            rt.message().value_received(),
        ))
        .context("constructor failed")?;

        Ok(ExecReturn {
            id_address: Address::new_id(id_address),
            robust_address,
        })
    }

    pub fn exec4(rt: &impl Runtime, params: Exec4Params) -> Result<Exec4Return, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&fil_actors_runtime::EAM_ACTOR_ADDR))?;

        let caller_id = rt.message().caller().id().unwrap();
        let delegated_address = Address::new_delegated(caller_id, &params.subaddress.to_vec())
            .map_err(|e| {
                ActorError::illegal_argument(format!("invalid delegated address: {}", e))
            })?;

        let robust_address = rt.new_actor_address()?;

        let (id_address, existing): (ActorID, bool) = rt.transaction(|s: &mut State, rt| {
            s.map_addresses_to_id(rt.store(), &robust_address, Some(&delegated_address))
                .context("failed to map addresses to ID")
        })?;

        if existing {
            let code_cid = rt.get_actor_code_cid(&id_address).context_code(
                fvm_shared::error::ExitCode::USR_FORBIDDEN,
                "cannot redeploy a deleted actor",
            )?;
            let placeholder_cid = rt.get_code_cid_for_type(Type::Placeholder);
            if code_cid != placeholder_cid {
                return Err(ActorError::forbidden(format!(
                    "cannot replace existing non-placeholder actor with code: {code_cid}"
                )));
            }
        }

        rt.create_actor(params.code_cid, id_address, Some(delegated_address))?;

        extract_send_result(rt.send_simple(
            &Address::new_id(id_address),
            METHOD_CONSTRUCTOR,
            params.constructor_params.into(),
            rt.message().value_received(),
        ))
        .context("constructor failed")?;

        Ok(Exec4Return {
            id_address: Address::new_id(id_address),
            robust_address,
        })
    }
}

impl ActorCode for IPCInitActor {
    type Methods = Method;

    fn name() -> &'static str {
        IPC_INIT_ACTOR_NAME
    }

    actor_dispatch_unrestricted! {
        Constructor => constructor,
        Exec => exec,
        Exec4 => exec4,
    }
}

/// Key modification: Allow ADM actor to exec any actor type
fn can_exec(rt: &impl Runtime, caller: &Cid, exec: &Cid) -> bool {
    let caller_id = rt.message().caller().id();

    // Allow ADM actor (ID 17) to create any actor type
    if caller_id == Ok(ADM_ACTOR_ID) {
        return true;
    }

    // Standard builtin actor checks
    rt.resolve_builtin_actor_type(exec)
        .map(|typ| match typ {
            Type::Multisig | Type::PaymentChannel => true,
            Type::Miner if rt.resolve_builtin_actor_type(caller) == Some(Type::Power) => true,
            _ => false,
        })
        .unwrap_or(false)
}
