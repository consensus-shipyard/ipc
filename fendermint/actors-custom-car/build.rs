// Copyright 2022-2024 Protocol Labs
// Copyright Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0, MIT

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use build_rs_utils::echo;
use cargo_metadata::{DependencyKind, MetadataCommand};
use color_eyre::eyre::{bail, eyre, OptionExt, Result};
use fil_actor_bundler::Bundler;
use fs_err as fs;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use toml::Value;
use tracing::info;

fn parse_dependencies_of_umbrella_crate(manifest_path: &Path) -> Result<Vec<(String, PathBuf)>> {
    let manifest = fs::read_to_string(manifest_path)?;
    let document = manifest.parse::<Value>()?;

    let dependencies = document
        .get("target")
        .ok_or_eyre("could not find target table")?
        .get(r#"cfg(target_arch = "wasm32")"#)
        .ok_or_eyre("could not find target_arch table")?
        .get("dependencies")
        .ok_or_eyre("could not find dependencies table")?
        .as_table()
        .ok_or_eyre("could not find dependencies")?;

    let mut ret = Vec::with_capacity(dependencies.len());
    for (name, details) in dependencies.iter() {
        let Some(path) = details.get("path").and_then(Value::as_str) else {
            continue;
        };
        ret.push((name.clone(), std::path::PathBuf::from(path)));
    }

    Ok(ret)
}

pub fn rerun_if_changed(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
}

/// Custom wrapper for a [`cargo_metadata::Package`] to store it in
/// a `HashSet`.
#[derive(Debug)]
struct DeduplicatePackage<'a> {
    package: &'a cargo_metadata::Package,
    identifier: String,
}
impl<'a> From<&'a cargo_metadata::Package> for DeduplicatePackage<'a> {
    fn from(package: &'a cargo_metadata::Package) -> Self {
        Self {
            package,
            identifier: format!("{}{}{:?}", package.name, package.version, package.source),
        }
    }
}

impl std::hash::Hash for DeduplicatePackage<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl PartialEq for DeduplicatePackage<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for DeduplicatePackage<'_> {}

