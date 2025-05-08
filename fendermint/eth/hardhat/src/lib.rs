// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Provides a "linker" for artifacts created by `forge build` in the hardhat directory structure
//!
//! The hardhat directory structure is a folder `Foo.sol` with a contained `Foo.json`.
//! The JSON file encodes the `ABI` in field `abi` as well as input variables and their types.
//! There are more fields, such as `bytecode`, `deployedBytecode` and `linkReferences` - the latter can be referenced from within the `bytecode`.

use anyhow::{anyhow as eyre, bail, Context, Result};
// use color_eyre::eyre::{self, bail, Context, ContextCompat, Result};
use core::{fmt, iter::Iterator};
use ethers_core::types as et;
use fendermint_vm_actor_interface::{diamond::EthContractMap, ipc::IPC_CONTRACTS};
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ord,
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    hash::Hash,
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

/// Contract source as it appears in dependencies, e.g. `"src/lib/SubnetIDHelper.sol"`, or "Gateway.sol".
/// It is assumed to contain the file extension.
pub type ContractSource = PathBuf;

/// Contract name as it appears in dependencies, e.g. `"SubnetIDHelper"`.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractName(pub(crate) String);

impl From<&'_ ContractName> for ContractName {
    fn from(value: &'_ ContractName) -> Self {
        value.clone()
    }
}

impl From<&'_ str> for ContractName {
    fn from(value: &'_ str) -> Self {
        Self(value.to_owned())
    }
}

impl ContractName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ContractName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for ContractName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractSourceAndName {
    pub name: ContractName,
    pub source: ContractSource,
}

/// Artifact from Hardhat build artifacts.
#[derive(Debug, PartialEq)]
pub struct DeploymentArtifact {
    pub bytecode: Vec<u8>,
    pub abi: ethers_core::abi::Abi,
}

/// Fully Qualified Name of a contract, e.g. `"src/lib/SubnetIDHelper.sol:SubnetIDHelper"`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FullyQualifiedName(pub(crate) String);

impl FullyQualifiedName {
    pub fn new(path: impl Into<ContractSource>, name: impl Into<ContractName>) -> Self {
        let path = path.into();
        let name = name.into();
        
        Self(format!(
            "{}:{}",
            path.to_string_lossy(),
            name.as_str()
        ))
    }
}

impl fmt::Display for FullyQualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Dependency tree for libraries.
///
/// Using a [BTreeMap] for deterministic ordering.
type DependencyTree<T> = BTreeMap<T, HashSet<T>>;

/// Hold all compiled and linked bytes in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidityActorContracts {
    /// that contain ABI, bytecode, link references, etc. keyed by their name.
    top_level: HashMap<ContractName, YetToLinkContractArtifact>,
    /// Eth libs that are not top level contracts
    libs: HashMap<ContractName, YetToLinkContractArtifact>,
}

impl SolidityActorContracts {
    /// Collects library and top-level contracts
    ///
    /// Returns a tuple containing a vector of library contracts
    /// and a map of top-level contracts.
    pub fn collect_contracts(&self) -> Result<(Vec<ContractName>, EthContractMap)> {
        let mut all_contract_names = Vec::new();
        let top_level_contracts = IPC_CONTRACTS.clone();

        // Add top-level contract names and their facet names.
        all_contract_names.extend(top_level_contracts.keys().map(|&x| x.to_string()));
        all_contract_names.extend(
            top_level_contracts
                .values()
                .flat_map(|c| c.facets.iter().map(|f| f.name.to_owned())),
        );

        let contract_names = Vec::from_iter(
            all_contract_names
                .iter()
                .cloned()
                .map(ContractName),
        );

        let mut eth_libs = self
            .dependencies(&contract_names)
            .with_context(|| "failed to collect EVM contract dependencies XX")?;

        // Keep only library contracts (exclude top-level ones).
        eth_libs.retain(|contract_name| !top_level_contracts.contains_key(contract_name.as_str()));

        Ok((eth_libs, top_level_contracts))
    }

