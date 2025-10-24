// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Deployer for Ethereum contracts and libraries.

pub mod utils;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use ethers::abi::Tokenize;
use ethers::contract::ContractFactory;
use ethers::core::types as eth_types;
use ethers::prelude::*;
use fendermint_eth_hardhat::{ContractSourceAndName, DeploymentArtifact, Hardhat, FQN};
use fendermint_vm_actor_interface::diamond::EthContractMap;
use fendermint_vm_actor_interface::ipc;
use fendermint_vm_genesis::ipc::GatewayParams;
use ipc_actors_abis::i_diamond::FacetCut;
use ipc_provider::manager::evm::gas_estimator_middleware::Eip1559GasEstimatorMiddleware;
use k256::ecdsa::SigningKey;

use crate::utils::{collect_contracts, collect_facets, contract_src};

// 200 is used because some networks like the Calibration network and mainnet can be slow,
// and the transaction deployment can fail even though the transaction is mined.
const TRANSACTION_RECEIPT_RETRIES: usize = 200;

type SignerWithFeeEstimator =
    Arc<Eip1559GasEstimatorMiddleware<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>;

#[derive(Debug)]
pub struct DeployedContracts {
    pub registry: eth_types::Address,
    pub gateway: eth_types::Address,
}

#[repr(u8)]
pub enum SubnetCreationPrivilege {
    Unrestricted = 0,
    Owner = 1,
}
/// Responsible for deploying Ethereum contracts and libraries.
pub struct EthContractDeployer {
    hardhat: Hardhat,
    ipc_contracts: Vec<ContractSourceAndName>,
    top_contracts: EthContractMap,
    lib_addrs: HashMap<FQN, eth_types::Address>,
    provider: SignerWithFeeEstimator,
    chain_id: u64,
}