impl Deref for DeduplicatePackage<'_> {
    type Target = cargo_metadata::Package;

    fn deref(&self) -> &Self::Target {
        self.package
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Actor {
    package_name: String,
    crate_path: PathBuf,
    wasm_blob_path: PathBuf,
}

/// Bundle all indidivual wasm blobs together
fn bundle_wasm_blobs_into_car(actor_wasm_blobs: &[Actor], dst: &Path) -> Result<PathBuf> {
    let mut bundler = Bundler::new(dst);
    for (
        Actor {
            package_name: pkg,
            wasm_blob_path: path,
            ..
        },
        id,
    ) in actor_wasm_blobs.iter().zip(1u32..)
    {
        // This actor version doesn't force synthetic CIDs; it uses genuine
        // content-addressed CIDs.
        let forced_cid = None;

        let actor_name = pkg
            .strip_prefix("fendermint_actor_")
            .ok_or_eyre(format!(
                "expected fendermint_actor_ prefix in actor package name; got: {pkg}"
            ))?
            .to_owned();

        let cid = bundler
            .add_from_file(id, actor_name, forced_cid, path)
            .map_err(|err| {
                eyre!(
                    "failed to add file {:?} to bundle for actor {}: {}",
                    path,
                    id,
                    err
                )
            })?;
        info!("added {} ({}) to bundle with CID {}", pkg, id, cid);
    }

    bundler
        .finish()
        .map_err(|e| eyre!("Failed to finish bundle builder: {e}"))?;

    Ok(dst.to_path_buf())
}

fn create_metadata_command(path: impl Into<PathBuf>) -> MetadataCommand {
    let mut metadata_command = MetadataCommand::new();
    metadata_command.manifest_path(path);
    // metadata_command.other_options(vec!["--offline".to_owned()]);
    metadata_command
}

/// Find the `Cargo.lock` relative to the `OUT_DIR` environment variable.
///
/// If the `Cargo.lock` cannot be found, we emit a warning and return `None`.
fn find_cargo_lock(out_dir: &Path) -> Option<PathBuf> {
    fn find_impl(mut path: PathBuf) -> Option<PathBuf> {
        loop {
            if path.join("Cargo.lock").exists() {
                return Some(path.join("Cargo.lock"));
            }

            if !path.pop() {
                return None;
            }
        }
    }

    if let Some(path) = find_impl(out_dir.to_path_buf()) {
        return Some(path);
    }

    None
}

/// Track files and paths related to the given package to rerun `build.rs` on any relevant change.
fn package_rerun_if_changed(package: &DeduplicatePackage) {
    let mut manifest_path = package.manifest_path.clone();
    if manifest_path.ends_with("Cargo.toml") {
        manifest_path.pop();
    }

    ignore::Walk::new(&manifest_path)
        .filter(|p| {
            // Ignore this entry if it is a directory that contains a `Cargo.toml` that is not the
            // `Cargo.toml` related to the current package. This is done to ignore sub-crates of a
            // crate. If such a sub-crate is a dependency, it will be processed independently
            // anyway.
            let Ok(p) = p else { return false };
            let p = p.path();
            p == manifest_path || !p.is_dir() || !p.join("Cargo.toml").exists()
        })
        .filter_map(|p| p.ok().map(|p| p.into_path()))
        .filter(|p| {
            p.extension()
                .map(|e| e == "rs" || e == "toml")
                .unwrap_or_default()
        })
        .for_each(|ref x| rerun_if_changed(x));
}

fn build_all_wasm_blobs(
    channel: &str,
    target: &str,
    actors: &[Actor],
    cwd: &Path,
    manifest_path: &Path,
    out_dir: &Path,
) -> Result<Vec<Actor>> {
    echo!(
        "actors-custom-car",
        purple,
        "Building {} actors",
        actors.len()
    );

    let package_args = actors
        .iter()
        .map(|actor| format!("-p={}", actor.package_name));

    echo!(
        "actors-custom-car",
        purple,
        "Target: channel={channel} target={target}"
    );

    let rustup = which::which("rustup")?;

    let profile = "wasm-actor";
    // Cargo build command for all test_actors at once.
    let mut cmd = Command::new(rustup);
    cmd.arg("run")
        .arg(channel)
        .arg("cargo")
        .arg("build")
        .current_dir(cwd)
        .args(package_args)
        .arg("--target")
        .arg(target)
        .arg("--profile")
        .arg(profile)
        .arg("--features=fil-actor")
        .arg(format!("--manifest-path={}", manifest_path.display()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // We are supposed to only generate artifacts under OUT_DIR,
        // so set OUT_DIR as the target directory for this build.
        .env("CARGO_TARGET_DIR", out_dir)
        // As we are being called inside a build-script, this env variable is set. However, we set
        // our own `RUSTFLAGS` and thus, we need to remove this. Otherwise cargo favors this
        // env variable.
        .env_remove("CARGO_ENCODED_RUSTFLAGS");

    echo!(
        "actors-custom-car",
        purple,
        "Executing WASM compilation command: {:?}",
        &cmd
    );

    // Launch the command.
    let mut child = cmd.spawn()?;

    // Pipe the output as cargo warnings. Unfortunately this is the only way to
    // get cargo build to print the output.
    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");
    let j1 = thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            echo!("custom-actor-car", cyan, "{}", line.unwrap());
        }
    });
    let j2 = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            echo!("custom-actor-car", cyan, "{}", line.unwrap());
        }
    });

    let _ = j1.join();
    let _ = j2.join();

    let exit_status = child.wait()?;
    if !exit_status.success() {
        bail!("actor build faile");
    }
    Ok(actors.to_vec())
}

