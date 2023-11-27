// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use async_stm::{atomically, retry, TVar};
use fendermint_vm_interpreter::fvm::state::snapshot::{BlockHeight, BlockStateParams, Snapshot};
use fendermint_vm_interpreter::fvm::state::FvmStateParams;
use fvm_ipld_blockstore::Blockstore;
use tendermint_rpc::Client;

/// State of snapshots, including the list of available completed ones
/// and the next eligible height.
#[derive(Clone)]
struct SnapshotState {
    /// The latest state parameters at a snapshottable height.
    latest_params: TVar<Option<BlockStateParams>>,
}

/// Interface to snapshot state for the application.
#[derive(Clone)]
pub struct SnapshotClient {
    /// The client will only notify the manager of snapshottable heights.
    snapshot_interval: BlockHeight,
    snapshot_state: SnapshotState,
}

impl SnapshotClient {
    /// Set the latest block state parameters and notify the manager.
    pub async fn on_commit(&self, block_height: BlockHeight, params: FvmStateParams) {
        if block_height % self.snapshot_interval == 0 {
            atomically(|| {
                self.snapshot_state
                    .latest_params
                    .write(Some((params.clone(), block_height)))
            })
            .await;
        }
    }
}

/// Create snapshots at regular block intervals.
pub struct SnapshotManager<BS, C> {
    /// Blockstore
    store: BS,
    /// CometBFT client.
    client: C,
    /// Location to store completed snapshots.
    snapshot_dir: PathBuf,
    /// Shared state of snapshots.
    snapshot_state: SnapshotState,
    /// How often to check CometBFT whether it has finished syncing.
    sync_poll_interval: Duration,
    /// Indicate whether CometBFT has finished syncing with the chain,
    /// so that we can skip snapshotting old states while catching up.
    is_syncing: TVar<bool>,
}

impl<BS, C> SnapshotManager<BS, C>
where
    BS: Blockstore + Clone + Send + Sync + 'static,
    C: Client + Clone + Send + Sync + 'static,
{
    /// Create a new manager.
    pub fn new(
        store: BS,
        client: C,
        snapshot_interval: BlockHeight,
        snapshot_dir: PathBuf,
        sync_poll_interval: Duration,
    ) -> (Self, SnapshotClient) {
        let manager = Self {
            client,
            store,
            snapshot_dir,
            snapshot_state: SnapshotState {
                // Start with nothing to snapshot until we are notified about a new height.
                // We could also look back to find the latest height we should have snapshotted.
                latest_params: TVar::new(None),
            },
            sync_poll_interval,
            // Assume we are syncing until we can determine otherwise.
            is_syncing: TVar::new(true),
        };
        let client = SnapshotClient {
            snapshot_interval,
            snapshot_state: manager.snapshot_state.clone(),
        };
        (manager, client)
    }

    /// Produce snapshots.
    pub async fn run(self) {
        // Start a background poll to CometBFT.
        // We could just do this once and await here, but this way ostensibly CometBFT could be
        // restarted without Fendermint and go through another catch up.
        {
            let client = self.client.clone();
            let is_syncing = self.is_syncing.clone();
            let poll_interval = self.sync_poll_interval;
            tokio::spawn(async move {
                poll_sync_status(client, is_syncing, poll_interval).await;
            });
        }

        let mut last_params = None;
        loop {
            let (params, height) = atomically(|| {
                // Check the current sync status. We could just query the API, but then we wouldn't
                // be notified when we finally reach the end, and we'd only snapshot the next height,
                // not the last one as soon as the chain is caught up.
                if *self.is_syncing.read()? {
                    retry()?;
                }

                match self.snapshot_state.latest_params.read()?.as_ref() {
                    None => retry()?,
                    unchanged if *unchanged == last_params => retry()?,
                    Some(new_params) => Ok(new_params.clone()),
                }
            })
            .await;

            if let Err(e) = self.create_snapshot(height, params.clone()).await {
                tracing::warn!(error =? e, height, "failed to create snapshot");
            }

            last_params = Some((params, height));
        }
    }

    /// Export a snapshot to a temporary file, then copy it to the snapshot directory.
    async fn create_snapshot(
        &self,
        height: BlockHeight,
        params: FvmStateParams,
    ) -> anyhow::Result<()> {
        let snapshot = Snapshot::new(self.store.clone(), params, height)
            .context("failed to create snapshot")?;

        let file_name = format!("snapshot-{height}.car");
        let file_path = self.snapshot_dir.join(file_name.clone());
        let temp_dir = tempfile::tempdir().context("failed to create temp dir for snapshot")?;
        let temp_path = temp_dir.path().join(file_name);

        // TODO: See if we can reuse the contents of an existing CAR file.

        tracing::debug!(
            height,
            path = temp_path.to_string_lossy().to_string(),
            "exporting snapshot..."
        );

        snapshot
            .write_car(temp_path.clone())
            .await
            .context("failed to write CAR file")?;

        std::fs::rename(temp_path, file_path.clone()).context("failed to move snapshot file")?;

        tracing::info!(
            height,
            path = file_path.to_string_lossy().to_string(),
            "exported snapshot"
        );

        Ok(())
    }
}

/// Periodically ask CometBFT if it has caught up with the chain.
async fn poll_sync_status<C>(client: C, is_syncing: TVar<bool>, poll_interval: Duration)
where
    C: Client + Send + Sync + 'static,
{
    loop {
        match client.status().await {
            Ok(status) => {
                let catching_up = status.sync_info.catching_up;

                atomically(|| {
                    if *is_syncing.read()? != catching_up {
                        is_syncing.write(catching_up)?;
                    }
                    Ok(())
                })
                .await;
            }
            Err(e) => {
                tracing::warn!(error =? e, "failed to poll CometBFT sync status");
            }
        }
        tokio::time::sleep(poll_interval).await;
    }
}
