// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! The tendermint aware syncer

use crate::finality::ParentViewPayload;
use crate::proxy::ParentQueryProxy;
use crate::sync::syncer::LotusParentSyncer;
use crate::{BlockHeight, ParentState};
use anyhow::Context;

/// Tendermint aware syncer
pub(crate) struct TendermintAwareSyncer<C, P> {
    inner: LotusParentSyncer<P>,
    tendermint_client: C,
}

impl<C, P> TendermintAwareSyncer<C, P>
where
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    pub fn new(inner: LotusParentSyncer<P>, tendermint_client: C) -> Self {
        Self {
            inner,
            tendermint_client,
        }
    }

    pub async fn set_committed(&self, checkpoint: ParentState) {
        self.inner.set_committed(checkpoint).await
    }

    pub async fn get_vote_below_height(&self, height: BlockHeight) -> Option<(BlockHeight, ParentViewPayload)> {
        self.inner.get_vote_below_height(height).await
    }

    pub async fn latest_height(&self) -> BlockHeight {
        self.inner.latest_height().await
    }

    /// Sync with the parent, unless CometBFT is still catching up with the network,
    /// in which case we'll get the changes from the subnet peers in the blocks.
    pub async fn sync(&self) -> anyhow::Result<()> {
        if self.is_syncing_peer().await? {
            tracing::debug!("syncing with peer, skip parent finality syncing this round");
            return Ok(());
        }
        self.inner.sync().await
    }

    async fn is_syncing_peer(&self) -> anyhow::Result<bool> {
        let status: tendermint_rpc::endpoint::status::Response = self
            .tendermint_client
            .status()
            .await
            .context("failed to get Tendermint status")?;
        Ok(status.sync_info.catching_up)
    }
}
