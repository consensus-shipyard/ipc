// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Utilities for deploying Ethereum contracts and libraries.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use ethers::core::types as eth_types;
use fendermint_eth_hardhat::{
    as_contract_name, ContractName,
};
use fendermint_vm_actor_interface::diamond::EthContractMap;
use ipc_actors_abis::i_diamond::FacetCut;

/// Returns the contract source path for a given contract name (e.g. "MyContract.sol").
pub fn contract_src(name: &str) -> PathBuf {
    PathBuf::from(format!("{name}.sol"))
}

/// Collects facet cuts for the diamond pattern for a specified top-level contract.
///
/// Lookup the given contract name.
pub fn collect_facets(
    contract_name: &str,
    top_contracts: &EthContractMap,
    lib_addrs: &HashMap<ContractName, eth_types::Address>,
) -> Result<Vec<FacetCut>> {
    let contract = top_contracts
        .get(contract_name)
        .ok_or_else(|| anyhow!("unknown top contract name: {contract_name}"))?;

    let facet_cuts = Result::<Vec<FacetCut>>::from_iter(contract.facets.iter().map(|facet| {
        let contract_name = as_contract_name(&facet.name);
        let facet_addr = lib_addrs
            .get(&contract_name)
            .ok_or_else(|| anyhow!("facet {} has not been deployed", facet.name))?;
        let selectors = Vec::from_iter(
            facet
                .abi
                .functions()
                .filter(|f| f.signature() != "init(bytes)")
                .map(|f| f.short_signature()),
        );

        Ok(FacetCut {
            facet_address: *facet_addr,
            action: 0, // 0 indicates an "Add" operation.
            function_selectors: selectors,
        })
    }))?;

    Ok(facet_cuts)
}
