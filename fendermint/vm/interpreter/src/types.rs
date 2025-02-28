use crate::fvm::FvmMessage;
use cid::Cid;
use fendermint_vm_message::signed::DomainHash;
use fvm::executor::ApplyRet;
use fvm_shared::{address::Address, error::ExitCode};
use fvm_shared::{ActorID, MethodNum, BLOCK_GAS_LIMIT};

use fendermint_actors_api::gas_market::Reading;
use fendermint_vm_message::query::{ActorState, GasEstimate, StateParams};
use fvm_shared::event::StampedEvent;
use std::collections::HashMap;

use crate::fvm::checkpoint::PowerUpdates;

/// Transaction check results are expressed by the exit code, so that they would
/// result in the same error code if they were applied.
pub struct CheckResponse {
    pub sender: Address,
    pub gas_limit: u64,
    pub exit_code: ExitCode,
    pub info: Option<String>,
    pub message: FvmMessage,
}

impl CheckResponse {
    /// Constructs a new check result from a message, an exit code, and optional info.
    pub fn new(msg: &FvmMessage, exit_code: ExitCode, info: Option<String>) -> Self {
        Self {
            sender: msg.from,
            gas_limit: msg.gas_limit,
            exit_code,
            info,
            message: msg.clone(),
        }
    }

    /// Constructs a new check result from a message with OK exit code and no info.
    pub fn new_ok(msg: &FvmMessage) -> Self {
        Self {
            sender: msg.from,
            gas_limit: msg.gas_limit,
            exit_code: ExitCode::OK,
            info: None,
            message: msg.clone(),
        }
    }
}

pub type Emitters = HashMap<ActorID, Address>;

pub type Event = (Vec<StampedEvent>, Emitters);
pub type BlockEndEvents = Vec<Event>;

pub struct ApplyMessageResponse {
    pub applied_message: AppliedMessage,
    pub domain_hash: Option<DomainHash>,
}

// TODO Karel - move this somewhere else? Since it is generic return type and not tight to apply_messages
/// The return value extended with some things from the message that
/// might not be available to the caller, because of the message lookups
/// and transformations that happen along the way, e.g. where we need
/// a field, we might just have a CID.
pub struct AppliedMessage {
    pub apply_ret: ApplyRet,
    pub from: Address,
    pub to: Address,
    pub method_num: MethodNum,
    pub gas_limit: u64,
    /// Delegated addresses of event emitters, if they have one.
    pub emitters: Emitters,
}

pub struct EndBlockResponse {
    pub power_updates: PowerUpdates,
    pub gas_market: Reading,
    /// The end block events to be recorded
    pub events: BlockEndEvents,
}

/// Close to what the ABCI sends: (Path, Bytes).
pub struct Query {
    pub path: String,
    pub params: Vec<u8>,
}

pub enum QueryResponse {
    /// Bytes from the IPLD store result, if found.
    Ipld(Option<Vec<u8>>),
    /// The full state of an actor, if found.
    ActorState(Option<Box<(ActorID, ActorState)>>),
    /// The results of a read-only message application.
    Call(Box<AppliedMessage>),
    /// The estimated gas limit.
    EstimateGas(GasEstimate),
    /// Current state parameters.
    StateParams(StateParams),
    /// Builtin actors known by the system.
    BuiltinActors(Vec<(String, Cid)>),
}

/// Decision to accept or reject a batch of messages for process method.
pub enum ProcessDecision {
    /// The batch of messages meets the criteria and should be included in the block.
    Accept,
    /// The batch of messages does not meet the criteria and should be rejected.
    Reject,
}

// TODO Karel - handle this type in the check function instead
// pub enum CheckDecision {
//     Accept(FvmCheckRet),
//     Reject,
// }
