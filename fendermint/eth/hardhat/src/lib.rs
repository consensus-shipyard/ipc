// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Provides a "linker" for artifacts created by `forge build` in the hardhat directory structure
//!
//! The hardhat directory structure is a folder `Foo.sol` with a contained `Foo.json`.
//! The JSON file encodes the `ABI` in field `abi` as well as input variables and their types.
//! There are more fields, such as `bytecode`, `deployedBytecode` and `linkReferences` - the latter can be referenced from within the `bytecode`.

use color_eyre::eyre::{self, bail, Context, ContextCompat, Result};
use core::iter::Iterator;
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
pub type ContractName = String;

pub type ContractSourceAndName = (ContractSource, ContractName);

/// Artifact from Hardhat build artifacts.
#[derive(Debug)]
pub struct DeploymentArtifact {
    pub bytecode: Vec<u8>,
    pub abi: ethers_core::abi::Abi,
}

/// Fully Qualified Name of a contract, e.g. `"src/lib/SubnetIDHelper.sol:SubnetIDHelper"`.
pub type FullyQualifiedName = String;

/// Dependency tree for libraries.
///
/// Using a [BTreeMap] for deterministic ordering.
type DependencyTree<T> = BTreeMap<T, HashSet<T>>;

/// Hold all compiled and linked bytes in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidityActorContracts {
    /// that contain ABI, bytecode, link references, etc. keyed by their name.
    top_level: HashMap<String, YetToLinkContractArtifact>,
    /// Eth libs that are not top level contracts
    libs: HashMap<String, YetToLinkContractArtifact>,
}

impl SolidityActorContracts {
    /// XXX
    pub fn collect_contracts(&self) -> Result<(Vec<ContractName>, EthContractMap)> {
        let mut all_contracts = Vec::new();
        let mut top_level_contracts = EthContractMap::default();

        top_level_contracts.extend(IPC_CONTRACTS.clone());

        all_contracts.extend(top_level_contracts.keys().map(|s| s.to_string()));
        all_contracts.extend(
            top_level_contracts
                .values()
                .flat_map(|contract| contract.facets.iter().map(|f| f.name.clone())),
        );
        // Collect dependencies of the main IPC actors.
        let mut eth_libs = self
            .dependencies(all_contracts.iter().map(|n| n.to_string()))
            .context("failed to collect EVM contract dependencies")?;

        // Only keep library dependencies, not contracts with constructors.
        eth_libs.retain(|d| !top_level_contracts.contains_key(d.as_str()));
        Ok((eth_libs, top_level_contracts))
    }

    /// TODO
    pub fn resolve_library_references(
        &self,
        contract_name_which: &str,
        library_addresses: &HashMap<FullyQualifiedName, et::Address>,
    ) -> eyre::Result<Vec<u8>> {
        let top_level_artifact = self
            .top_level
            .get(contract_name_which)
            .ok_or_else(|| eyre::eyre!("No such contract: {}", contract_name_which))?;
        top_level_artifact.resolve_library_references(library_addresses)
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
        let mut deps = DependencyTree::<ContractName>::default();

        let mut queue = VecDeque::<ContractName>::from_iter(
            root_contracts.into_iter().map(|name| name.to_string()),
        );

        // Construct dependency tree by recursive traversal.
        while let Some(name) = queue.pop_front() {
            if deps.contains_key(&name) {
                continue;
            }

            let artifact = self
                .get_lib(&name)
                .or_else(|| self.get_top_level(&name))
                .ok_or_else(|| eyre::eyre!("failed to load dependency artifact: {}", name,))?;

            let entry = deps.entry(name).or_default();

            for (path, lib_name) in artifact.libraries_needed() {
                entry.insert(lib_name.clone());
                queue.push_back(lib_name);
            }
        }

        // Topo-sort the libraries in the order of deployment.
        let sorted = topo_sort::<ContractName>(deps)?;

        Ok(sorted)
    }

