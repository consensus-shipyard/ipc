// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.

use std::error::Error;
use std::path::Path;
use std::process::Command;

const ACTORS: &[&str] = &["chainmetadata_v2"];

fn main() -> Result<(), Box<dyn Error>> {
    let cargo = std::env::var_os("CARGO").expect("no CARGO env var");

    let out_dir = std::env::var_os("OUT_DIR")
        .as_ref()
        .map(Path::new)
        .map(|p| p.join("upgrade_examples"))
        .expect("no OUT_DIR env var");

    let mut cmd = Command::new(cargo);
    cmd.arg("build")
        .args(
            ACTORS
                .iter()
                .map(|pkg| "-p=fendermint_actor_".to_owned() + pkg),
        )
        .arg("--target=wasm32-unknown-unknown")
        .arg("--profile=wasm")
        .arg("--features=fil-actor")
        // We are supposed to only generate artifacts under OUT_DIR,
        // so set OUT_DIR as the target directory for this build.
        .env("CARGO_TARGET_DIR", &out_dir)
        // As we are being called inside a build-script, this env variable is set. However, we set
        // our own `RUSTFLAGS` and thus, we need to remove this. Otherwise cargo favors this
        // env variable.
        .env_remove("CARGO_ENCODED_RUSTFLAGS");

    let mut child = cmd.spawn().expect("failed to launch cargo build");
    let result = child.wait().expect("failed to wait for build to finish");
    if !result.success() {
        return Err("actor build failed".into());
    }

    std::fs::create_dir_all("output")
        .expect("failed to create output dir for the upgrade_examples actors");

    for pkg in ACTORS.iter() {
        let bytecode_path = Path::new(&out_dir)
            .join("wasm32-unknown-unknown/wasm")
            .join(format!("fendermint_actor_{}.wasm", pkg));

        std::fs::copy(
            &bytecode_path,
            format!("output/fendermint_actor_{}.wasm", pkg),
        )
        .unwrap();
    }

    Ok(())
}
