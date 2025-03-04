// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use color_eyre::eyre::{eyre, OptionExt, Result};
use fil_actor_bundler::Bundler;
use fs_err as fs;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use toml::Value;


// Taken from `polkadot-sdk`'s `wasm-builder`
fn generate_rerun_if_changed_instructions(
	cargo_manifest: &Path,
	project_folder: &Path,
	wasm_workspace: &Path,
) {
	// Rerun `build.rs` if the `Cargo.lock` changes
	if let Some(cargo_lock) = find_cargo_lock(cargo_manifest) {
		rerun_if_changed(cargo_lock);
	}

	let metadata = create_metadata_command(project_folder.join("Cargo.toml"))
		.exec()
		.expect("`cargo metadata` can not fail!");

	let package = metadata
		.packages
		.iter()
		.find(|p| p.manifest_path == cargo_manifest)
		.expect("The crate package is contained in its own metadata; qed");

	// Start with the dependencies of the crate we want to compile for wasm.
	let mut dependencies = package.dependencies.iter().collect::<Vec<_>>();

	// Collect all packages by follow the dependencies of all packages we find.
	let mut packages = HashSet::new();
	packages.insert(DeduplicatePackage::from(package));

	while let Some(dependency) = dependencies.pop() {
		// Ignore all dev dependencies
		if dependency.kind == DependencyKind::Development {
			continue
		}

		let path_or_git_dep =
			dependency.source.as_ref().map(|s| s.starts_with("git+")).unwrap_or(true);

		let package = metadata
			.packages
			.iter()
			.filter(|p| !p.manifest_path.starts_with(wasm_workspace))
			.find(|p| {
				// Check that the name matches and that the version matches or this is
				// a git or path dep. A git or path dependency can only occur once, so we don't
				// need to check the version.
				(path_or_git_dep || dependency.req.matches(&p.version)) && dependency.name == p.name
			});

		if let Some(package) = package {
			if packages.insert(DeduplicatePackage::from(package)) {
				dependencies.extend(package.dependencies.iter());
			}
		}
	}

	// Make sure that if any file/folder of a dependency change, we need to rerun the `build.rs`
	packages.iter().for_each(package_rerun_if_changed);

	compressed_or_compact_wasm.map(|w| rerun_if_changed(w.wasm_binary_path()));
	rerun_if_changed(bloaty_wasm.bloaty_path());

	// Register our env variables
	println!("cargo:rerun-if-env-changed={}", crate::SKIP_BUILD_ENV);
	println!("cargo:rerun-if-env-changed={}", crate::WASM_BUILD_TYPE_ENV);
	println!("cargo:rerun-if-env-changed={}", crate::WASM_BUILD_RUSTFLAGS_ENV);
	println!("cargo:rerun-if-env-changed={}", crate::WASM_TARGET_DIRECTORY);
	println!("cargo:rerun-if-env-changed={}", crate::WASM_BUILD_TOOLCHAIN);
	println!("cargo:rerun-if-env-changed={}", crate::WASM_BUILD_STD);
	println!("cargo:rerun-if-env-changed={}", crate::RUNTIME_TARGET);
	println!("cargo:rerun-if-env-changed={}", crate::WASM_BUILD_CARGO_ARGS);
}

/// A wasm actor we have yet to build
pub struct ActorToBuild {
    pub idx: usize,
    pub pkg_name: String,
    pub manifest_path: PathBuf,
}

/// Build all actor files into their respective wasm blobs
pub fn build(mut actors: impl Iterator<Item=ActorToBuild>, cargo: impl AsRef<Path>, dest: impl AsRef<Path>) -> color_eyre::eyre::Result<std::path::PathBuf> {
    let cargo = cargo.as_ref();
    let dest = dest.as_ref();
    let manifest_path = manifest_path.as_ref();

    // Cargo build command for all test_actors at once.
    let mut cmd = Command::new(cargo);
    cmd.arg("build")
        .args(actors.iter().map(|actor| format!("-p={}", actor.pkg_name)))
        .arg("--target=wasm32-unknown-unknown")
        .arg("--profile=wasm")
        .arg("--features=fil-actor")
        .arg(format!("--manifest-path={}", actor.manifest_path.display()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // We are supposed to only generate artifacts under OUT_DIR,
        // so set OUT_DIR as the target directory for this build.
        .env("CARGO_TARGET_DIR", &out_dir)
        // As we are being called inside a build-script, this env variable is set. However, we set
        // our own `RUSTFLAGS` and thus, we need to remove this. Otherwise cargo favors this
        // env variable.
        .env_remove("CARGO_ENCODED_RUSTFLAGS");

    // Print out the command line we're about to run.
    println!("cargo:warning=cmd={:?}", &cmd);

    // Launch the command.
    let mut child = cmd.spawn()?;

    // Pipe the output as cargo warnings. Unfortunately this is the only way to
    // get cargo build to print the output.
    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");
    let j1 = thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            println!("cargo:warning={:?}", line.unwrap());
        }
    });
    let j2 = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            println!("cargo:warning={:?}", line.unwrap());
        }
    });

    j1.join().unwrap();
    j2.join().unwrap();

    let happy = child.wait().is_ok_and(|exit| exit.success());
    if !happy {
        bail!("actor build failed");
    }

    // Create the bundle, use the individual wasm blobs
    fs::create_dir_all(dest)?;

    info!("Created bundle {}", dst.display());

    Ok(dst)
}
