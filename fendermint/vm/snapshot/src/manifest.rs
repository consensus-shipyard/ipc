// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};

use anyhow::Context;
use fendermint_vm_interpreter::fvm::state::{
    snapshot::{BlockHeight, SnapshotVersion},
    FvmStateParams,
};
use serde::{Deserialize, Serialize};

/// The file name in snapshot directories that contains the manifest.
const MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SnapshotManifest {
    /// Block height where the snapshot was taken.
    pub block_height: BlockHeight,
    /// Snapshot size in bytes.
    pub size: usize,
    /// Number of chunks in the snapshot.
    pub chunks: usize,
    /// SHA2 hash of the snapshot contents.
    ///
    /// Using a [tendermint::Hash] type because it has nice formatting in JSON.
    pub checksum: tendermint::Hash,
    /// The FVM parameters at the time of the snapshot,
    /// which are also in the CAR file, but it might be
    /// useful to see. It is annotated for human readability.
    pub state_params: FvmStateParams,
    /// Snapshot format version
    pub version: SnapshotVersion,
}

/// A snapshot directory and its manifest.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SnapshotItem {
    /// Directory containing this snapshot, ie. the manifest ane the parts.
    pub snapshot_dir: PathBuf,
    pub manifest: SnapshotManifest,
}

/// Save a manifest along with the other snapshot files into a snapshot specific directory.
pub fn write_manifest(
    snapshot_dir: impl AsRef<Path>,
    manifest: &SnapshotManifest,
) -> anyhow::Result<PathBuf> {
    let json =
        serde_json::to_string_pretty(&manifest).context("failed to convert manifest to JSON")?;

    let manifest_path = snapshot_dir.as_ref().join(MANIFEST_FILE_NAME);

    std::fs::write(&manifest_path, json).context("failed to write manifest file")?;

    Ok(manifest_path)
}

/// Collect all the manifests from a directory containing snapshot-directories, e.g.
/// `snapshots/snapshot-1/manifest.json` etc.
pub fn list_manifests(snapshot_dir: impl AsRef<Path>) -> anyhow::Result<Vec<SnapshotItem>> {
    let contents = std::fs::read_dir(snapshot_dir).context("failed to read snapshot directory")?;

    // Collect all manifest file paths.
    let mut manifests = Vec::new();
    for entry in contents {
        match entry {
            Ok(entry) => match entry.metadata() {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        let manifest_path = entry.path().join(MANIFEST_FILE_NAME);
                        if manifest_path.exists() {
                            manifests.push((entry.path(), manifest_path))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error =? e, "faulty entry metadata");
                }
            },
            Err(e) => {
                tracing::error!(error =? e, "faulty snapshot entry");
            }
        }
    }

    // Parse manifests
    let mut items = Vec::new();
    for (snapshot_dir, manifest) in manifests {
        let json = std::fs::read_to_string(&manifest).context("failed to open manifest")?;
        match serde_json::from_str(&json) {
            Ok(manifest) => items.push(SnapshotItem {
                snapshot_dir,
                manifest,
            }),
            Err(e) => {
                tracing::error!(
                    manifest = manifest.to_string_lossy().to_string(),
                    error =? e,
                    "unable to parse snapshot manifest"
                );
            }
        }
    }

    // Order by oldest to newest.
    items.sort_by_key(|i| i.manifest.block_height);

    Ok(items)
}

#[cfg(feature = "arb")]
mod arb {
    use fendermint_testing::arb::{ArbCid, ArbTokenAmount};
    use fendermint_vm_core::{chainid, Timestamp};
    use fendermint_vm_interpreter::fvm::state::FvmStateParams;
    use fvm_shared::version::NetworkVersion;
    use quickcheck::Arbitrary;

    use super::SnapshotManifest;

    impl quickcheck::Arbitrary for SnapshotManifest {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let checksum: [u8; 32] = std::array::from_fn(|_| u8::arbitrary(g));

            Self {
                block_height: Arbitrary::arbitrary(g),
                size: Arbitrary::arbitrary(g),
                chunks: Arbitrary::arbitrary(g),
                checksum: tendermint::Hash::from_bytes(
                    tendermint::hash::Algorithm::Sha256,
                    &checksum,
                )
                .unwrap(),
                state_params: FvmStateParams {
                    state_root: ArbCid::arbitrary(g).0,
                    timestamp: Timestamp(Arbitrary::arbitrary(g)),
                    network_version: NetworkVersion::MAX,
                    base_fee: ArbTokenAmount::arbitrary(g).0,
                    circ_supply: ArbTokenAmount::arbitrary(g).0,
                    chain_id: chainid::from_str_hashed(String::arbitrary(g).as_str())
                        .unwrap()
                        .into(),
                    power_scale: *g.choose(&[-1, 0, 3]).unwrap(),
                },
                version: Arbitrary::arbitrary(g),
            }
        }
    }
}
