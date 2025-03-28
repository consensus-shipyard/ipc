// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Provides a "linker" for artifacts created by `forge build` in the hardhat directory structure
//!
//! The hardhat directory structure is a folder `Foo.sol` with a contained `Foo.json`.
//! The JSON file encodes the `ABI` in field `abi` as well as input variables and their types.
//! There are more fields, such as `bytecode`, `deployedBytecode` and `linkReferences` - the latter can be referenced from within the `bytecode`.

use anyhow::{anyhow, bail, Context};
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
    top_level_contracts: EthContractMap,
    /// Eth libs that are not top level contracts
    eth_libs: EthContractMap,
}

impl SolidityActorContracts {
    /// Obtain the compiled top level contract
    pub fn get_lib(&self, fqn_contract_name: &str) -> Option {
        self.eth_libs.get(fqn_contract_name)
    }

    /// Obtain the compiled top level contract
    pub fn get_top_level(&self, fqn_contract_name: &str) -> Option {
        self.top_level_contracts.get(fqn_contract_name)
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
    contract_dir: PathBuf,
}

impl SolidityActorContractsLoader {
    /// Load the contracts from the hardhat format disk
    ///
    /// The output directory used with `forge build` the full-fat JSON files
    /// that contain ABI, bytecode, link references, etc.
    fn load_directory_inner(&self) -> Result<SolidityActorContracts> {
        fn contract_src(name: &str) -> PathBuf {
            PathBuf::from(format!("{name}.sol"))
        }

        let mut all_contracts = Vec::new();
        let mut top_level_contracts = EthContractMap::default();

        top_level_contracts.extend(IPC_CONTRACTS.clone());

        all_contracts.extend(top_level_contracts.keys());
        all_contracts.extend(
            top_level_contracts
                .values()
                .flat_map(|c| c.facets.iter().map(|f| f.name)),
        );
        // Collect dependencies of the main IPC actors.
        let mut eth_libs = Vec::<_>::from_iter(
            self.hardhat
                .dependencies(&all_contracts.iter().map(|n| (contract_src(n), *n))),
        )
        .context("failed to collect EVM contract dependencies")?;

        // Only keep library dependencies, not contracts with constructors.
        eth_libs.retain(|(_, d)| !top_level_contracts.contains_key(d.as_str()));

        Ok(SolidityActorContracts {
            top_level_contracts,
            eth_libs,
        })
    }

    /// A new loader
    pub fn load_directory(contract_dir: &Path) -> Result<SolidityActorContracts> {
        let loader = Self {
            contract_dir: contract_dir.to_path_buf(),
        };
        loader.load_directory_inner()
    }

    /// Fully qualified name of a source and contract.
    pub fn fully_qualified_name(&self, contract_path: &Path, contract_name: &str) -> String {
        format!("{}:{}", contract_path.to_string_lossy(), contract_name)
    }

    /// Construct bytecode from the compiled contract path
    ///
    /// Also replaces all linked libraries
    #[deprecated("Should decouple IO from logic ops")]
    pub fn bytecode(
        &self,
        contract_path: impl AsRef<Path>,
        contract_name: &str,
        libraries: &HashMap<FullyQualifiedName, et::Address>,
    ) -> anyhow::Result<Vec<u8>> {
        let contract_bytes = fs_err::read(contract_path.as_ref())?;
        self.resolve_library_references(&contract_bytes, contract_name, libraries)
    }

    /// Read the bytecode of the contract and replace all links in it with library addresses,
    /// similar to how the [hardhat-ethers](https://github.com/NomicFoundation/hardhat/blob/7cc06ab222be8db43265664c68416fdae3030418/packages/hardhat-ethers/src/internal/helpers.ts#L165C42-L165C42)
    /// plugin does it.
    ///
    /// The contract source is expected to be the logical path to a Solidity contract,
    /// including the extension, i.e. a [`ContractSource`].
    pub fn resolve_library_references(
        &self,
        contract_bytes: &[u8],
        contract_name: &str,
        libraries: &HashMap<FullyQualifiedName, et::Address>,
    ) -> anyhow::Result<Vec<u8>> {
        let artifact = self.artifact(contract_name, contract_src.as_ref())?;

        // Get the bytecode which is in hex format with placeholders for library references.
        let mut bytecode = artifact.bytecode.object.clone();

        // Replace all library references with their address.
        // Here we differ slightly from the TypeScript version in that we don't return an error
        // for entries in the library address map that we end up not needing, so we can afford
        // to know less about which contract needs which exact references when we call them,
        for (lib_path, lib_name) in artifact.libraries_needed() {
            // References can be given with Fully Qualified Name, or just the contract name,
            // but they must be unique and unambiguous.
            let fqn = self.fully_qualified_name(&lib_name, &lib_path);

            let lib_addr = match (libraries.get(&fqn), libraries.get(&lib_name)) {
                (None, None) => {
                    bail!("failed to resolve library: {fqn}")
                }
                (Some(_), Some(_)) => bail!("ambiguous library: {fqn}"),
                (Some(addr), None) => addr,
                (None, Some(addr)) => addr,
            };

            let lib_addr = hex::encode(lib_addr.0);

            for pos in artifact.library_positions(&lib_path, &lib_name) {
                let start = 2 + pos.start * 2;
                let end = start + pos.length * 2;
                bytecode.replace_range(start..end, &lib_addr);
            }
        }

        let bytecode = hex::decode(bytecode.trim_start_matches("0x"))
            .context("failed to decode contract from hex")?;

        Ok(DeploymentArtifact {
            bytecode,
            abi: artifact.abi.clone(),
        })
    }

