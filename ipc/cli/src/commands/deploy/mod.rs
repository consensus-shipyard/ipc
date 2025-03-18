// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use clap::{Args, ValueEnum};
use fendermint_eth_deployer::{EthContractDeployer, SubnetCreationPrivilege};
use fendermint_eth_hardhat::Hardhat;
use std::fmt;
use std::path::PathBuf;

use crate::{CommandLineHandler, GlobalArguments};

/// A CLI-friendly wrapper for `SubnetCreationPrivilege`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliSubnetCreationPrivilege {
    Unrestricted,
    Owner,
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

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> anyhow::Result<()> {
        let hardhat = Hardhat::new(args.contracts_dir.clone());
        let mut deployer =
            EthContractDeployer::new(hardhat, &args.url, &args.private_key, args.chain_id)?;
        let deployed_contracts = deployer
            .deploy_all(args.subnet_creation_privilege.into())
            .await?;

        println!("{:=^60}", " Deployed Contracts ");
        println!("Registry: {}", deployed_contracts.registry);
        println!("Gateway : {}", deployed_contracts.gateway);
        println!("{:=^60}", "");
        Ok(())
    }
}

/// Command-line arguments for deploying the IPC contracts.
///
/// All fields are mandatory.
#[derive(Debug, Args)]
#[command(about = "Deploy the IPC contracts")]
pub(crate) struct DeployCommandArgs {
    /// The URL of the Ethereum provider.
    #[arg(short, long, required = true)]
    url: String,

    /// The private key for signing transactions.
    #[arg(short, long, required = true)]
    private_key: String,

    /// The chain ID of the target network.
    #[arg(short, long, required = true)]
    chain_id: u64,

    /// Directory containing the contract files.
    #[arg(short, long, required = true)]
    contracts_dir: PathBuf,

    // Subnet creation privilege
    #[arg(short, long, value_enum, default_value_t = CliSubnetCreationPrivilege::Unrestricted)]
    subnet_creation_privilege: CliSubnetCreationPrivilege,
}
