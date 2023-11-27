// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::car;
use crate::manifest::{list_manifests, write_manifest, SnapshotItem, SnapshotManifest};
use anyhow::Context;
use async_stm::{atomically, retry, Stm, TVar};
use fendermint_vm_interpreter::fvm::state::snapshot::{BlockHeight, BlockStateParams, Snapshot};
use fendermint_vm_interpreter::fvm::state::FvmStateParams;
use fvm_ipld_blockstore::Blockstore;
use sha2::{Digest, Sha256};
use tendermint_rpc::Client;

/// The file name to export the CAR to.
const SNAPSHOT_FILE_NAME: &str = "snapshot.car";

/// State of snapshots, including the list of available completed ones
/// and the next eligible height.
#[derive(Clone)]
struct SnapshotState {
    /// The latest state parameters at a snapshottable height.
    latest_params: TVar<Option<BlockStateParams>>,
    snapshots: TVar<im::Vector<SnapshotItem>>,
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
    pub fn on_commit(&self, block_height: BlockHeight, params: FvmStateParams) -> Stm<()> {
        if block_height % self.snapshot_interval == 0 {
            self.snapshot_state
                .latest_params
                .write(Some((params, block_height)))?;
        }
        Ok(())
    }

    /// List completed snapshots.
    pub fn list_snapshots(&self) -> Stm<im::Vector<SnapshotItem>> {
        self.snapshot_state.snapshots.read_clone()
    }
}

/// Create snapshots at regular block intervals.
pub struct SnapshotManager<BS> {
    /// Blockstore
    store: BS,
    /// Location to store completed snapshots.
    snapshot_dir: PathBuf,
    /// Target size in bytes for snapshot chunks.
    snapshot_chunk_size: usize,
    /// Number of snapshots to keep.
    ///
    /// 0 means unlimited.
    snapshot_history_size: usize,
    /// Shared state of snapshots.
    snapshot_state: SnapshotState,
    /// How often to check CometBFT whether it has finished syncing.
    sync_poll_interval: Duration,
    /// Indicate whether CometBFT has finished syncing with the chain,
    /// so that we can skip snapshotting old states while catching up.
    is_syncing: TVar<bool>,
}

