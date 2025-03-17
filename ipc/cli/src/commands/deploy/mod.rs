// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use clap::Args;
use fendermint_eth_deployer::EthContractDeployer;
use fendermint_eth_hardhat::Hardhat;
use std::path::PathBuf;

use crate::{CommandLineHandler, GlobalArguments};

/// The command to deploy the IPC contracts.
pub(crate) struct DeployCommand;

#[async_trait]
impl CommandLineHandler for DeployCommand {
    type Arguments = DeployCommandArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> anyhow::Result<()> {
        let hardhat = Hardhat::new(args.contracts_dir.clone());
        let mut deployer =
            EthContractDeployer::new(hardhat, &args.url, &args.private_key, args.chain_id)?;
        deployer.deploy().await?;
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
}
