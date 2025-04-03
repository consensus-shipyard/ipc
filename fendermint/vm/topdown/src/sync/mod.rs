// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A constant running process that fetch or listener to parent state

pub(crate) mod syncer;
pub(crate) mod tendermint;

use crate::proxy::ParentQueryProxy;
use crate::ParentState;
use ethers::utils::hex;
use fvm_shared::chainid::ChainID;
use std::sync::Arc;
use std::time::Duration;

pub use syncer::fetch_topdown_events;

/// Query the fendermint state from the child block chain state.
///
/// It returns `None` from queries until the ledger has been initialized.
pub trait FendermintStateQuery {
    /// Get the latest committed parent state
    fn get_latest_topdown_parent_state(&self) -> anyhow::Result<Option<ParentState>>;

    /// Obtains the chain id from the fendermint state
    fn get_chain_id(&self) -> anyhow::Result<ChainID>;
}

/// Queries the starting parent state synced for polling. First checks the committed parent state, if none, that
/// means the chain has just started, then query from the parent to get the genesis epoch.
pub(crate) async fn query_starting_parent_state<T, P>(
    query: &Arc<T>,
    parent_client: &Arc<P>,
) -> anyhow::Result<ParentState>
where
    T: FendermintStateQuery + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    loop {
        let mut finality = match query.get_latest_topdown_parent_state() {
            Ok(Some(finality)) => finality,
            Ok(None) => {
                tracing::debug!("app not ready for query yet");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            Err(e) => {
                tracing::warn!(error = e.to_string(), "cannot get committed finality");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        tracing::info!(finality = finality.to_string(), "latest finality committed");

        // this means there are no previous committed finality yet, we fetch from parent to get
        // the genesis epoch of the current subnet and its corresponding block hash.
        if finality.height == 0 {
            let genesis_epoch = parent_client.get_genesis_epoch().await?;
            tracing::debug!(genesis_epoch = genesis_epoch, "obtained genesis epoch");
            let r = parent_client.get_block_hash(genesis_epoch).await?;
            tracing::debug!(
                block_hash = hex::encode(&r.block_hash),
                "obtained genesis block hash",
            );

            finality = ParentState {
                height: genesis_epoch,
                block_hash: r.block_hash,
            };
            tracing::info!(
                genesis_finality = finality.to_string(),
                "no previous finality committed, fetched from genesis epoch"
            );
        }

        return Ok(finality);
    }
}
