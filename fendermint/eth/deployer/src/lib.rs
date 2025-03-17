// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use ethers::abi::Tokenize;
use ethers::contract::ContractFactory;
use ethers::core::types as eth_types;
use ethers::prelude::*;
use k256::ecdsa::SigningKey;

use fendermint_eth_hardhat::{ContractSourceAndName, Hardhat, FQN};
use fendermint_vm_actor_interface::diamond::{EthContract, EthContractMap};
use fendermint_vm_actor_interface::ipc;
use fendermint_vm_actor_interface::ipc::IPC_CONTRACTS;
use fendermint_vm_genesis::ipc::GatewayParams;
use ipc_actors_abis::i_diamond::FacetCut;

/// Helper to construct a contract source path (e.g. "MyContract.sol").
pub fn contract_src(name: &str) -> PathBuf {
    PathBuf::from(format!("{name}.sol"))
}

/// Collects library and top-level contracts.
pub fn collect_contracts(
    hardhat: &Hardhat,
) -> Result<(Vec<ContractSourceAndName>, EthContractMap)> {
    let mut all_contracts = Vec::new();
    let mut top_level_contracts = EthContractMap::default();

    // Populate top-level contracts from predefined IPC contracts.
    top_level_contracts.extend(IPC_CONTRACTS.clone());

    // Collect contract names from top-level contracts and their facets.
    all_contracts.extend(top_level_contracts.keys());
    all_contracts.extend(
        top_level_contracts
            .values()
            .flat_map(|c| c.facets.iter().map(|f| f.name)),
    );

    // Get dependencies, but only keep library contracts.
    let mut eth_libs = hardhat
        .dependencies(
            &all_contracts
                .iter()
                .map(|n| (contract_src(n), *n))
                .collect::<Vec<_>>(),
        )
        .context("failed to collect EVM contract dependencies")?;
    eth_libs.retain(|(_, d)| !top_level_contracts.contains_key(d.as_str()));
    Ok((eth_libs, top_level_contracts))
}

/// Deploys Ethereum contracts and libraries.
pub struct EthContractDeployer {
    hardhat: Hardhat,
    ipc_contracts: Vec<ContractSourceAndName>,
    top_contracts: EthContractMap,
    lib_addrs: HashMap<FQN, eth_types::Address>,
    provider: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    chain_id: u64,
}

impl EthContractDeployer {
    /// Creates a new deployer instance.
    pub fn new(hardhat: Hardhat, url: &str, private_key: &str, chain_id: u64) -> Result<Self> {
        let provider = Provider::<Http>::try_from(url).context("failed to create HTTP provider")?;
        let wallet: LocalWallet = private_key.parse().context("invalid private key")?;
        let wallet = wallet.with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, wallet);

        let (ipc_contracts, top_contracts) =
            collect_contracts(&hardhat).context("failed to collect contracts")?;

