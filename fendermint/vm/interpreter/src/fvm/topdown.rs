// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::state::{ipc::GatewayCaller, FvmExecState};
use fvm_ipld_blockstore::Blockstore;

const QUORUM_REACHED: u8 = 1;
const QUORUM_ABANDONED: u8 = 2;

#[derive(Clone)]
pub struct TopdownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
}

impl<DB> TopdownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            gateway_caller: GatewayCaller::default(),
        }
    }

    pub fn execute_topdown_voting_outcome(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<()> {
        let (outcome, vote, tokens_to_mint) = self.gateway_caller.get_voting_outcome(state)?;
        if outcome == QUORUM_REACHED {
            tracing::info!(vote = hex::encode(vote), "quorum reached, execute vote");

            if !tokens_to_mint.is_zero() {
                tracing::info!(
                    token = tokens_to_mint.to_string(),
                    "tokens to mint in child"
                );

                self.gateway_caller
                    .mint_to_gateway(state, tokens_to_mint.clone())?;

                state.update_circ_supply(|circ_supply| {
                    *circ_supply += tokens_to_mint;
                });
            }

            self.gateway_caller.execute(state, vote)?;
        } else if outcome == QUORUM_ABANDONED {
            tracing::warn!("quorum abandoned, restart validator voting");
            self.gateway_caller.clear_votes(state)?;
        }

        Ok(())
    }
}
