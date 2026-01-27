// Copyright 2022-2026 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_stm::{atomically, atomically_or_err};
use fendermint_tracing::emit;
use fendermint_vm_event::ParentFinalityMissingQuorum;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::{IpcMessage, ParentFinality};
use fendermint_vm_topdown::proxy::IPCProviderProxyWithLatency;
use fendermint_vm_topdown::voting::ValidatorKey;
use fendermint_vm_topdown::voting::VoteTally;
use fendermint_vm_topdown::{
    BlockHeight, CachedFinalityProvider, IPCParentFinality, ParentFinalityProvider,
    ParentViewProvider, Toggle,
};
use fvm_shared::clock::ChainEpoch;
use std::sync::Arc;

use crate::fvm::end_block_hook::PowerUpdates;

type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>;

/// Legacy (vote-based) parent finality handler.
///
/// Encapsulates all vote/quorum/provider logic; `TopDownManager` should orchestrate execution.
#[derive(Clone)]
pub struct LegacyTopDownHandler {
    provider: TopDownFinalityProvider,
    votes: VoteTally,
}

impl LegacyTopDownHandler {
    pub fn new(provider: TopDownFinalityProvider, votes: VoteTally) -> Self {
        Self { provider, votes }
    }

    pub fn is_enabled(&self) -> bool {
        self.provider.is_enabled()
    }

    pub fn genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
        self.provider.genesis_epoch()
    }

    pub async fn attest(&self, finality: ParentFinality) -> bool {
        let prop = IPCParentFinality {
            height: finality.height as u64,
            block_hash: finality.block_hash,
        };
        atomically(|| self.provider.check_proposal(&prop)).await
    }

    pub async fn update_voting_power_table(&self, power_updates: &PowerUpdates) {
        let power_updates_mapped: Vec<_> = power_updates
            .0
            .iter()
            .map(|v| (ValidatorKey::from(v.public_key.0), v.power.0))
            .collect();

        atomically(|| self.votes.update_power_table(power_updates_mapped.clone())).await
    }

    pub async fn chain_message_for_proposal(&self) -> Option<ChainMessage> {
        tracing::debug!("using legacy voting-based finality");
        self.chain_message_from_finality_or_quorum().await
    }

    pub async fn validator_changes_from(
        &self,
        from: BlockHeight,
        to: BlockHeight,
    ) -> anyhow::Result<Vec<ipc_api::staking::PowerChangeRequest>> {
        self.provider.validator_changes_from(from, to).await
    }

    pub async fn top_down_msgs_from(
        &self,
        from: BlockHeight,
        to: BlockHeight,
    ) -> anyhow::Result<Vec<ipc_api::cross::IpcEnvelope>> {
        self.provider.top_down_msgs_from(from, to).await
    }

    pub async fn on_finality_executed(
        &self,
        finality: IPCParentFinality,
        proposer: Option<&str>,
        local_block_height: u64,
    ) -> anyhow::Result<()> {
        atomically_or_err(|| {
            self.provider.set_new_finality(finality.clone())?;
            self.votes.set_finalized(
                finality.height,
                finality.block_hash.clone(),
                proposer,
                Some(local_block_height),
            )?;
            Ok(())
        })
        .await
    }

    async fn chain_message_from_finality_or_quorum(&self) -> Option<ChainMessage> {
        atomically(|| self.votes.pause_votes_until_find_quorum()).await;

        let (parent, quorum) = atomically(|| {
            let parent = self.provider.next_proposal()?;

            let quorum = self
                .votes
                .find_quorum()?
                .map(|(height, block_hash)| IPCParentFinality { height, block_hash });

            Ok((parent, quorum))
        })
        .await;

        let parent = parent?;

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
}