        Ok(Self {
            hardhat,
            ipc_contracts,
            top_contracts,
            lib_addrs: HashMap::new(),
            provider: Arc::new(client),
            chain_id,
        })
    }

    /// Deploys a library contract.
    ///
    /// Reads the library artifact, replaces placeholders with the correct addresses,
    /// deploys the library, and records its address.
    async fn deploy_library(&mut self, lib_src: impl AsRef<Path>, lib_name: &str) -> Result<()> {
        let fqn = self.hardhat.fqn(lib_src.as_ref(), lib_name);
        tracing::info!("Deploying library {lib_name}");

        let artifact = self
            .hardhat
            .prepare_deployment_artifact(&lib_src, lib_name, &self.lib_addrs)
            .with_context(|| format!("failed to load library bytecode for {fqn}"))?;

        let factory = ContractFactory::new(
            artifact.abi.into(),
            artifact.bytecode.into(),
            self.provider.clone(),
        );
        let contract = factory.deploy(())?.send().await?;
        let eth_addr = contract.address();
        tracing::info!(?eth_addr, lib_name, "Library deployed successfully");

        self.lib_addrs.insert(fqn, eth_addr);
        Ok(())
    }

    /// Deploys a top-level contract with the given constructor parameters.
    ///
    /// The function prepares the artifact, creates a contract factory, deploys the contract,
    /// and returns its Ethereum address.
    async fn deploy_contract<T>(
        &self,
        contract_name: &str,
        constructor_params: T,
    ) -> Result<eth_types::Address>
    where
        T: Tokenize,
    {
        let src = contract_src(contract_name);
        tracing::info!("Deploying top-level contract {contract_name}");

        let artifact = self
            .hardhat
            .prepare_deployment_artifact(&src, contract_name, &self.lib_addrs)
            .with_context(|| format!("failed to load {contract_name} bytecode"))?;

        let factory = ContractFactory::new(
            artifact.abi.into(),
            artifact.bytecode.into(),
            self.provider.clone(),
        );
        let contract = factory.deploy(constructor_params)?.send().await?;
        let eth_addr = contract.address();
        tracing::info!(?eth_addr, contract_name, "Contract deployed successfully");

        Ok(eth_addr)
    }

    async fn deploy_gateway(&self) -> Result<eth_types::Address> {
        use ipc::gateway::{
            ConstructorParameters as GatewayConstructor, CONTRACT_NAME as GATEWAY_NAME,
        };
        use ipc_api::subnet_id::SubnetID;

        let ipc_params = GatewayParams::new(SubnetID::new(self.chain_id, vec![]));
        let params = GatewayConstructor::new(ipc_params, vec![])
            .context("failed to create gateway constructor parameters")?;

        let facets = self
            .facets(GATEWAY_NAME)
            .context("failed to collect gateway facets")?;

        self.deploy_contract(GATEWAY_NAME, (facets, params))
            .await
            .context("failed to deploy gateway contract")
    }

    async fn deploy_registry(
        &self,
        gateway_addr: eth_types::Address,
    ) -> Result<eth_types::Address> {
        use ipc::registry::{
            ConstructorParameters as RegistryConstructor, CONTRACT_NAME as REGISTRY_NAME,
        };

        let mut facets = self
            .facets(REGISTRY_NAME)
            .context("failed to collect registry facets")?;

        // Extract facets based on expected order.
        let getter_facet = facets.remove(0);
        let manager_facet = facets.remove(0);
        let rewarder_facet = facets.remove(0);
        let checkpointer_facet = facets.remove(0);
        let pauser_facet = facets.remove(0);
        let diamond_loupe_facet = facets.remove(0);
        let diamond_cut_facet = facets.remove(0);
        let ownership_facet = facets.remove(0);
        let activity_facet = facets.remove(0);

        debug_assert_eq!(facets.len(), 2, "SubnetRegistry should have 2 extra facets");

        let params = RegistryConstructor {
            gateway: gateway_addr,
            getter_facet: getter_facet.facet_address,
            manager_facet: manager_facet.facet_address,
            rewarder_facet: rewarder_facet.facet_address,
            pauser_facet: pauser_facet.facet_address,
            checkpointer_facet: checkpointer_facet.facet_address,
            diamond_cut_facet: diamond_cut_facet.facet_address,
            diamond_loupe_facet: diamond_loupe_facet.facet_address,
            ownership_facet: ownership_facet.facet_address,
            activity_facet: activity_facet.facet_address,
            subnet_getter_selectors: getter_facet.function_selectors,
            subnet_manager_selectors: manager_facet.function_selectors,
            subnet_rewarder_selectors: rewarder_facet.function_selectors,
            subnet_checkpointer_selectors: checkpointer_facet.function_selectors,
            subnet_pauser_selectors: pauser_facet.function_selectors,
            subnet_actor_diamond_cut_selectors: diamond_cut_facet.function_selectors,
            subnet_actor_diamond_loupe_selectors: diamond_loupe_facet.function_selectors,
            subnet_actor_ownership_selectors: ownership_facet.function_selectors,
            subnet_actor_activity_selectors: activity_facet.function_selectors,
            creation_privileges: 0, // 0: Unrestricted, 1: Owner.
        };

        self.deploy_contract(REGISTRY_NAME, (facets, params))
            .await
            .context("failed to deploy registry contract")
    }

    /// Deploys all contracts: first libraries, then top-level contracts.
    ///
    /// Deploys library contracts (which update the internal address map) and then deploys
    /// the gateway and registry contracts.
    pub async fn deploy(&mut self) -> Result<()> {
        // Deploy all required libraries.
        for (lib_src, lib_name) in self.ipc_contracts.clone() {
            self.deploy_library(lib_src, &lib_name)
                .await
                .with_context(|| format!("failed to deploy library {lib_name}"))?;
        }

        // Deploy the IPC Gateway contract.
        let gateway_addr = self.deploy_gateway().await?;

        // Deploy the IPC SubnetRegistry contract.
        self.deploy_registry(gateway_addr).await?;

        Ok(())
    }

    /// Collects facet cuts for the diamond pattern based on the deployed library addresses.
    fn facets(&self, contract_name: &str) -> Result<Vec<FacetCut>> {
        let contract = self.top_contract(contract_name)?;
        let mut facet_cuts = Vec::new();

        for facet in contract.facets.iter() {
            let facet_name = facet.name;
            let src = contract_src(facet_name);
            let facet_fqn = self.hardhat.fqn(&src, facet_name);

            let facet_addr = self
                .lib_addrs
                .get(&facet_fqn)
                .ok_or_else(|| anyhow!("facet {facet_name} has not been deployed"))?;

            let function_selectors = facet
                .abi
                .functions()
                .filter(|f| f.signature() != "init(bytes)")
                .map(|f| f.short_signature())
                .collect();

            facet_cuts.push(FacetCut {
                facet_address: *facet_addr,
                action: 0, // 0 indicates an "Add" operation.
                function_selectors,
            });
        }

        Ok(facet_cuts)
    }

    /// Retrieves a top-level contract by name.
    fn top_contract(&self, contract_name: &str) -> Result<&EthContract> {
        self.top_contracts
            .get(contract_name)
            .ok_or_else(|| anyhow!("unknown top contract name: {contract_name}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::EthContractDeployer;
    use fendermint_eth_hardhat::Hardhat;
    use std::path::PathBuf;
    use tracing_subscriber;

    #[tokio::test]
    async fn deploy_contracts_test() {
        // Initialize tracing subscriber to print logs during tests.
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let hardhat = Hardhat::new(PathBuf::from("/Users/karlem/work/ipc/contracts/out"));
        let mut deployer = EthContractDeployer::new(
            hardhat,
            "http://localhost:8545",
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            31337,
        )
        .expect("failed to create deployer");

        deployer.deploy().await.expect("deployment failed");
    }
}
