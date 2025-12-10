//! Adapted from https://github.com/foundry-rs/foundry/blob/60f0b692acae47a4933bb4a0bc4a29cab8831ba1/crates/forge/bin/cmd/bind.rs
//!
//! This build script generates Rust bindings for Solidity contracts using Forge.
//!
//! Ideally, this script would programmatically execute `forge install` and `forge build`
//! to avoid committing generated artifacts (the bindings) to version control.
//! This is the standard practice for build outputs.
//!
//! Currently, downstream crates can use the pre-generated bindings directly.
//! However, this requires developers to manually run `make rust-bindings` (which performs the
//! Forge build and bind) whenever the Solidity facades change and then commit the resulting
//! changes to version control.
//!
//! While convenient for downstream users, this approach is suboptimal.
//! A future improvement would be to implement programmatic `forge install` and `forge build`
//! within this script, eliminating the manual steps and the need to commit build
//! artifacts.
//! This would ensure that downstream crates always use up-to-date bindings without relying on
//! potentially outdated committed versions and would streamline the development workflow.
//!
//! SPDX-License-Identifier: Apache-2.0, MIT

use std::path::{Path, PathBuf};

use alloy_primitives::map::HashSet;
use eyre::Result;
use forge::{fs::json_files, MultiSolMacroGen, SolMacroGen};
use regex::Regex;

mod forge;

const FACADES: &[&str] = &[
    "BlobReader",
    "Blobs",
    "Bucket",
    "Config",
    "Credit",
    "Gas",
    "Machine",
    "Timehub",
];

fn main() {
    if std::env::var("BUILD_BINDINGS").unwrap_or("0".to_string()) == "0" {
        return;
    }

    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let artifacts_dir = PathBuf::from(format!("{}/../../out", cargo_dir));

    for facade in FACADES {
        let out_dir = PathBuf::from(format!(
            "{}/src/{}_facade",
            cargo_dir,
            facade.to_lowercase()
        ));
        let select = Regex::new(format!("I{}Facade", facade).as_str()).unwrap();
        let binder = ForgeBinder {
            artifacts: artifacts_dir.clone(),
            out: out_dir,
            select: vec![select],
        };
        binder
            .run()
            .unwrap_or_else(|_| panic!("failed to generate {} bindings", facade));
    }
}

#[derive(Clone, Debug)]
pub struct ForgeBinder {
    pub artifacts: PathBuf,
    pub out: PathBuf,
    pub select: Vec<Regex>,
}

impl ForgeBinder {
    pub fn run(self) -> Result<()> {
        self.generate_bindings(&self.artifacts, &self.out)?;
        Ok(())
    }

    fn get_filter(&self) -> Result<Filter> {
        Ok(Filter::Select(self.select.clone()))
    }

    /// Returns an iterator over the JSON files and the contract name in the `artifacts` directory.
    fn get_json_files(&self, artifacts: &Path) -> Result<impl Iterator<Item = (String, PathBuf)>> {
        let filter = self.get_filter()?;
        Ok(json_files(artifacts)
            .filter_map(|path| {
                // Ignore the build info JSON.
                if path.to_str()?.contains("build-info") {
                    return None;
                }

                // We don't want `.metadata.json` files.
                let stem = path.file_stem()?.to_str()?;
                if stem.ends_with(".metadata") {
                    return None;
                }

                let name = stem.split('.').next().unwrap();

                // Best effort identifier cleanup.
                let name = name.replace(char::is_whitespace, "").replace('-', "_");

                Some((name, path))
            })
            .filter(move |(name, _path)| filter.is_match(name)))
    }

    fn get_solmacrogen(&self, artifacts: &Path) -> Result<MultiSolMacroGen> {
        let mut dup = HashSet::<String>::default();
        let instances = self
            .get_json_files(artifacts)?
            .filter_map(|(name, path)| {
                if dup.insert(name.clone()) {
                    Some(SolMacroGen::new(path, name))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let multi = MultiSolMacroGen::new(instances);
        eyre::ensure!(!multi.instances.is_empty(), "No contract artifacts found");
        Ok(multi)
    }

    /// Generate the bindings
    fn generate_bindings(&self, artifacts: &Path, bindings_root: &Path) -> Result<()> {
        let mut solmacrogen = self.get_solmacrogen(artifacts)?;
        solmacrogen.write_to_module(bindings_root, false)
    }
}

pub enum Filter {
    All,
    Select(Vec<Regex>),
    Skip(Vec<Regex>),
}

impl Filter {
    pub fn is_match(&self, name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Select(regexes) => regexes.iter().any(|regex| regex.is_match(name)),
            Self::Skip(regexes) => !regexes.iter().any(|regex| regex.is_match(name)),
        }
    }

    pub fn skip_default() -> Self {
        let skip = [
            ".*Test.*",
            ".*Script",
            "console[2]?",
            "CommonBase",
            "Components",
            "[Ss]td(Chains|Math|Error|Json|Utils|Cheats|Style|Invariant|Assertions|Toml|Storage(Safe)?)",
            "[Vv]m.*",
            "IMulticall3",
        ]
            .iter()
            .map(|pattern| Regex::new(pattern).unwrap())
            .collect::<Vec<_>>();

        Self::Skip(skip)
    }
}
