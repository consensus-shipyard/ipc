// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Bridge relay actor implementation.
//!
//! This actor runs on the IPC subnet and:
//! 1. Receives `RelayLockEvent` calls carrying decoded `TokensLocked` events from BridgeLock.sol.
//! 2. Validates the event against configured rules (amount bounds, token allowlist, recipient).
//! 3. Enforces replay protection via a persistent HAMT of processed transfer IDs.
//! 4. On success: emits a relay event (the subnet's cross-message layer picks this up and
//!    delivers the mint instruction to BridgeMint.sol on Ethereum Sepolia).
//! 5. On failure: records the rejection, increments reject_count, returns the reason.
//!    Never silently drops a message.

use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorDowncast;
use fil_actors_runtime::ActorError;
use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;

use crate::{
    ConstructorParams, IsProcessedParams, Method, RelayLockEventParams, RelayLockEventReturn,
    State, StatsReturn, UpdateAddressesParams, UpdateValidationRulesParams,
    ValidationError, BRIDGE_RELAY_ACTOR_NAME,
};

fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    // ─── Constructor ─────────────────────────────────────────────────────────

    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        // Only the system actor may instantiate this actor.
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        if params.bridge_lock_addr == Address::new_id(0) {
            return Err(actor_error!(
                illegal_argument,
                "bridge_lock_addr must not be zero"
            ));
        }
        if params.bridge_mint_addr == Address::new_id(0) {
            return Err(actor_error!(
                illegal_argument,
                "bridge_mint_addr must not be zero"
            ));
        }

        let state = State::new(
            rt.store(),
            params.bridge_lock_addr,
            params.bridge_mint_addr,
            params.validation_rules,
        )
        .map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct bridge-relay state",
            )
        })?;

        rt.create(&state)?;
        log::info!("[bridge-relay] actor constructed");
        Ok(())
    }

    // ─── RelayLockEvent ───────────────────────────────────────────────────────

    /// Process a decoded `TokensLocked` event from BridgeLock.sol.
    ///
    /// The caller is expected to be an authorised relayer (in practice the IPC subnet
    /// infrastructure that observes Filecoin events and calls this method).
    ///
    /// Returns `RelayLockEventReturn` with `success: true` on successful relay, or
    /// `success: false` with a `rejection_reason` on any validation/replay failure.
    /// Never reverts on business-logic failures — only reverts on system errors.
    fn relay_lock_event(
        rt: &impl Runtime,
        params: RelayLockEventParams,
    ) -> Result<RelayLockEventReturn, ActorError> {
        // Any caller may submit (the subnet infrastructure drives this).
        rt.validate_immediate_caller_accept_any()?;

        let event = &params.event;
        let transfer_id = event.transfer_id;

        // ── Validate ──────────────────────────────────────────────────────────
        let validation_error: Option<ValidationError> = rt.transaction(|st: &mut State, rt| {
            // 1. Replay protection check
            let already_processed = st
                .is_processed(rt.store(), &transfer_id)
                .map_err(|e| {
                    e.downcast_default(
                        ExitCode::USR_ILLEGAL_STATE,
                        "replay check failed",
                    )
                })?;
            if already_processed {
                st.reject_count += 1;
                return Ok(Some(ValidationError::DuplicateTransfer { transfer_id }));
            }

            // 2. Validation rules
            if let Err(err) = st.validation_rules.validate(event) {
                log::warn!(
                    "[bridge-relay] transfer {:?} rejected: {}",
                    hex::encode(transfer_id),
                    err
                );
                st.reject_count += 1;
                return Ok(Some(err));
            }

            // ── Mark as processed and emit relay ──────────────────────────────
            let epoch = rt.curr_epoch() as u64;
            st.mark_processed(rt.store(), &transfer_id, epoch)
                .map_err(|e| {
                    e.downcast_default(
                        ExitCode::USR_ILLEGAL_STATE,
                        "failed to mark transfer as processed",
                    )
                })?;
            st.relay_count += 1;

            log::info!(
                "[bridge-relay] relaying transfer {:?}: token={:?} recipient={:?} amount={}",
                hex::encode(transfer_id),
                event.token,
                event.recipient,
                event.amount,
            );

            Ok(None)
        })?;

        if let Some(err) = validation_error {
            // Emit rejection event so off-chain monitors can observe it.
            rt.emit_event(
                &fil_actors_runtime::EventBuilder::new()
                    .field_indexed("type", &"bridge-relay/rejected")
                    .field_indexed("transfer_id", &transfer_id.as_ref())
                    .field("reason", &err.to_string())
                    .build()
                    .map_err(|e| {
                        actor_error!(illegal_state, "failed to build rejection event: {e}")
                    })?,
            )
            .map_err(|e| actor_error!(illegal_state, "failed to emit rejection event: {e}"))?;

            return Ok(RelayLockEventReturn {
                success: false,
                rejection_reason: Some(err.to_string()),
            });
        }

        // ── Emit relay event ──────────────────────────────────────────────────
        // The IPC subnet infrastructure observes this event and triggers the
        // cross-chain message to BridgeMint.sol on Ethereum Sepolia.
        rt.emit_event(
            &fil_actors_runtime::EventBuilder::new()
                .field_indexed("type", &"bridge-relay/relayed")
                .field_indexed("transfer_id", &transfer_id.as_ref())
                .field("token", &event.token.to_bytes())
                .field("recipient", &event.recipient.to_bytes())
                .field("amount", &event.amount.atto().to_bytes_be())
                .build()
                .map_err(|e| {
                    actor_error!(illegal_state, "failed to build relay event: {e}")
                })?,
        )
        .map_err(|e| actor_error!(illegal_state, "failed to emit relay event: {e}"))?;

        Ok(RelayLockEventReturn {
            success: true,
            rejection_reason: None,
        })
    }

    // ─── UpdateValidationRules ────────────────────────────────────────────────

    /// Update the validation rules. Admin (SYSTEM_ACTOR) only.
    fn update_validation_rules(
        rt: &impl Runtime,
        params: UpdateValidationRulesParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.transaction(|st: &mut State, _| {
            st.validation_rules = params.rules;
            Ok(())
        })?;
        log::info!("[bridge-relay] validation rules updated");
        Ok(())
    }

    // ─── UpdateAddresses ──────────────────────────────────────────────────────

    /// Update BridgeLock / BridgeMint addresses. Admin (SYSTEM_ACTOR) only.
    fn update_addresses(
        rt: &impl Runtime,
        params: UpdateAddressesParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.transaction(|st: &mut State, _| {
            st.bridge_lock_addr = params.bridge_lock_addr;
            st.bridge_mint_addr = params.bridge_mint_addr;
            Ok(())
        })?;
        log::info!("[bridge-relay] addresses updated");
        Ok(())
    }

    // ─── GetStats ─────────────────────────────────────────────────────────────

    fn get_stats(rt: &impl Runtime) -> Result<StatsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        Ok(StatsReturn {
            relay_count: st.relay_count,
            reject_count: st.reject_count,
            bridge_lock_addr: st.bridge_lock_addr,
            bridge_mint_addr: st.bridge_mint_addr,
        })
    }

    // ─── IsProcessed ──────────────────────────────────────────────────────────

    fn is_processed(
        rt: &impl Runtime,
        params: IsProcessedParams,
    ) -> Result<bool, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.is_processed(rt.store(), &params.transfer_id)
            .map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "is_processed check failed")
            })
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        BRIDGE_RELAY_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        RelayLockEvent => relay_lock_event,
        UpdateValidationRules => update_validation_rules,
        UpdateAddresses => update_addresses,
        GetStats => get_stats,
        IsProcessed => is_processed,
    }
}
