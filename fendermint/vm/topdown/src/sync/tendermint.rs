// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! The tendermint aware syncer

use crate::sync::syncer::LotusParentSyncer;
use crate::sync::ParentFinalityStateQuery;
use crate::{finality::ParentViewPayload, proxy::ParentQueryProxy};
use anyhow::Context;
use async_stm::auxtx::Aux;
use fendermint_storage::{Codec, Encode, KVStore, KVWritable};

/// Tendermint aware syncer
pub(crate) struct TendermintAwareSyncer<T, C, P, S: KVStore, DB> {
    inner: LotusParentSyncer<T, P, S, DB>,
    tendermint_client: C,
}

impl<T, C, P, S, DB> TendermintAwareSyncer<T, C, P, S, DB>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
    S: KVStore + Encode<u64> + Codec<Option<ParentViewPayload>> + 'static,
    S::Namespace: Send + Sync + 'static,
    DB: KVWritable<S> + Send + Sync + Clone + 'static,
    for<'a> DB::Tx<'a>: Aux,
{
    pub fn new(inner: LotusParentSyncer<T, P, S, DB>, tendermint_client: C) -> Self {
        Self {
            inner,
            tendermint_client,
        }
    }

    /// Sync with the parent, unless CometBFT is still catching up with the network,
    /// in which case we'll get the changes from the subnet peers in the blocks.
    pub async fn sync(&mut self) -> anyhow::Result<()> {
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