    /// Traverse the linked references and return the library contracts to be deployed in topological order.
    ///
    /// The result will include the top contracts as well, and it's up to the caller to filter them out if
    /// they have more complicated deployments including constructors. This is because there can be diamond
    /// facets among them which aren't ABI visible dependencies but should be deployed as libraries.
    pub fn dependencies(
        &self,
        root_contracts: &[(impl AsRef<Path>, &str)],
    ) -> anyhow::Result<Vec<ContractSourceAndName>> {
        let mut deps: DependencyTree<ContractSourceAndName> = Default::default();

        let mut queue = root_contracts
            .iter()
            .map(|(s, c)| (PathBuf::from(s.as_ref()), c.to_string()))
            .collect::<VecDeque<_>>();

        // Construct dependency tree by recursive traversal.
        while let Some(sc) = queue.pop_front() {
            if deps.contains_key(&sc) {
                continue;
            }

            let artifact = self
                .artifact(&sc.0, &sc.1)
                .with_context(|| format!("failed to load dependency artifact: {}", sc.1))?;

            let cds = deps.entry(sc).or_default();

            for (ls, ln) in artifact.libraries_needed() {
                cds.insert((ls.clone(), ln.clone()));
                queue.push_back((ls, ln));
            }
        }

        // Topo-sort the libraries in the order of deployment.
        let sorted = topo_sort(deps)?;

        Ok(sorted)
    }

    /// Concatenate the contracts directory with the expected layout to get
    /// the path to the JSON file of a contract, which is under a directory
    /// named after the Solidity file.
    fn contract_path(&self, contract_src: &Path, contract_name: &str) -> anyhow::Result<PathBuf> {
        // There is currently no example of a Solidity directory containing multiple JSON files,
        // but it possible if there are multiple contracts in the file.

        let base_name = contract_src
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("failed to produce base name for {contract_src:?}"))?;

        let path = self
            .contracts_dir
            .join(base_name)
            .join(format!("{contract_name}.json"));

        Ok(path)
    }

    /// Parse the Hardhat artifact of a contract.
    fn artifact(&self, contract_name: &str, contract_src: &Path) -> anyhow::Result<Artifact> {
        let contract_path = self.contract_path(contract_src, contract_name)?;

        let json = fs::read_to_string(&contract_path)
            .with_context(|| format!("failed to read {contract_path:?}"))?;

        let artifact =
            serde_json::from_str::<Artifact>(&json).context("failed to parse Hardhat artifact")?;

        Ok(artifact)
    }
}

#[derive(Deserialize)]
struct Artifact {
    pub bytecode: Bytecode,
    pub abi: ethers_core::abi::Abi,
}

impl Artifact {
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
}

/// Match the `"bytecode"` entry in the Hardhat build artifact.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Bytecode {
    /// Hexadecimal format with placeholders for links.
    pub object: String,
    pub link_references: HashMap<ContractSource, HashMap<ContractName, Vec<Position>>>,
}

/// Indicate where a placeholder appears in the bytecode object.
#[derive(Deserialize)]
struct Position {
    pub start: usize,
    pub length: usize,
}

/// Return elements of a dependency tree in topological order.
fn topo_sort<T>(mut dependency_tree: DependencyTree<T>) -> anyhow::Result<Vec<T>>
where
    T: Eq + PartialEq + Hash + Ord + Clone,
{
    let mut sorted = Vec::new();

    while !dependency_tree.is_empty() {
        let leaf = match dependency_tree.iter().find(|(_, ds)| ds.is_empty()) {
            Some((k, _)) => k.clone(),
            None => bail!("circular reference in the dependencies"),
        };

        dependency_tree.remove(&leaf);

        for (_, ds) in dependency_tree.iter_mut() {
            ds.remove(&leaf);
        }

        sorted.push(leaf);
    }

    Ok(sorted)
}
