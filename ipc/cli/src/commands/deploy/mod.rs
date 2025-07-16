// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::anyhow;
use async_trait::async_trait;
use clap::{Args, ValueEnum};
use ethers::types::Address;
use fendermint_eth_deployer::{DeployedContracts, EthContractDeployer, SubnetCreationPrivilege};
use fendermint_eth_hardhat::Hardhat;
use ipc_provider::new_evm_keystore_from_arc_config;
use ipc_wallet::EvmKeyStore;
use ipc_wallet::{EthKeyAddress, PersistentKeyStore};
use serde::{self, Deserialize};
use std::path::PathBuf;
use std::{fmt, sync::Arc};

use crate::{CommandLineHandler, GlobalArguments};

/// A CLI-friendly wrapper for `SubnetCreationPrivilege`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliSubnetCreationPrivilege {
    Unrestricted,
    Owner,
}

impl Default for CliSubnetCreationPrivilege {
    fn default() -> Self {
        CliSubnetCreationPrivilege::Unrestricted
    }
}

impl fmt::Display for CliSubnetCreationPrivilege {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliSubnetCreationPrivilege::Unrestricted => write!(f, "Unrestricted"),
            CliSubnetCreationPrivilege::Owner => write!(f, "Owner"),
        }
    }
}

impl From<CliSubnetCreationPrivilege> for SubnetCreationPrivilege {
    fn from(cli: CliSubnetCreationPrivilege) -> Self {
        match cli {
            CliSubnetCreationPrivilege::Unrestricted => SubnetCreationPrivilege::Unrestricted,
            CliSubnetCreationPrivilege::Owner => SubnetCreationPrivilege::Owner,
        }
    }
}
/// The command to deploy the IPC contracts.
pub(crate) struct DeployCommand;

#[async_trait]
impl CommandLineHandler for DeployCommand {
    type Arguments = DeployCommandArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> anyhow::Result<()> {
        // Load configuration and create the keystore.
        let config = Arc::new(global.config()?);
        let keystore = new_evm_keystore_from_arc_config(config)?;
        let deployed_contracts = deploy_contracts(keystore, &args.config).await?;

        println!("{:=^60}", " Deployed Contracts ");
        println!("Registry: {:?}", deployed_contracts.registry);
        println!("Gateway : {:?}", deployed_contracts.gateway);
        println!("{:=^60}", "");

        Ok(())
    }
}

pub(crate) async fn deploy_contracts(
    key_store: PersistentKeyStore<EthKeyAddress>,
    config: &DeployConfig,
) -> anyhow::Result<DeployedContracts> {
    // Retrieve the key info for the provided "from" address.
    let key_info: ipc_wallet::EvmKeyInfo =
        key_store.get(&config.from.into())?.ok_or_else(|| {
            anyhow!(
                "address {} does not have a private key in key store",
                config.from
            )
        })?;

    let (artifacts_path, _maybe_tempdir) = match &config.artifacts_path {
        Some(path) => (path.clone(), None),
        None => {
            let (temp_dir, path) = contracts_artifacts::extract_to_tempdir()?;
            (path, Some(temp_dir))
        }
    };

    let hardhat = Hardhat::new(artifacts_path);

    let mut deployer = EthContractDeployer::new(
        hardhat,
        &config.url,
        key_info.private_key(),
        config.chain_id,
    )?;

    let deployed_contracts = deployer
        .deploy_all(config.subnet_creation_privilege.into())
        .await?;

    Ok(deployed_contracts)
}

/// Shared deploy config for both CLI flags and YAML.
///
/// - Clap will pick up each `#[arg(...)]` for `ipc-cli deploy`
/// - Serde will map kebab-case YAML keys to the same fields
#[derive(Debug, Clone, Args, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DeployConfig {
    /// The URL of the Ethereum provider.
    #[arg(short, long, help = "Ethereum provider URL")]
    pub url: String,

    /// Submitter address, must be in the keystore.
    #[arg(short, long, help = "Submitter address for contract deployment")]
    pub from: Address,

    /// The chain ID of the target network.
    #[arg(short = 'i', long, help = "Ethereum chain ID")]
    pub chain_id: u64,

    /// Directory containing the compiled contract files.
    #[arg(
        short,
        long,
        help = "Directory containing the contract files. Defaults to builtin contracts."
    )]
    pub artifacts_path: Option<PathBuf>,

    /// Subnet creation privilege (Unrestricted | Whitelisted | Restricted).
    #[arg(
        short = 'p',
        long,
        value_enum,
        default_value_t = CliSubnetCreationPrivilege::Unrestricted,
        help = "Subnet creation privilege"
    )]
    #[serde(default)]
    pub subnet_creation_privilege: CliSubnetCreationPrivilege,
}

/// Command-line arguments for deploying the IPC contracts.
///
/// All fields are mandatory.
#[derive(Debug, Args)]
#[command(about = "Deploy the IPC contracts")]
pub(crate) struct DeployCommandArgs {
    #[command(flatten)]
    pub config: DeployConfig,
}