    /// TODO
    pub fn resolve_library_references(
        &self,
        contract_name_which: &ContractName,
        library_addresses: &HashMap<ContractName, et::Address>,
    ) -> Result<DeploymentArtifact> {
        let top_level_artifact = self
            .get_contract(contract_name_which)
            .ok_or_else(|| eyre!("No such contract: {}", contract_name_which))?;
        let abi = top_level_artifact.abi.clone();
        let bytecode = top_level_artifact.resolve_library_references(library_addresses)?;
        Ok(DeploymentArtifact { bytecode, abi })
    }

    /// Traverse the linked references and return the library contracts to be deployed in topological order.
    ///
    /// The result will include the top contracts as well, and it's up to the caller to filter them out if
    /// they have more complicated deployments including constructors. This is because there can be diamond
    /// facets among them which aren't ABI visible dependencies but should be deployed as libraries.
    pub fn dependencies<I: IntoIterator<Item = S>, S: ToString>(
        &self,
        root_contracts: I,
    ) -> Result<Vec<ContractName>> {
        dbg!(self);
        let mut deps = DependencyTree::<ContractName>::default();

        let mut queue =
            VecDeque::<ContractName>::from_iter(root_contracts.into_iter().map(|name| {
                let name = ContractName(name.to_string());
                tracing::info!("Root contract: {name}");
                name
            }));
        tracing::info!("Root contracts # = {}", queue.len());

        // Construct dependency tree by recursive traversal.
        while let Some(name) = queue.pop_front() {
            if deps.contains_key(&name) {
                // if dependency already contains the name, we don't need to do anything
                tracing::debug!("Already processed dep: contract {name}");
                continue;
            }

            tracing::info!("Processing queue: contract {name}");

            let artifact = self
                .get_lib(&name)
                .inspect(|&_x| {
                    tracing::info!("Dependency lib: {name}",);
                })
                .or_else(|| self.get_top_level(&name))
                .inspect(|_x| {
                    tracing::info!("Dependency top: {name}");
                })
                .ok_or_else(|| eyre!("Failed to load dependency artifact: {name}"))?;

            let entry: &mut HashSet<_> = deps.entry(name.clone()).or_default();

            for (_lib_source, lib_name) in artifact.libraries_needed() {
                tracing::info!("Adding dependency edge: {name} -> {lib_name}");
                entry.insert(lib_name.clone());
                queue.push_back(lib_name);
            }
        }

        // Topo-sort the libraries in the order of deployment.
        let sorted = topo_sort::<ContractName>(deps)?;

        Ok(sorted)
    }

    /// Obtain the compiled library contract
    pub fn get_lib(&self, contract_name: &ContractName) -> Option<&YetToLinkContractArtifact> {
        self.libs.get(contract_name)
    }

    /// Obtain the compiled top level contract
    ///
    /// Note: Does not yield any _facets_ of the top-level contracts. Those are considered libs.
    pub fn get_top_level(
        &self,
        contract_name: &ContractName,
    ) -> Option<&YetToLinkContractArtifact> {
        self.top_level.get(contract_name)
    }

    /// Obtain the compiled top level OR library contract
    pub fn get_contract(&self, contract_name: &ContractName) -> Option<&YetToLinkContractArtifact> {
        self.top_level
            .get(contract_name)
            .or_else(|| self.get_lib(contract_name))
    }

    /// Wrap it into a json object
    ///
    /// Used for storing and in-binary inclusion
    pub fn from_json(reader: impl std::io::Read) -> Result<Self> {
        Ok(serde_json::from_reader(reader)?)
    }

    /// Wrap it into a json object
    ///
    pub fn to_json(&self) -> Result<String> {
        let s = serde_json::to_string_pretty(self)?;
        Ok(s)
    }
}