impl EthContractDeployer {
    /// Creates a new `EthContractDeployer` instance.
    pub fn new(hardhat: Hardhat, url: &str, private_key: &[u8], chain_id: u64) -> Result<Self> {
        let provider = Provider::<Http>::try_from(url).context("failed to create HTTP provider")?;
        let wallet: LocalWallet =
            LocalWallet::from_bytes(private_key).context("invalid private key")?;
        let wallet = wallet.with_chain_id(chain_id);
        let signer = SignerMiddleware::new(provider, wallet.clone());
        let client = Eip1559GasEstimatorMiddleware::new(signer);

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

    /// Deploys all contracts:
    /// first libraries, then the gateway and registry contracts.
    pub async fn deploy_all(
        &mut self,
        subnet_creation_privilege: SubnetCreationPrivilege,
    ) -> Result<DeployedContracts> {
        self.deploy_all_with_progress(
            subnet_creation_privilege,
            None::<fn(&str, &str, usize, usize)>,
        )
        .await
    }

    /// Deploys all contracts with progress reporting:
    /// first libraries, then the gateway and registry contracts.
    ///
    /// The progress callback receives:
    /// - contract_name: Name of the contract being deployed
    /// - contract_type: Type of contract ("library", "gateway", "registry")
    /// - current_step: Current step number (0-based)
    /// - total_steps: Total number of deployment steps
    pub async fn deploy_all_with_progress<F>(
        &mut self,
        subnet_creation_privilege: SubnetCreationPrivilege,
        progress_callback: Option<F>,
    ) -> Result<DeployedContracts>
    where
        F: Fn(&str, &str, usize, usize) + Send + Sync,
    {
        let total_steps = self.ipc_contracts.len() + 2; // libraries + gateway + registry
        let mut current_step = 0;

        // Deploy all required libraries with progress reporting.
        for (lib_src, lib_name) in self.ipc_contracts.clone() {
            if let Some(ref callback) = progress_callback {
                callback(&lib_name, "library", current_step, total_steps);
            }

            self.deploy_library(&lib_src, &lib_name)
                .await
                .with_context(|| format!("failed to deploy library {lib_name}"))?;

            current_step += 1;
        }

        // Deploy the IPC Gateway contract with progress reporting.
        if let Some(ref callback) = progress_callback {
            callback("Gateway", "gateway", current_step, total_steps);
        }
        let gateway_addr = self.deploy_gateway().await?;
        current_step += 1;

        // Deploy the IPC SubnetRegistry contract with progress reporting.
        if let Some(ref callback) = progress_callback {
            callback("Registry", "registry", current_step, total_steps);
        }
        let registry_addr = self
            .deploy_registry(gateway_addr, subnet_creation_privilege)
            .await?;

        Ok(DeployedContracts {
            registry: registry_addr,
            gateway: gateway_addr,
        })
    }

    /// Deploys a library contract.
    ///
    /// Reads the library artifact, substitutes placeholders with correct addresses,
    /// deploys the library, and records its address.
    async fn deploy_library(&mut self, lib_src: &Path, lib_name: &str) -> Result<()> {
        let fqn = self.hardhat.fqn(lib_src, lib_name);
        tracing::info!("Deploying library: {}", lib_name);

        let artifact = self
            .hardhat
            .prepare_deployment_artifact(lib_src, lib_name, &self.lib_addrs)
            .with_context(|| format!("failed to load library bytecode for {fqn}"))?;

        let address = self.deploy_artifact(artifact, ()).await?;

        tracing::info!(?address, "Library deployed successfully");
        self.lib_addrs.insert(fqn, address);
        Ok(())
    }

    /// Deploys a top-level contract with the given constructor parameters.
    async fn deploy_contract<T>(
        &self,
        contract_name: &str,
        constructor_params: T,
    ) -> Result<eth_types::Address>
    where
        T: Tokenize,
    {
        let src = contract_src(contract_name);
        tracing::info!("Deploying top-level contract: {}", contract_name);

        let artifact = self
            .hardhat
            .prepare_deployment_artifact(&src, contract_name, &self.lib_addrs)
            .with_context(|| format!("failed to load {contract_name} bytecode"))?;

        let address = self.deploy_artifact(artifact, constructor_params).await?;
        tracing::info!(?address, "Contract deployed successfully");

        Ok(address)
    }

    /// Deploys the provided deployment artifact with constructor parameters.
    async fn deploy_artifact<T>(
        &self,
        artifact: DeploymentArtifact,
        constructor_params: T,
    ) -> Result<eth_types::Address>
    where
        T: Tokenize,
    {
        let factory = ContractFactory::new(
            artifact.abi,
            artifact.bytecode.into(),
            self.provider.clone(),
        );

        let deployer = factory
            .deploy(constructor_params)
            .context("failed to create deployer")?;

        // Send the transaction and wait for the receipt.
        let pending_tx = deployer
            .client()
            .send_transaction(
                deployer.tx.clone(),
                Some(BlockId::Number(BlockNumber::Pending)),
            )
            .await?;

        tracing::info!(tx_hash = ?pending_tx.tx_hash(), "Transaction sent, awaiting confirmation");

        let receipt = pending_tx
            .confirmations(1)
            .retries(TRANSACTION_RECEIPT_RETRIES)
            .await?
            .ok_or_else(|| anyhow!("failed to get transaction receipt"))?;

        let address = receipt
            .contract_address
            .ok_or_else(|| anyhow!("transaction receipt missing contract address"))?;

        Ok(address)
    }

    /// Deploys the gateway contract.
    async fn deploy_gateway(&self) -> Result<eth_types::Address> {
        use ipc::gateway::{
            ConstructorParameters as GatewayConstructor, CONTRACT_NAME as GATEWAY_NAME,
        };
        use ipc_api::subnet_id::SubnetID;

        let ipc_params = GatewayParams::new(SubnetID::new(self.chain_id, vec![]));
        let commit_sha = get_commit_sha();
        let params = GatewayConstructor::new(ipc_params, vec![], commit_sha)
            .context("failed to create gateway constructor parameters")?;

        let facets = self
            .collect_facets(GATEWAY_NAME)
            .context("failed to collect gateway facets")?;

        self.deploy_contract(GATEWAY_NAME, (facets, params))
            .await
            .context("failed to deploy gateway contract")
    }

    /// Deploys the registry contract.
    async fn deploy_registry(
        &self,
        gateway_addr: eth_types::Address,
        subnet_creation_privilege: SubnetCreationPrivilege,
    ) -> Result<eth_types::Address> {
        use ipc::registry::{
            ConstructorParameters as RegistryConstructor, CONTRACT_NAME as REGISTRY_NAME,
        };

        let mut facets = self
            .collect_facets(REGISTRY_NAME)
            .context("failed to collect registry facets")?;

        // Ensure there are enough facets.
        if facets.len() < 9 {
            return Err(anyhow!(
                "expected at least 9 facets for registry contract, got {}",
                facets.len()
            ));
        }

        // Destructure the first 9 facets.
        let getter_facet = facets.remove(0);
        let manager_facet = facets.remove(0);
        let rewarder_facet = facets.remove(0);
        let checkpointer_facet = facets.remove(0);
        let pauser_facet = facets.remove(0);
        let diamond_loupe_facet = facets.remove(0);
        let diamond_cut_facet = facets.remove(0);
        let ownership_facet = facets.remove(0);
        let activity_facet = facets.remove(0);

        if facets.len() != 2 {
            return Err(anyhow!(
                "expected 2 extra facets for registry contract, got {}",
                facets.len()
            ));
        }

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
            creation_privileges: subnet_creation_privilege as u8,
        };

        self.deploy_contract(REGISTRY_NAME, (facets, params))
            .await
            .context("failed to deploy registry contract")
    }

    /// Collects facet cuts for the diamond pattern for a specified top-level contract.
    fn collect_facets(&self, contract_name: &str) -> Result<Vec<FacetCut>> {
        collect_facets(
            contract_name,
            &self.hardhat,
            &self.top_contracts,
            &self.lib_addrs,
        )
    }
}

/// Get the commit SHA for contract deployment.
/// Uses the COMMIT_SHA environment variable if set, otherwise uses git,
/// or falls back to a default value.
fn get_commit_sha() -> [u8; 32] {
    // Try to get from environment variable first (matches TypeScript deployment)
    if let Ok(sha) = std::env::var("COMMIT_SHA") {
        let mut result = [0u8; 32];
        let bytes = sha.as_bytes();
        let len = bytes.len().min(32);
        result[..len].copy_from_slice(&bytes[..len]);
        return result;
    }

    // Try to get from git
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mut result = [0u8; 32];
            let bytes = sha.as_bytes();
            let len = bytes.len().min(32);
            result[..len].copy_from_slice(&bytes[..len]);
            return result;
        }
    }

    // Fall back to default value (matches test default)
    let default_sha = b"c7d8f53f";
    let mut result = [0u8; 32];
    result[..default_sha.len()].copy_from_slice(default_sha);
    result
}