    /// Obtain the compiled top level contract
    pub fn get_lib(&self, fqn_contract_name: &str) -> Option<&YetToLinkContractArtifact> {
        self.libs.get(fqn_contract_name)
    }

    /// Obtain the compiled top level contract
    pub fn get_top_level(&self, fqn_contract_name: &str) -> Option<&YetToLinkContractArtifact> {
        self.top_level.get(fqn_contract_name)
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

/// Fully qualified name of a source and contract.
pub fn fully_qualified_name(contract_path: &Path, contract_name: &str) -> String {
    format!("{}:{}", contract_path.to_string_lossy(), contract_name)
}

/// Utility to link bytecode from `forge build` process build artifacts.
#[derive(Clone, Debug)]
pub struct SolidityActorContractsLoader {
    contracts_dir: PathBuf,
}

impl SolidityActorContractsLoader {
    /// Load the contracts from the hardhat format disk
    ///
    /// The output directory used with `forge build` the full-fat JSON files
    /// that contain ABI, bytecode, link references, etc.
    ///
    /// Contract paths are derived from the name and the `self.contracts_dir`.
    fn load_directory_inner(&self) -> Result<SolidityActorContracts> {
        fn as_file_name(name: &str) -> PathBuf {
            PathBuf::from(format!("{name}.sol"))
        }

        let mut all_contracts = Vec::new();
        let mut top_level_contracts = EthContractMap::default();

        top_level_contracts.extend(IPC_CONTRACTS.clone());

        all_contracts.extend(top_level_contracts.keys().map(|&s| s.to_owned()));
        all_contracts.extend(
            top_level_contracts
                .values()
                .flat_map(|c| c.facets.iter().map(|f| f.name.clone())),
        );
        // Collect dependencies of the main IPC actors.
        let mut lib_deps = Vec::from_iter(
            self.dependencies(all_contracts.iter().map(|name| (as_file_name(name), name)))
                .context("failed to collect EVM contract dependencies")?,
        );

        // Only keep library dependencies, not contracts with constructors.
        lib_deps.retain(|(_, d): &(_, _)| !top_level_contracts.contains_key(d.as_str()));

        let top_level_artifacts =
            Result::<HashMap<FullyQualifiedName, YetToLinkContractArtifact>>::from_iter(
                top_level_contracts
                    .into_iter()
                    .map(|(name, eth): (&str, _)| {
                        let contract_path = as_file_name(name);
                        let fqn = fully_qualified_name(&contract_path, name);
                        let artifact: YetToLinkContractArtifact =
                            self.artifact(name, &contract_path)?;
                        Ok((fqn, artifact))
                    }),
            )?;

        let lib_artifacts =
            Result::<HashMap<FullyQualifiedName, YetToLinkContractArtifact>>::from_iter(
                lib_deps.into_iter().map(|(_path, name): (_, String)| {
                    let name = name.as_str();
                    let contract_path: PathBuf = as_file_name(name);
                    let fqn = fully_qualified_name(&contract_path, name);
                    let artifact = self.artifact(name, &contract_path)?;
                    Ok((fqn, artifact))
                }),
            )?;

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
        loader.load_directory_inner()
    }

    /// Construct bytecode from the compiled contract path
    ///
    /// Also replaces all linked libraries
    #[deprecated(note = "Should decouple IO from logic ops")]
    pub fn bytecode(
        &self,
        contract_path: impl AsRef<Path>,
        contract_name: &str,
        libraries: &HashMap<FullyQualifiedName, et::Address>,
    ) -> Result<Vec<u8>> {
        let artifact = self.artifact(contract_name, contract_path.as_ref())?;
        artifact.resolve_library_references(libraries)
    }

    #[deprecated(note = "Use `SolidityActorContracs::dependencies` instead, decouple IO")]
    /// Traverse the linked references and return the library contracts to be deployed in topological order.
    ///
    /// The result will include the top contracts as well, and it's up to the caller to filter them out if
    /// they have more complicated deployments including constructors. This is because there can be diamond
    /// facets among them which aren't ABI visible dependencies but should be deployed as libraries.
    pub fn dependencies<I: IntoIterator<Item = (A, S)>, S: ToString, A: AsRef<Path>>(
        &self,
        root_contracts: I,
    ) -> Result<Vec<ContractSourceAndName>> {
        let mut deps: DependencyTree<ContractSourceAndName> = Default::default();

        let mut queue = VecDeque::<(PathBuf, String)>::from_iter(
            root_contracts
                .into_iter()
                .map(|(s, c)| (PathBuf::from(s.as_ref()), c.to_string())),
        );

        // Construct dependency tree by recursive traversal.
        while let Some(sc) = queue.pop_front() {
            if deps.contains_key(&sc) {
                continue;
            }
            let (path, name) = &sc;

            let artifact = self.artifact(name, path).with_context(|| {
                format!(
                    "failed to load dependency artifact: {} from {}",
                    name,
                    path.display()
                )
            })?;

            let cds = deps.entry(sc).or_default();

            for (lib_path, lib_name) in artifact.libraries_needed() {
                cds.insert((lib_path.clone(), lib_name.clone()));
                queue.push_back((lib_path, lib_name));
            }
        }

        // Topo-sort the libraries in the order of deployment.
        let sorted = topo_sort(deps)?;

        Ok(sorted)
    }

    /// Concatenate the contracts directory with the expected layout to get
    /// the path to the JSON file of a contract, which is under a directory
    /// named after the Solidity file.
    fn contract_path(&self, contract_src: &Path, contract_name: &str) -> Result<PathBuf> {
        // There is currently no example of a Solidity directory containing multiple JSON files,
        // but it possible if there are multiple contracts in the file.

        let base_name = contract_src
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| eyre::eyre!("failed to produce base name for {contract_src:?}"))?;

        let path = self
            .contracts_dir
            .join(base_name)
            .join(contract_name)
            .with_extension("json");

        Ok(path)
    }

    /// Parse the Hardhat artifact of a contract.
    ///
    /// Note: Does the actual IO and parses from json to a rust type.
    fn artifact(
        &self,
        contract_name: &str,
        contract_src: &Path,
    ) -> Result<YetToLinkContractArtifact> {
        let contract_path = self.contract_path(contract_src, contract_name)?;

        let json = fs::read_to_string(&contract_path)
            .wrap_err_with(|| format!("failed to read {contract_path:?}"))?;

        let artifact = serde_json::from_str::<YetToLinkContractArtifact>(&json)
            .wrap_err("failed to parse Hardhat artifact")?;

        Ok(artifact)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct YetToLinkContractArtifact {
    pub bytecode: Bytecode,
    pub abi: ethers_core::abi::Abi,
}

use std::fmt;

impl fmt::Debug for YetToLinkContractArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "deps [ ")?;
        for (_, libname) in self.libraries_needed() {
            write!(f, "{}, ", libname)?;
        }
        write!(f, "]")?;

        let n = self.bytecode.object.len();
        write!(f, "{n} bytes [ ")?;
        let s = self.bytecode.object.as_bytes();
        let k = n.saturating_sub(10);
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
        libraries: &HashMap<FullyQualifiedName, et::Address>,
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
            let fqn = fully_qualified_name(&lib_path, &lib_name);

            let lib_addr = match (libraries.get(&fqn), libraries.get(&lib_name)) {
                (None, None) => {
                    bail!("failed to resolve library: {fqn}")
                }
                (Some(addr1), Some(addr2)) if addr1 == addr2 => addr1,
                (Some(_), Some(_)) => bail!("ambiguous library: {fqn}"),
                (Some(addr), None) => addr,
                (None, Some(addr)) => addr,
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
struct Bytecode {
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
fn topo_sort<T>(mut dependency_tree: DependencyTree<T>) -> Result<Vec<T>>
where
    T: Eq + PartialEq + Hash + Ord + Clone,
{
    let mut sorted = Vec::new();

    while !dependency_tree.is_empty() {
        let Some((leaf, _)) = dependency_tree.iter().find(|(_, ds)| ds.is_empty()) else {
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