/// Utility to link bytecode from `forge build` process build artifacts.
#[derive(Clone, Debug)]
pub struct SolidityActorContractsLoader {
    contracts_dir: PathBuf,
}

fn as_file_name(name: &str) -> ContractSource {
    PathBuf::from(format!("{name}.sol"))
}

pub fn as_contract_name(name: impl AsRef<str>) -> ContractName {
    ContractName(name.as_ref().to_string())
}

impl SolidityActorContractsLoader {
    /// Load the contracts from the hardhat format disk
    ///
    /// The output directory used with `forge build` the full-fat JSON files
    /// that contain ABI, bytecode, link references, etc.
    ///
    /// Contract paths are derived from the name and the `self.contracts_dir`.
    fn load_directory_inner(&self) -> Result<SolidityActorContracts> {
        let mut all_contracts = Vec::<ContractName>::new();
        let mut top_level_contracts = EthContractMap::default();

        top_level_contracts.extend(IPC_CONTRACTS.clone());
        dbg!(Vec::from_iter(top_level_contracts.keys()));

        all_contracts.extend(top_level_contracts.keys().map(as_contract_name));
        all_contracts.extend(
            top_level_contracts
                .values()
                .flat_map(|c| c.facets.iter().map(|f| as_contract_name(&f.name))),
        );
        // Collect dependencies of the main IPC actors.
        let mut lib_deps = Vec::from_iter(
            self.dependencies(
                all_contracts
                    .iter()
                    .map(|name| (as_file_name(name.as_str()), name.clone())),
            )
            .context("failed to collect EVM contract dependencies 4")?
            .into_iter()
            .map(|ContractSourceAndName { name, source }| (source, name)),
        );

        // Only keep library dependencies, not contracts with constructors.
        lib_deps.retain(|(_, d): &(_, ContractName)| !top_level_contracts.contains_key(d.as_str()));

        let top_level_artifacts =
            Result::<HashMap<ContractName, YetToLinkContractArtifact>>::from_iter(
                top_level_contracts.into_keys().map(|name| {
                        let contract_name = as_contract_name(name);
                        let contract_path = as_file_name(name);
                        let artifact: YetToLinkContractArtifact = self
                            .artifact(&contract_name, &contract_path)
                            .context(format!("Failed to load top level {name}"))?;
                        Ok((contract_name, artifact))
                    }),
            )?;

        let lib_artifacts =
            Result::<HashMap<_, YetToLinkContractArtifact>>::from_iter(lib_deps.into_iter().map(
                |(_source, contract_name): (_, ContractName)| {
                    let contract_path: PathBuf = as_file_name(contract_name.as_str());
                    let artifact = self.artifact(&contract_name, &contract_path)?;
                    Ok((contract_name, artifact))
                },
            ))?;

        Ok(SolidityActorContracts {
            top_level: top_level_artifacts,
            libs: lib_artifacts,
        })
    }

    /// A new loader
    pub fn load_directory(contract_dir: &Path) -> Result<SolidityActorContracts> {
        let loader = Self {
            contracts_dir: contract_dir.to_path_buf(),
        };
        dbg!(loader.load_directory_inner())
    }

    /// Construct bytecode from the compiled contract path
    ///
    /// Also replaces all linked libraries
    #[deprecated(note = "Should decouple IO from logic ops")]
    pub fn bytecode(
        &self,
        contract_path: impl AsRef<Path>,
        contract_name: &ContractName,
        libraries: &HashMap<ContractName, et::Address>,
    ) -> Result<Vec<u8>> {
        let artifact = self.artifact(contract_name, contract_path.as_ref())?;
        artifact.resolve_library_references(libraries)
    }

