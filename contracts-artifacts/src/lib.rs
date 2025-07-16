// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::Context;
use rust_embed::RustEmbed;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

#[derive(RustEmbed)]
#[folder = "../contracts/out"]
pub struct Artifacts;

pub fn extract_to_tempdir() -> anyhow::Result<(TempDir, PathBuf)> {
    let tmp = TempDir::new().context("failed to create temp dir")?;
    let base = tmp.path().to_path_buf();

    for file in Artifacts::iter() {
        let rel = std::path::Path::new(file.as_ref());
        let dst = base.join(rel);

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create dir {:?}", parent))?;
        }

        let embedded = Artifacts::get(file.as_ref())
            .with_context(|| format!("missing embedded file {}", file.as_ref()))?;

        fs::write(&dst, embedded.data.as_ref())
            .with_context(|| format!("failed to write {:?}", dst))?;
    }

    Ok((tmp, base))
}
