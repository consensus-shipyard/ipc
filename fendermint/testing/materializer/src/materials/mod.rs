// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::Path;

mod defaults;

use anyhow::Context;
pub use defaults::*;

/// Type family of all the things a [Materializer] can create.
///
/// Kept separate from the [Materializer] so that we can wrap one in another
/// and pass the same types along.
pub trait Materials {
    /// Represents the entire hierarchy of a testnet, e.g. a common docker network
    /// and directory on the file system. It has its own type so the materializer
    /// doesn't have to remember what it created for a testnet, and different
    /// testnets can be kept isolated from each other.
    type Network: Send + Sync;
    /// Capture where the IPC stack (the gateway and the registry) has been deployed on a subnet.
    /// These are the details which normally go into the `ipc-cli` configuration files.
    type Deployment: Sync + Send;
    /// Represents an account identity, typically a key-value pair.
    type Account: Ord + Sync + Send;
    /// Represents the genesis.json file (can be a file location, or a model).
    type Genesis: Sync + Send;
    /// The address of a dynamically created subnet.
    type Subnet: Sync + Send;
    /// The handle to a node; could be a (set of) docker container(s) or remote addresses.
    type Node: Sync + Send;
    /// The handle to a relayer process.
    type Relayer: Sync + Send;
}

/// Write some content to a file.
///
/// It will create all the directories along the path.
pub fn export(
    output_dir: impl AsRef<Path>,
    name: &str,
    ext: &str,
    contents: impl AsRef<str>,
) -> anyhow::Result<()> {
    let file_name = if ext.is_empty() {
        name.into()
    } else {
        format!("{name}.{ext}")
    };

    let dir_path = output_dir.as_ref();
    let file_path = dir_path.join(file_name);

    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path).with_context(|| {
            format!("failed to create directory {}", dir_path.to_string_lossy())
        })?;
    }

    std::fs::write(&file_path, contents.as_ref())
        .with_context(|| format!("failed to write to {}", file_path.to_string_lossy()))?;

    Ok(())
}