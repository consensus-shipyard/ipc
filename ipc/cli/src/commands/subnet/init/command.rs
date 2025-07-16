// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::subnet::init::{config::SubnetInitConfig, handlers::handle_init};
use crate::{CommandLineHandler, GlobalArguments};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

/// CLI arguments for the `subnet init` command.
#[derive(Debug, Args)]
#[command(
    name = "init",
    about = "Bootstraps a new child subnet end-to-end from a YAML spec"
)]
pub struct InitSubnetArgs {
    /// Path to the subnet-init YAML configuration file
    #[arg(long, help = "Path to subnet init YAML config file")]
    pub config: String,
}

/// Handler for the `subnet init` command.
///
/// This command will:
/// 1. Optionally deploy gateway and registry contracts on the parent chain.
/// 2. Create the subnet on-chain via `ipc-cli subnet create`.
/// 3. Optionally activate the subnet on-chain.
/// 4. Generate and seal genesis from parent chain.
pub struct InitSubnet;

#[async_trait]
impl CommandLineHandler for InitSubnet {
    type Arguments = InitSubnetArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let init_config = SubnetInitConfig::load(&args.config)?;

        handle_init(global, &init_config).await?;

        Ok(())
    }
}
