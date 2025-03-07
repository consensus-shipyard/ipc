// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::bottomup::PowerUpdates;
use crate::fvm::FvmMessage;
use cid::Cid;
use fendermint_actors_api::gas_market::Reading;
use fendermint_vm_message::query::{ActorState, GasEstimate, StateParams};
use fendermint_vm_message::signed::DomainHash;
use fvm::executor::ApplyRet;
use fvm_shared::{address::Address, error::ExitCode, event::StampedEvent, ActorID, MethodNum};
use std::collections::HashMap;

/// Response for checking a transaction.
/// The check result is expressed by an exit code (and optional info) so that
/// it would result in the same error code if the message were applied.
#[derive(Debug, Clone)]
pub struct CheckResponse {
    pub sender: Address,
    pub gas_limit: u64,
    pub exit_code: ExitCode,
    pub info: Option<String>,
    pub message: FvmMessage,
    pub priority: i64,
}

impl CheckResponse {
    /// Constructs a new check result from a message, an exit code, and optional info.
    pub fn new(
        msg: &FvmMessage,
        exit_code: ExitCode,
        info: Option<String>,
        priority: Option<i64>,
    ) -> Self {
        Self {
            sender: msg.from,
            gas_limit: msg.gas_limit,
            exit_code,
            info,
            message: msg.clone(),
            priority: priority.unwrap_or(0),
        }
    }

    /// Constructs a new check result from a message with OK exit code and no info.
    pub fn new_ok(msg: &FvmMessage, priority: i64) -> Self {
        Self {
            sender: msg.from,
            gas_limit: msg.gas_limit,
            exit_code: ExitCode::OK,
            info: None,
            message: msg.clone(),
            priority,
        }
    }
}

/// Represents the result of applying a message.
#[derive(Debug, Clone)]
pub struct AppliedMessage {
    pub apply_ret: ApplyRet,
    pub from: Address,
    pub to: Address,
    pub method_num: MethodNum,
    pub gas_limit: u64,
    /// Delegated addresses of event emitters, if available.
    pub emitters: Emitters,
}

/// Response from applying a message.
#[derive(Debug, Clone)]
pub struct ApplyMessageResponse {
    pub applied_message: AppliedMessage,
    /// Domain-specific transaction hash for EVM compatibility.
    pub domain_hash: Option<DomainHash>,
}

/// Response from beginning a block.
#[derive(Debug, Clone)]
pub struct BeginBlockResponse {
    pub applied_cron_message: AppliedMessage,
}

/// Response from ending a block.
#[derive(Debug, Clone)]
pub struct EndBlockResponse {
    pub power_updates: PowerUpdates,
    pub gas_market: Reading,
    /// End-block events to be recorded.
    pub events: BlockEndEvents,
}

/// Response for preparing messages for a block.
#[derive(Debug, Clone)]
pub struct PrepareMessagesResponse {
    pub messages: Vec<Vec<u8>>,
    pub total_bytes: usize,
}

/// Decision for attesting a batch of messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestMessagesResponse {
    /// The batch meets the criteria and should be accepted.
    Accept,
    /// The batch does not meet the criteria and should be rejected.
    Reject,
}

/// Query request (similar to what ABCI sends: a path and parameters as bytes).
#[derive(Debug, Clone)]
pub struct Query {
    pub path: String,
    pub params: Vec<u8>,
}

/// Responses to queries.
#[derive(Debug, Clone)]
pub enum QueryResponse {
    /// Bytes from the IPLD store result, if found.
    Ipld(Option<Vec<u8>>),
    /// Full state of an actor, if found.
    ActorState(Option<Box<(ActorID, ActorState)>>),
    /// The result of a read-only message application.
    Call(Box<AppliedMessage>),
    /// Estimated gas limit.
    EstimateGas(GasEstimate),
    /// Current state parameters.
    StateParams(StateParams),
    /// Builtin actors known by the system.
    BuiltinActors(Vec<(String, Cid)>),
}

/// Mapping of actor IDs to addresses (for event emitters).
pub type Emitters = HashMap<ActorID, Address>;

/// A block event, consisting of stamped events and their associated emitters.
pub type Event = (Vec<StampedEvent>, Emitters);

/// A collection of block events.
pub type BlockEndEvents = Vec<Event>;