    /// Traverse the linked references and return the library contracts to be deployed in topological order.
    ///
    /// The result will include the top contracts as well, and it's up to the caller to filter them out if
    /// they have more complicated deployments including constructors. This is because there can be diamond
    /// facets among them which aren't ABI visible dependencies but should be deployed as libraries.
    #[deprecated(note = "Use `SolidityActorContracs::dependencies` instead, decouple IO")]
    pub fn dependencies<I: IntoIterator<Item = (A, S)>, S: ToString, A: AsRef<Path>>(
        &self,
        root_contracts: I,
    ) -> Result<Vec<ContractSourceAndName>> {
        let mut deps: DependencyTree<ContractSourceAndName> = Default::default();

        let mut queue =
            VecDeque::<ContractSourceAndName>::from_iter(root_contracts.into_iter().map(
                |(s, c)| ContractSourceAndName {
                    name: ContractName(c.to_string()),
                    source: PathBuf::from(s.as_ref()),
                },
            ));

        // Construct dependency tree by recursive traversal.
        while let Some(sc) = queue.pop_front() {
            if deps.contains_key(&sc) {
                continue;
            }
            let ContractSourceAndName { name, source } = &sc;

            let artifact = self.artifact(name, source).with_context(|| {
                format!(
                    "failed to load dependency artifact: {} from {}",
                    name,
                    source.display()
                )
            })?;

            let entry = deps.entry(sc).or_default();

            for (lib_path, lib_name) in artifact.libraries_needed() {
                let needed_library = ContractSourceAndName {
                    source: lib_path,
                    name: lib_name,
                };
                entry.insert(needed_library.clone());
                queue.push_back(needed_library);
            }
        }

        // Topo-sort the libraries in the order of deployment.
        let sorted = topo_sort(deps)?;

        Ok(sorted)
    }

    /// Concatenate the contracts directory with the expected layout to get
    /// the path to the JSON file of a contract, which is under a directory
    /// named after the Solidity file.
    fn contract_path(&self, contract_path: &Path, contract_name: &ContractName) -> Result<PathBuf> {
        // There is currently no example of a Solidity directory containing multiple JSON files,
        // but it possible if there are multiple contracts in the file.

        let base_name = contract_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| eyre!("failed to produce base name for {contract_path:?}"))?;

        let path = self
            .contracts_dir
            .join(base_name)
            .join(contract_name.as_str())
            .with_extension("json");

        Ok(path)
    }

    /// Parse the Hardhat artifact of a contract.
    ///
    /// Note: Does the actual IO and parses from json to a rust type.
    fn artifact(
        &self,
        contract_name: &ContractName,
        contract_src: &Path,
    ) -> Result<YetToLinkContractArtifact> {
        let contract_path = self.contract_path(contract_src, contract_name)?;

        let json = fs::read_to_string(&contract_path)
            .context(format!("failed to read {}", contract_path.display()))?;

        let artifact =
            serde_json::from_str::<YetToLinkContractArtifact>(&json).context(format!(
                "failed to parse yet to link contract artifact JSON from {}",
                contract_path.display()
            ))?;

        Ok(artifact)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct YetToLinkContractArtifact {
    pub bytecode: Bytecode,
    pub abi: ethers_core::abi::Abi,
}

impl fmt::Debug for YetToLinkContractArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let deps = self.libraries_needed();
        write!(f, "deps ({}) [ ", deps.len())?;
        for (_, libname) in deps {
            write!(f, "{}, ", libname)?;
        }
        write!(f, "] - ")?;

        let n = self.bytecode.object.len();
        write!(f, "{n} bytes [ ")?;
        let s = self.bytecode.object.as_bytes();
        let k = n.saturating_sub(20);
        if k < 2 {
            for byte in &s[..(n - k)] {
                write!(f, "{:02x} ", byte)?;
            }
        } else {
            let r = (n - k) / 2;
            for byte in &s[0..r] {
                write!(f, "{:02x} ", byte)?;
            }
            write!(f, " ... ")?;
            for byte in &s[(n - r)..][..(r - 1)] {
                write!(f, "{:02x} ", byte)?;
            }
        }
        write!(f, "{:02x}", s[n - 1])?;
        write!(f, "]")
    }
}

