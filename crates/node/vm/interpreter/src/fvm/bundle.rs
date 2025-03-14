// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

// TODO factor out into a test support crate
// Find the root of the workspace, not this crate, which is what `env!("CARGO_MANIFEST_DIR")` would return
fn cargo_workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;

    let cargo_path = PathBuf::from(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().parent().unwrap().to_path_buf()
}

/// Path to the Solidity contracts, indended to be used in tests.
pub fn contracts_path() -> PathBuf {
    std::env::var("FM_CONTRACTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| cargo_workspace_dir().join("contracts").join("out"))
}
