use crate::fvm::state::FvmExecState;
use crate::fvm::store::ReadOnlyBlockstore;
use crate::fvm::FvmMessage;
use anyhow::Ok;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, error::ExitCode};
use std::sync::Arc;

use crate::types::*;

/// Checks the actor state (balance and sequence) for the sender of message.
pub fn check_nonce_and_sufficient_balance(
    state: &FvmExecState<ReadOnlyBlockstore<Arc<impl Blockstore + Clone + 'static>>>,
    msg: &FvmMessage,
) -> anyhow::Result<FvmCheckRet> {
    // Look up the actor associated with the sender's address.
    let actor = match lookup_actor(&state, &msg.from)? {
        Some(actor) => actor,
        None => {
            return Ok(FvmCheckRet::new(
                msg,
                ExitCode::SYS_SENDER_STATE_INVALID,
                None,
            ))
        }
    };

    // Calculate the required balance.
    let balance_needed = msg.gas_fee_cap.clone() * msg.gas_limit;

    // Check for sufficient balance.
    if actor.balance < balance_needed {
        return Ok(FvmCheckRet::new(
            msg,
            ExitCode::SYS_SENDER_STATE_INVALID,
            Some(format!(
                "actor balance {} less than needed {}",
                actor.balance, balance_needed
            )),
        ));
    }

    // Check for a nonce match.
    if actor.sequence != msg.sequence {
        return Ok(FvmCheckRet::new(
            msg,
            ExitCode::SYS_SENDER_STATE_INVALID,
            Some(format!(
                "expected sequence {}, got {}",
                actor.sequence, msg.sequence
            )),
        ));
    }

    Ok(FvmCheckRet::new(msg, ExitCode::OK, None))
}

/// Looks up an actor by address in the state tree.
fn lookup_actor(
    state: &FvmExecState<ReadOnlyBlockstore<Arc<impl Blockstore + Clone + 'static>>>,
    address: &Address,
) -> anyhow::Result<Option<ActorState>> {
    let state_tree = state.state_tree();

    let id = match state_tree.lookup_id(address)? {
        Some(id) => id,
        None => return Ok(None),
    };

    let actor = match state_tree.get_actor(id)? {
        Some(actor) => actor,
        None => return Ok(None),
    };

    Ok(Some(actor))
}
