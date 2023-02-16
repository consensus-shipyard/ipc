// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_trait::async_trait;

use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, error::ExitCode};

use crate::CheckInterpreter;

use super::{FvmCheckState, FvmMessage, FvmMessageInterpreter};

/// Transaction check results are expressed by the exit code, so that hopefully
/// they would result in the same error code if they were applied.
pub struct FvmCheckRet {
    pub sender: Address,
    pub gas_limit: u64,
    pub exit_code: ExitCode,
}

#[async_trait]
impl<DB> CheckInterpreter for FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync,
{
    type State = FvmCheckState<DB>;
    type Message = FvmMessage;
    type Output = FvmCheckRet;

    /// Check that:
    /// * sender exists
    /// * sender nonce matches the message sequence
    /// * sender has enough funds to cover the gas cost
    async fn check(
        &self,
        mut state: Self::State,
        msg: Self::Message,
        _is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        let checked = |state, exit_code| {
            let ret = FvmCheckRet {
                sender: msg.from,
                gas_limit: msg.gas_limit,
                exit_code,
            };
            Ok((state, ret))
        };

        // NOTE: This would be a great place for let-else, but clippy runs into a compilation bug.
        if let Some(id) = state.state_tree.lookup_id(&msg.from)? {
            if let Some(mut actor) = state.state_tree.get_actor(id)? {
                let balance_needed = msg.gas_fee_cap * msg.gas_limit;
                if actor.balance < balance_needed || actor.sequence != msg.sequence {
                    return checked(state, ExitCode::SYS_SENDER_STATE_INVALID);
                } else {
                    actor.sequence += 1;
                    actor.balance -= balance_needed;
                    state.state_tree.set_actor(id, actor);
                    return checked(state, ExitCode::OK);
                }
            }
        }
        return checked(state, ExitCode::SYS_SENDER_INVALID);
    }
}
