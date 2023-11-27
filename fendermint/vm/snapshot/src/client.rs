// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{sync::Arc, time::SystemTime};

use async_stm::{abort, Stm, StmResult, TVar};
use fendermint_vm_interpreter::fvm::state::{
    snapshot::{BlockHeight, SnapshotVersion},
    FvmStateParams,
};
use tempfile::tempdir;

use crate::{
    manifest,
    state::{SnapshotDownload, SnapshotState},
    SnapshotError, SnapshotItem, SnapshotManifest,
};

/// Interface to snapshot state for the application.
#[derive(Clone)]
pub struct SnapshotClient {
    /// The client will only notify the manager of snapshottable heights.
    snapshot_interval: BlockHeight,
    state: SnapshotState,
}

impl SnapshotClient {
    pub fn new(snapshot_interval: BlockHeight, state: SnapshotState) -> Self {
        Self {
            snapshot_interval,
            state,
        }
    }
    /// Set the latest block state parameters and notify the manager.
    ///
    /// Call this with the block height where the `app_hash` in the block reflects the
    /// state in the parameters, that is, the in the *next* block.
    pub fn notify(&self, block_height: BlockHeight, state_params: FvmStateParams) -> Stm<()> {
        if block_height % self.snapshot_interval == 0 {
            self.state
                .latest_params
                .write(Some((state_params, block_height)))?;
        }
        Ok(())
    }

    /// List completed snapshots.
    pub fn list_snapshots(&self) -> Stm<im::Vector<SnapshotItem>> {
        self.state.snapshots.read_clone()
    }

    /// Try to find a snapshot, if it still exists.
    ///
    /// If found, mark it as accessed, so that it doesn't get purged while likely to be requested or read from disk.
    pub fn access_snapshot(
        &self,
        block_height: BlockHeight,
        version: SnapshotVersion,
    ) -> Stm<Option<SnapshotItem>> {
        let mut snapshots = self.state.snapshots.read_clone()?;
        let mut snapshot = None;
        for s in snapshots.iter_mut() {
            if s.manifest.block_height == block_height && s.manifest.version == version {
                s.last_access = SystemTime::now();
                snapshot = Some(s.clone());
                break;
            }
        }
        if snapshot.is_some() {
            self.state.snapshots.write(snapshots)?;
        }
        Ok(snapshot)
    }

    /// If the offered snapshot is accepted, we create a temporary directory to hold the chunks
    /// and remember it as our current snapshot being downloaded.
    pub fn offer_snapshot(&self, manifest: SnapshotManifest) -> StmResult<(), SnapshotError> {
        if manifest.version != 1 {
            abort(SnapshotError::IncompatibleVersion(manifest.version))
        } else {
            match tempdir() {
                Ok(dir) => {
                    let download = SnapshotDownload {
                        manifest,
                        download_dir: Arc::new(dir),
                        next_index: TVar::new(0),
                    };
                    self.state.current_download.write(Some(download))?;
                    Ok(())
                }
                Err(e) => abort(SnapshotError::from(e))?,
            }
        }
    }

    /// Take a chunk sent to us by a remote peer. This is our chance to validate chunks on the fly.
    ///
    /// Return a flag indicating whether all the chunks have been received and loaded to the blockstore.
    pub fn apply_chunk(&self, index: u32, contents: Vec<u8>) -> StmResult<bool, SnapshotError> {
        if let Some(cd) = self.state.current_download.read()?.as_ref() {
            let next_index = cd.next_index.read_clone()?;
            if index != next_index {
                abort(SnapshotError::UnexpectedChunk(next_index, index))
            } else {
                let part_path = cd
                    .download_dir
                    .as_ref()
                    .path()
                    .join(format!("{}.part", index));

                // We are doing IO inside the STM transaction, but that's okay because there is no contention on the download.
                match std::fs::write(part_path, contents) {
                    Ok(()) => {
                        let next_index = index + 1;
                        cd.next_index.write(next_index)?;

                        if next_index == cd.manifest.chunks {
                            // Verify the checksum then load the snapshot and remove the current download from memory.
                            match manifest::parts_checksum(cd.download_dir.as_ref()) {
                                Ok(checksum) => {
                                    if checksum == cd.manifest.checksum {
                                        // TODO: Import Snapshot.
                                        Ok(true)
                                    } else {
                                        abort(SnapshotError::WrongChecksum(
                                            cd.manifest.checksum,
                                            checksum,
                                        ))
                                    }
                                }
                                Err(e) => abort(SnapshotError::IoError(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    e.to_string(),
                                ))),
                            }
                        } else {
                            Ok(false)
                        }
                    }
                    Err(e) => {
                        // If we failed to save the data to disk we can return an error that will cause all snapshots to be aborted.
                        // There is no point trying to clear download from the state here because if we `abort` then all changes will be dropped.
                        abort(SnapshotError::from(e))
                    }
                }
            }
        } else {
            abort(SnapshotError::NoDownload)
        }
    }
}
