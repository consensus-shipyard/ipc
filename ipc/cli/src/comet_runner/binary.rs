// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

/// Embedded CometBFT binary data
static COMET_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/comet"));

/// Initialize the CometBFT binary
pub fn init_comet_binary() -> Result<PathBuf> {
    let file_name = if cfg!(windows) {
        "cometbft.exe"
    } else {
        "cometbft"
    };
    let binary_path = env::temp_dir().join(file_name);

    // Skip if already exists
    if binary_path.exists() {
        return Ok(binary_path);
    }

    // Write the binary
    fs::write(&binary_path, COMET_BIN).with_context(|| {
        format!(
            "failed to write CometBFT binary to {}",
            binary_path.display()
        )
    })?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    Ok(binary_path)
}
