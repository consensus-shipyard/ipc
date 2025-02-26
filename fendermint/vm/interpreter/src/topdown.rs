use crate::fvm::PowerUpdates;
use async_stm::atomically;
use fendermint_tracing::emit;
use fendermint_vm_event::ParentFinalityMissingQuorum;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::IpcMessage;
use fendermint_vm_message::ipc::ParentFinality;
use fendermint_vm_topdown::proxy::IPCProviderProxyWithLatency;
use fendermint_vm_topdown::voting::ValidatorKey;
use fendermint_vm_topdown::voting::VoteTally;
use fendermint_vm_topdown::{
    CachedFinalityProvider, IPCParentFinality, ParentFinalityProvider, ParentViewProvider, Toggle,
};
use fvm_shared::clock::ChainEpoch;
use std::sync::Arc;

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::topdown::{commit_finality, execute_topdown_msgs};
use anyhow::{bail, Context};
use fvm_ipld_blockstore::Blockstore;

use crate::types::ApplyResponse;

type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>;

pub struct TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    provider: TopDownFinalityProvider,
    votes: VoteTally,
    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
}

impl<DB> TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    pub async fn is_finality_valid(&self, finality: ParentFinality) -> bool {
        let prop = IPCParentFinality {
            height: finality.height as u64,
            block_hash: finality.block_hash,
        };
        atomically(|| self.provider.check_proposal(&prop)).await
    }

    /// Prepares a top-down execution message based on the current parent's finality proposal and quorum.
    ///
    /// This function first pauses incoming votes to prevent interference during processing. It then atomically retrieves
    /// both the next parent's proposal and the quorum of votes. If either the parent's proposal or the quorum is missing,
    /// the function returns `None`. When both are available, it selects the finality with the lower block height and wraps
    /// it into a `ChainMessage` for top-down execution.
    pub async fn message_from_finality_or_quorum(&self) -> Option<ChainMessage> {
        // Prepare top down proposals.
        // Before we try to find a quorum, pause incoming votes. This is optional but if there are lots of votes coming in it might hold up proposals.
        atomically(|| self.votes.pause_votes_until_find_quorum()).await;

        // The pre-requisite for proposal is that there is a quorum of gossiped votes at that height.
        // The final proposal can be at most as high as the quorum, but can be less if we have already,
        // hit some limits such as how many blocks we can propose in a single step.
        let (parent, quorum) = atomically(|| {
            let parent = self.provider.next_proposal()?;

            let quorum = self
                .votes
                .find_quorum()?
                .map(|(height, block_hash)| IPCParentFinality { height, block_hash });

            Ok((parent, quorum))
        })
        .await;

        // If there is no parent proposal, exit early.
        let parent = if let Some(parent) = parent {
            parent
        } else {
            return None;
        };

        // Require a quorum; if it's missing, log and exit.
        let quorum = if let Some(quorum) = quorum {
            quorum
        } else {
            emit!(
                DEBUG,
                ParentFinalityMissingQuorum {
                    block_height: parent.height,
                    block_hash: &hex::encode(&parent.block_hash),
                }
            );
            return None;
        };

        // Choose the lower height between the parent's proposal and the quorum.
        let finality = if parent.height <= quorum.height {
            parent
        } else {
            quorum
        };

        Some(ChainMessage::Ipc(IpcMessage::TopDownExec(ParentFinality {
            height: finality.height as ChainEpoch,
            block_hash: finality.block_hash,
        })))
    }

    pub async fn update_voting_power_table(&self, power_updates: &PowerUpdates) {
        let power_updates_mapped: Vec<_> = power_updates
            .0
            .iter()
            .map(|v| (ValidatorKey::from(v.public_key.0), v.power.0))
            .collect();

        atomically(|| self.votes.update_power_table(power_updates_mapped.clone())).await
    }

    // TODO Karel - separate this huge function and clean up
    pub async fn execute_topdown_msg(
        &self,
        mut state: &mut FvmExecState<DB>,
        finality: ParentFinality,
    ) -> anyhow::Result<ApplyResponse> {
        if !self.provider.is_enabled() {
            bail!("cannot execute IPC top-down message: parent provider disabled");
        }

        // commit parent finality first
        let finality = IPCParentFinality::new(finality.height, finality.block_hash);
        tracing::debug!(
            finality = finality.to_string(),
            "chain interpreter received topdown exec proposal",
        );

        let (prev_height, prev_finality) = commit_finality(
            &self.gateway_caller,
            &mut state,
            finality.clone(),
            &self.provider,
        )
        .await
        .context("failed to commit finality")?;

        tracing::debug!(
            previous_committed_height = prev_height,
            previous_committed_finality = prev_finality
                .as_ref()
                .map(|f| format!("{f}"))
                .unwrap_or_else(|| String::from("None")),
            "chain interpreter committed topdown finality",
        );

        // The height range we pull top-down effects from. This _includes_ the proposed
        // finality, as we assume that the interface we query publishes only fully
        // executed blocks as the head of the chain. This is certainly the case for
        // Ethereum-compatible JSON-RPC APIs, like Filecoin's. It should be the case
        // too for future Filecoin light clients.
        //
        // Another factor to take into account is the chain_head_delay, which must be
        // non-zero. So even in the case where deferred execution leaks through our
        // query mechanism, it should not be problematic because we're guaranteed to
        // be _at least_ 1 height behind.
        let (execution_fr, execution_to) = (prev_height + 1, finality.height);

        // error happens if we cannot get the validator set from ipc agent after retries
        let validator_changes = self
            .provider
            .validator_changes_from(execution_fr, execution_to)
            .await
            .context("failed to fetch validator changes")?;

        tracing::debug!(
            from = execution_fr,
            to = execution_to,
            msgs = validator_changes.len(),
            "chain interpreter received total validator changes"
        );

        self.gateway_caller
            .store_validator_changes(&mut state, validator_changes)
            .context("failed to store validator changes")?;

        // error happens if we cannot get the cross messages from ipc agent after retries
        let msgs = self
            .provider
            .top_down_msgs_from(execution_fr, execution_to)
            .await
            .context("failed to fetch top down messages")?;

        tracing::debug!(
            number_of_messages = msgs.len(),
            start = execution_fr,
            end = execution_to,
            "chain interpreter received topdown msgs",
        );

        let ret = execute_topdown_msgs(&self.gateway_caller, &mut state, msgs)
            .await
            .context("failed to execute top down messages")?;

        tracing::debug!("chain interpreter applied topdown msgs");

        let local_block_height = state.block_height() as u64;
        let proposer = state
            .block_producer()
            .map(|id| hex::encode(id.serialize_compressed()));
        let proposer_ref = proposer.as_deref();

        atomically(|| {
            self.provider
                .set_new_finality(finality.clone(), prev_finality.clone())?;

            self.votes.set_finalized(
                finality.height,
                finality.block_hash.clone(),
                proposer_ref,
                Some(local_block_height),
            )?;

            Ok(())
        })
        .await;

        tracing::debug!(
            finality = finality.to_string(),
            "chain interpreter has set new"
        );

        // TODO Karel - the execute function should return apply response instead
        Ok(ApplyResponse {
            apply_ret: ret.apply_ret,
            from: ret.from,
            to: ret.to,
            method_num: ret.method_num,
            gas_limit: ret.gas_limit,
            emitters: ret.emitters,
        })
    }
}
