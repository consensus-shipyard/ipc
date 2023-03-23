// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};
use std::str::FromStr;

fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

/// Path to the builtin-actor bundle, indended to be used in tests.
pub fn bundle_path() -> PathBuf {
    let bundle_path = std::env::var("BUILTIN_ACTORS_BUNDLE").unwrap_or_else(|_| {
        workspace_dir()
            .join("../builtin-actors/output/bundle.car")
            .to_string_lossy()
            .into_owned()
    });

    PathBuf::from_str(&bundle_path).expect("malformed bundle path")
}