impl<BS> SnapshotManager<BS>
where
    BS: Blockstore + Clone + Send + Sync + 'static,
{
    /// Create a new manager.
    pub fn new(
        store: BS,
        snapshot_interval: BlockHeight,
        snapshot_dir: PathBuf,
        snapshot_chunk_size: usize,
        snapshot_history_size: usize,
        sync_poll_interval: Duration,
    ) -> anyhow::Result<(Self, SnapshotClient)> {
        let snapshot_items = list_manifests(&snapshot_dir).context("failed to list manifests")?;

        let snapshot_state = SnapshotState {
            // Start with nothing to snapshot until we are notified about a new height.
            // We could also look back to find the latest height we should have snapshotted.
            latest_params: TVar::new(None),
            snapshots: TVar::new(snapshot_items.into()),
        };

        let manager = Self {
            store,
            snapshot_dir,
            snapshot_chunk_size,
            snapshot_history_size,
            snapshot_state: snapshot_state.clone(),
            sync_poll_interval,
            // Assume we are syncing until we can determine otherwise.
            is_syncing: TVar::new(true),
        };

        let client = SnapshotClient {
            snapshot_interval,
            snapshot_state,
        };

        Ok((manager, client))
    }

    /// Produce snapshots.
    pub async fn run<C>(self, client: C)
    where
        C: Client + Send + Sync + 'static,
    {
        // Start a background poll to CometBFT.
        // We could just do this once and await here, but this way ostensibly CometBFT could be
        // restarted without Fendermint and go through another catch up.
        {
            if self.sync_poll_interval.is_zero() {
                atomically(|| self.is_syncing.write(false)).await;
            } else {
                let is_syncing = self.is_syncing.clone();
                let poll_interval = self.sync_poll_interval;
                tokio::spawn(async move {
                    poll_sync_status(client, is_syncing, poll_interval).await;
                });
            }
        }

        let mut last_params = None;
        loop {
            let (state_params, block_height) = atomically(|| {
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

            match self
                .create_snapshot(block_height, state_params.clone())
                .await
            {
                Ok(item) => {
                    tracing::info!(
                        snapshot = item.snapshot_dir.to_string_lossy().to_string(),
                        block_height,
                        chunks_count = item.manifest.chunks,
                        snapshot_size = item.manifest.size,
                        "exported snapshot"
                    );
                    // Add the snapshot to the in-memory records.
                    atomically(|| {
                        self.snapshot_state
                            .snapshots
                            .modify_mut(|items| items.push_back(item.clone()))
                    })
                    .await;
                }
                Err(e) => {
                    tracing::warn!(error =? e, block_height, "failed to create snapshot");
                }
            }

            // Delete old snapshots.
            self.prune_history().await;

            last_params = Some((state_params, block_height));
        }
    }

    /// Remove snapshot directories if we have more than the desired history size.
    async fn prune_history(&self) {
        if self.snapshot_history_size == 0 {
            return;
        }

        let removables = atomically(|| {
            self.snapshot_state.snapshots.modify_mut(|snapshots| {
                let mut removables = Vec::new();
                while snapshots.len() > self.snapshot_history_size {
                    if let Some(snapshot) = snapshots.pop_front() {
                        removables.push(snapshot);
                    } else {
                        break;
                    }
                }
                removables
            })
        })
        .await;

        for r in removables {
            if let Err(e) = std::fs::remove_dir_all(&r.snapshot_dir) {
                tracing::error!(error =? e, snapshot_dir = r.snapshot_dir.to_string_lossy().to_string(), "failed to remove snapsot");
            }
        }
    }

    /// Export a snapshot to a temporary file, then copy it to the snapshot directory.
    async fn create_snapshot(
        &self,
        block_height: BlockHeight,
        state_params: FvmStateParams,
    ) -> anyhow::Result<SnapshotItem> {
        let snapshot = Snapshot::new(self.store.clone(), state_params.clone(), block_height)
            .context("failed to create snapshot")?;

        let snapshot_version = snapshot.version();
        let snapshot_name = format!("snapshot-{block_height}");
        let temp_dir = tempfile::Builder::new()
            .prefix(&snapshot_name)
            .tempdir()
            .context("failed to create temp dir for snapshot")?;

        let snapshot_path = temp_dir.path().join(SNAPSHOT_FILE_NAME);
        let checksum_path = temp_dir.path().join("parts.sha256");
        let parts_path = temp_dir.path().join("parts");

        // TODO: See if we can reuse the contents of an existing CAR file.

        tracing::debug!(
            block_height,
            path = snapshot_path.to_string_lossy().to_string(),
            "exporting snapshot..."
        );

        // Export the state to a CAR file.
        snapshot
            .write_car(&snapshot_path)
            .await
            .context("failed to write CAR file")?;

        let snapshot_size = std::fs::metadata(&snapshot_path)
            .context("failed to get snapshot metadata")?
            .len() as usize;

        // Create a checksum over the CAR file.
        let checksum_bytes = checksum(&snapshot_path).context("failed to compute checksum")?;
        std::fs::write(&checksum_path, checksum_bytes).context("failed to write checksum file")?;

        // Create a directory for the parts.
        std::fs::create_dir(&parts_path).context("failed to create parts dir")?;

        // Split the CAR file into chunks.
        // They can be listed in the right order with e.g. `ls | sort -n`
        // Alternatively we could pad them with zeroes based on the original file size and the chunk size,
        // but this way it will be easier to return them based on a numeric index.
        let chunks_count = car::split(
            &snapshot_path,
            &parts_path,
            self.snapshot_chunk_size,
            |idx| format!("{idx}.part"),
        )
        .await
        .context("failed to split CAR into chunks")?;

        // Create and export a manifest that we can easily look up.
        let manifest = SnapshotManifest {
            block_height,
            size: snapshot_size,
            chunks: chunks_count,
            checksum: checksum_bytes,
            state_params,
            version: snapshot_version,
        };
        let _ = write_manifest(temp_dir.path(), &manifest).context("failed to export manifest")?;

        // Move snapshot to final location - doing it in one step so there's less room for error.
        let snapshot_dir = self.snapshot_dir.join(&snapshot_name);
        std::fs::rename(temp_dir.path(), &snapshot_dir).context("failed to move snapshot")?;

        // Delete the big CAR file - keep the parts only.
        std::fs::remove_file(snapshot_dir.join(SNAPSHOT_FILE_NAME))
            .context("failed to remove CAR file")?;

        Ok(SnapshotItem {
            snapshot_dir,
            manifest,
        })
    }
}

/// Create a Sha256 checksum of a file.
fn checksum(path: impl AsRef<Path>) -> anyhow::Result<tendermint::Hash> {
    let mut file = std::fs::File::open(&path)?;
    let mut hasher = Sha256::new();
    let _ = std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize().into();
    Ok(tendermint::Hash::Sha256(hash))
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

#[cfg(test)]
mod tests {
    use std::{io::Write, sync::Arc, time::Duration};

    use async_stm::{atomically, retry};
    use cid::multihash::MultihashDigest;
    use fendermint_vm_genesis::Genesis;
    use fendermint_vm_interpreter::{
        fvm::{
            bundle::{bundle_path, contracts_path},
            state::{snapshot::Snapshot, FvmGenesisState, FvmStateParams},
            store::memory::MemoryBlockstore,
            FvmMessageInterpreter,
        },
        GenesisInterpreter,
    };
    use fvm::engine::MultiEngine;
    use quickcheck::Arbitrary;
    use tempfile::NamedTempFile;

    use crate::manifest;

    use super::{checksum, SnapshotManager};

    #[test]
    fn file_checksum() {
        let content = b"Hello Checksum!";

        let mut file = NamedTempFile::new().expect("new temp file");
        file.write_all(content).expect("write contents");
        let file_path = file.into_temp_path();
        let file_digest = checksum(file_path).expect("checksum");

        let content_digest = cid::multihash::Code::Sha2_256.digest(content);
        let content_digest = content_digest.digest();

        assert_eq!(file_digest.as_bytes(), content_digest)
    }

    // Initialise genesis and export it directly to see if it works.
    #[tokio::test]
    async fn create_snapshot_directly() {
        let (state_params, store) = init_genesis().await;
        let snapshot = Snapshot::new(store, state_params, 0).expect("failed to create snapshot");
        let tmp_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        snapshot
            .write_car(&tmp_path)
            .await
            .expect("failed to write snapshot");
    }

    // Initialise genesis, create a snapshot manager, export a snapshot, create another manager, list snapshots.
    // Don't forget to run this with `--release` beause of Wasm.
    #[tokio::test]
    async fn create_snapshot_with_manager() {
        let (state_params, store) = init_genesis().await;

        // Now we have one store initialized with genesis, let's create a manager and snapshot it.
        let temp_dir = tempfile::tempdir().expect("failed to create tmp dir");

        // Not polling because it's cumbersome to mock it.
        let never_poll_sync = Duration::ZERO;
        let never_poll_client = mock_client();

        let (snapshot_manager, snapshot_client) = SnapshotManager::new(
            store.clone(),
            1,
            temp_dir.path().into(),
            10000,
            1,
            never_poll_sync,
        )
        .expect("failed to create snapshot manager");

        // Start the manager in the background
        tokio::spawn(async move { snapshot_manager.run(never_poll_client).await });

        // Make sure we have no snapshots currently.
        let snapshots = atomically(|| snapshot_client.list_snapshots()).await;
        assert!(snapshots.is_empty());

        // Notify about snapshottable height.
        atomically(|| snapshot_client.on_commit(0, state_params.clone())).await;

        // Wait for the new snapshot to appear in memory.
        let snapshots = tokio::time::timeout(
            Duration::from_secs(10),
            atomically(|| {
                let snapshots = snapshot_client.list_snapshots()?;
                if snapshots.is_empty() {
                    retry()
                } else {
                    Ok(snapshots)
                }
            }),
        )
        .await
        .expect("failed to export snapshot");

        assert_eq!(snapshots.len(), 1);

        let snapshot = snapshots.into_iter().next().unwrap();
        assert!(snapshot.manifest.chunks > 1);
        assert_eq!(snapshot.manifest.block_height, 0);
        assert_eq!(snapshot.manifest.state_params, state_params);
        assert_eq!(
            snapshot.snapshot_dir.as_path(),
            temp_dir.path().join("snapshot-0")
        );

        let _ = std::fs::File::open(snapshot.snapshot_dir.join("manifest.json"))
            .expect("manifests file exists");

        let snapshots = manifest::list_manifests(temp_dir.path()).unwrap();

        assert_eq!(snapshots.len(), 1, "can list manifests");
        assert_eq!(snapshots[0], snapshot);

        // Create a new manager instance
        let (_, new_client) =
            SnapshotManager::new(store, 1, temp_dir.path().into(), 10000, 1, never_poll_sync)
                .expect("failed to create snapshot manager");

        let snapshots = atomically(|| new_client.list_snapshots()).await;
        assert!(!snapshots.is_empty(), "loads manifests on start");
    }

    async fn init_genesis() -> (FvmStateParams, MemoryBlockstore) {
        let mut g = quickcheck::Gen::new(5);
        let genesis = Genesis::arbitrary(&mut g);

        let bundle = std::fs::read(bundle_path()).expect("failed to read bundle");
        let multi_engine = Arc::new(MultiEngine::default());

        let store = MemoryBlockstore::new();
        let state = FvmGenesisState::new(store.clone(), multi_engine, &bundle)
            .await
            .expect("failed to create state");

        let interpreter =
            FvmMessageInterpreter::new(mock_client(), None, contracts_path(), 1.05, 1.05, false);

        let (state, out) = interpreter
            .init(state, genesis)
            .await
            .expect("failed to init genesis");

        let state_root = state.commit().expect("failed to commit");

        let state_params = FvmStateParams {
            state_root,
            timestamp: out.timestamp,
            network_version: out.network_version,
            base_fee: out.base_fee,
            circ_supply: out.circ_supply,
            chain_id: out.chain_id.into(),
            power_scale: out.power_scale,
        };

        (state_params, store)
    }

    fn mock_client() -> tendermint_rpc::MockClient<tendermint_rpc::MockRequestMethodMatcher> {
        tendermint_rpc::MockClient::new(tendermint_rpc::MockRequestMethodMatcher::default()).0
    }
}