/// Use the `Cargo.lock` file to find the workspace manifest and extract dependency information
///
/// Requires `rustup` to be installed.
fn print_cargo_rerun_if_dependency_instructions(
    cargo_manifest: &Path,
    out_dir: &Path,
) -> Result<()> {
    // Rerun `build.rs` if the `Cargo.lock` changes
    let cargo_lock = find_cargo_lock(out_dir).ok_or_eyre("Couldn't find Cargo.lock")?;
    rerun_if_changed(&cargo_lock);

    let workspace_dir = cargo_lock
        .parent()
        .expect("A file always should have a parent");

    let metadata = create_metadata_command(workspace_dir.join("Cargo.toml")).exec()?;

    let package = metadata
        .packages
        .iter()
        .find(|p| p.manifest_path == cargo_manifest)
        .ok_or_eyre("Detected a circular dependency. The crate depends on itself..")?;

    // Start with the dependencies of the crate we want to compile for wasm.
    let mut dependencies = Vec::from_iter(package.dependencies.iter());

    // Collect all packages by follow the dependencies of all packages we find.
    let mut packages = indexmap::IndexSet::new();
    packages.insert(DeduplicatePackage::from(package));

    while let Some(dependency) = dependencies.pop() {
        // Ignore all dev dependencies
        if dependency.kind == DependencyKind::Development {
            continue;
        }

        let path_or_git_dep = dependency
            .source
            .as_ref()
            .map(|s| s.starts_with("git+"))
            .unwrap_or(true);

        let maybe_package = metadata
            .packages
            .iter()
            .filter(|p| !p.manifest_path.starts_with(workspace_dir))
            .find(|p| {
                // Check that the name matches and that the version matches or this is
                // a git or path dep. A git or path dependency can only occur once, so we don't
                // need to check the version.
                (path_or_git_dep || dependency.req.matches(&p.version)) && dependency.name == p.name
            });

        if let Some(package) = maybe_package {
            if packages.insert(DeduplicatePackage::from(package)) {
                dependencies.extend(package.dependencies.iter());
            }
        }
    }

    // Make sure that if any file/folder of a dependency change, we need to rerun the `build.rs`
    packages.iter().for_each(package_rerun_if_changed);

    Ok(())
}

fn main() -> Result<()> {
    // Rust provided out dir, used as intermediate place to store a) wasm target dir b) their outputs
    // the final `.car` file will sit under `dst`.
    let out_dir = std::env::var_os("OUT_DIR")
        .as_ref()
        .map(Path::new)
        .map(|p| p.join("bundle"))
        .ok_or_eyre("Must have OUT_DIR defined, this is only ever run from build.rs")?;

    let package_dir =
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or_eyre("CARGO_MANIFEST_DIR unset")?;
    let package_dir = Path::new(&package_dir);
    let actors_manifest_path = package_dir
        .parent()
        .unwrap()
        .join("actors")
        .join("Cargo.toml");
    let workspace_dir = fs_err::canonicalize(package_dir.parent().unwrap().parent().unwrap())?;

    let target = "wasm32-unknown-unknown";

    // one cannot specify the file, so parse it and use the toolchain explicity
    let toolchain_file_path = workspace_dir.join("rust-toolchain.toml");
    let toolchain_file = fs_err::read_to_string(&toolchain_file_path)?;
    let toolchain: toml::Table = toml::from_str(&toolchain_file)?;
    let toolchain = toolchain
        .get("toolchain")
        .ok_or_else(|| eyre!("Missing toolchain in {}", toolchain_file_path.display()))?;
    let channel: &toml::Value = toolchain
        .get("channel")
        .ok_or_else(|| eyre!("Missing channel in {}", toolchain_file_path.display()))?;
    let channel: String = channel.clone().try_into()?;
    // we use the `channel` as additional prefix to avoid compilation errors on
    // `rustc` upgrades in `rust-toolchain.toml`
    let out_dir = out_dir.join(&channel);

    let profile = "wasm-actor";

    // the joins represent the subdirectories under which `cargo` creates the actual WASM aritifacts
    let wasm_blob_dir = out_dir.join(target).join(profile);

    echo!(
        "actors-custom-car",
        purple,
        "Writing wasm blob to {}",
        wasm_blob_dir.display()
    );

    let actors = parse_dependencies_of_umbrella_crate(&actors_manifest_path)?;
    let actors = Vec::from_iter(actors.iter().map(|(name, crate_path)| Actor {
        package_name: name.as_str().to_owned(),
        wasm_blob_path: wasm_blob_dir.join(name.as_str()).with_extension("wasm"),
        crate_path: crate_path.clone(),
    }));

    print_cargo_rerun_if_dependency_instructions(&actors_manifest_path, &out_dir)?;

    build_all_wasm_blobs(
        &channel,
        target,
        &actors,
        &workspace_dir,
        &actors_manifest_path,
        &out_dir,
    )?;

    let bundle_car_dest_dir = actors_manifest_path.parent().unwrap().join("output");

    fs_err::create_dir_all(&bundle_car_dest_dir)?;
    let bundle_car_path = bundle_car_dest_dir.join("custom_actors_bundle.car");

    rerun_if_changed(&bundle_car_path);

    bundle_wasm_blobs_into_car(&actors, &bundle_car_path)?;

    Ok(())
}
