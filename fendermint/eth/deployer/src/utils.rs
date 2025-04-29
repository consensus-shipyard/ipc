// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Utilities for deploying Ethereum contracts and libraries.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use ethers::core::types as eth_types;
use fendermint_eth_hardhat::{
    fully_qualified_name, ContractSourceAndName, FullyQualifiedName, SolidityActorContracts,
};
use fendermint_vm_actor_interface::diamond::EthContractMap;
use fendermint_vm_actor_interface::ipc::IPC_CONTRACTS;
use ipc_actors_abis::i_diamond::FacetCut;

/// Returns the contract source path for a given contract name (e.g. "MyContract.sol").
pub fn contract_src(name: &str) -> PathBuf {
    PathBuf::from(format!("{name}.sol"))
}

/// Collects library and top-level contracts.
/// Returns a tuple containing a vector of library contracts (with their source paths)
/// and a map of top-level contracts.
pub fn collect_contracts(
    hardhat: &SolidityActorContracts,
) -> Result<(Vec<ContractSourceAndName>, EthContractMap)> {
    let mut all_contract_names = Vec::new();
    let top_level_contracts = IPC_CONTRACTS.clone();

    // Add top-level contract names and their facet names.
    all_contract_names.extend(top_level_contracts.keys().cloned());
    all_contract_names.extend(
        top_level_contracts
            .values()
            .flat_map(|c| c.facets.iter().map(|f| f.name)),
    );

    let contracts_with_paths = all_contract_names
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<String>>();

    let mut eth_libs = hardhat
        .dependencies(&contracts_with_paths)
        .context("failed to collect EVM contract dependencies")?;

    // Keep only library contracts (exclude top-level ones).
    eth_libs.retain(|(_, contract_name)| !top_level_contracts.contains_key(contract_name.as_str()));

    Ok((eth_libs, top_level_contracts))
}

/// Collects facet cuts for the diamond pattern for a specified top-level contract.
pub fn collect_facets(
    contract_name: &str,
    hardhat: &SolidityActorContracts,
    top_contracts: &EthContractMap,
    lib_addrs: &HashMap<FullyQualifiedName, eth_types::Address>,
) -> Result<Vec<FacetCut>> {
    let contract = top_contracts
        .get(contract_name)
        .ok_or_else(|| anyhow!("unknown top contract name: {contract_name}"))?;

    let facet_cuts = contract
        .facets
        .iter()
        .map(|facet| {
            let src = contract_src(&facet.name);
            let facet_fqn = fully_qualified_name(&src, &facet.name);
            let facet_addr = lib_addrs
                .get(&facet_fqn)
                .ok_or_else(|| anyhow!("facet {} has not been deployed", facet.name))?;
            let selectors = facet
                .abi
                .functions()
                .filter(|f| f.signature() != "init(bytes)")
                .map(|f| f.short_signature())
                .collect();

            Ok(FacetCut {
                facet_address: *facet_addr,
                action: 0, // 0 indicates an "Add" operation.
                function_selectors: selectors,
            })
        })
        .collect::<Result<Vec<FacetCut>>>()?;

    Ok(facet_cuts)
}