impl YetToLinkContractArtifact {
    // Collect the libraries this contract needs
    pub fn libraries_needed(&self) -> Vec<(ContractSource, ContractName)> {
        Vec::from_iter(
            self.bytecode
                .link_references
                .iter()
                .flat_map(|(lib_src_path, links)| {
                    links
                        .keys()
                        .map(|lib_name| (lib_src_path.to_owned(), lib_name.to_owned()))
                }),
        )
    }

    pub fn library_positions(
        &self,
        lib_src: &ContractSource,
        lib_name: &ContractName,
    ) -> impl Iterator<Item = &Position> {
        match self
            .bytecode
            .link_references
            .get(lib_src)
            .and_then(|links| links.get(lib_name))
        {
            Some(ps) => ps.iter(),
            None => [].iter(),
        }
    }

    /// Read the bytecode of the contract and replace all links in it with library addresses,
    /// similar to how the [hardhat-ethers](https://github.com/NomicFoundation/hardhat/blob/7cc06ab222be8db43265664c68416fdae3030418/packages/hardhat-ethers/src/internal/helpers.ts#L165C42-L165C42)
    /// plugin does it.
    ///
    /// The contract source is expected to be the logical path to a Solidity contract,
    /// including the extension, ie. a [`ContractSource`].
    pub fn resolve_library_references(
        &self,
        libraries: &HashMap<ContractName, et::Address>,
    ) -> Result<Vec<u8>> {
        // Get the bytecode which is in hex format with placeholders for library references.
        let mut bytecode = self.bytecode.object.clone();

        // Replace all library references with their address.
        // Here we differ slightly from the TypeScript version in that we don't return an error
        // for entries in the library address map that we end up not needing, so we can afford
        // to know less about which contract needs which exact references when we call them,
        for (lib_path, lib_name) in self.libraries_needed() {
            // References can be given with Fully Qualified Name, or just the contract name,
            // but they must be unique and unambiguous.
            let fqn = FullyQualifiedName::new(&lib_path, &lib_name);
            tracing::debug!("Requires library: {fqn}");

            let Some(lib_addr) = libraries.get(&lib_name) else {
                bail!("failed to resolve library: {fqn}")
            };

            let lib_addr = hex::encode(lib_addr.0);

            for pos in self.library_positions(&lib_path, &lib_name) {
                let start = 2 + pos.start * 2;
                let end = start + pos.length * 2;
                bytecode.replace_range(start..end, &lib_addr);
            }
        }

        let bytecode = hex::decode(bytecode.trim_start_matches("0x"))
            .context("failed to decode contract from hex")?;

        Ok(bytecode)
    }
}

/// Match the `"bytecode"` entry in the Hardhat build artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bytecode {
    /// Hexadecimal format with placeholders for links.
    pub object: String,
    pub link_references: HashMap<ContractSource, HashMap<ContractName, Vec<Position>>>,
}

/// Indicate where a placeholder appears in the bytecode object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub start: usize,
    pub length: usize,
}

/// Return elements of a dependency tree in topological order.
///
/// Note: Defines the deployment order, since only post-deployment we know the
/// contracts address, and hence can deploy contracts that use it.
fn topo_sort<T>(mut dependency_tree: DependencyTree<T>) -> Result<Vec<T>>
where
    T: Eq + PartialEq + Hash + Ord + Clone,
{
    let mut sorted = Vec::new();

    while !dependency_tree.is_empty() {
        let Some((leaf, _)) = dependency_tree.iter().find(|(_, set)| set.is_empty()) else {
            bail!("circular reference in the dependencies")
        };
        let leaf = leaf.clone();

        dependency_tree.remove(&leaf);

        for (_, children) in dependency_tree.iter_mut() {
            children.remove(&leaf);
        }

        sorted.push(leaf);
    }

    Ok(sorted)
}
