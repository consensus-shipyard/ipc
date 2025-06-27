// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Handler for the `ipc-cli subnet init` command.
//!
//! This module implements the "subnet init" workflow, which reads a YAML config
//! and executes each phase: deploy (optional), create on-chain, genesis pull/seal,
//! and emits metadata for downstream steps.

use anyhow::Context;
use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_types::EthAddress;
use openssl::provider;
use serde::Deserialize;
use std::{fmt::Debug, fs, path::Path};

use crate::commands::deploy::{deploy_contracts, DeployConfig};
use crate::commands::subnet::create::{create_subnet, SubnetCreateConfig};
use crate::{get_ipc_provider, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::{
    config::{EVMSubnet, Subnet, SubnetConfig},
    new_evm_keystore_from_config,
};
use std::sync::Arc;

/// Top‑level YAML schema for `subnet init`.
#[derive(Debug, Deserialize)]
struct SubnetInitConfig {
    /// Optional on-chain contract deployment settings
    #[serde(default)]
    deploy: Option<DeployConfig>,

    create: SubnetCreateConfig,
}

/// Load and parse a YAML config file into `SubnetInitConfig`.
fn load_init_config<P: AsRef<Path>>(path: P) -> anyhow::Result<SubnetInitConfig> {
    let contents = fs::read_to_string(&path)?;
    let cfg = serde_yaml::from_str::<SubnetInitConfig>(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.as_ref().display(), e))?;
    Ok(cfg)
}

/// Struct representing the `subnet init` command.
///
/// This command will:
/// 1. Optionally deploy the gateway and registry contracts on the parent chain.
/// 2. Create the subnet on-chain via the `ipc-cli subnet create` step.
/// 3. Create genesis by pulling it from parent and sealing it.
/// 4. Emit a metadata file for downstream consumption.
pub struct InitSubnet;

#[async_trait]
impl CommandLineHandler for InitSubnet {
    type Arguments = InitSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::info!("Loading configuration from {}", arguments.config);
        let init_config = load_init_config(&arguments.config)?;
        log::debug!("Config loaded: {:?}", init_config);

        let ipc_provider_config = global.config()?;
        let mut ipc_provider_config = Arc::new(ipc_provider_config);

        if let Some(deploy) = &init_config.deploy {
            let provider_http = deploy.url.parse().context("invalid RPC URL")?;
            log::info!("Deploying contracts");

            let keystore = new_evm_keystore_from_config(ipc_provider_config.clone())?;
            let deployed_contracts = deploy_contracts(keystore, deploy).await?;

            log::info!("Deployed contracts: {:?}", deployed_contracts);
            let subnet_id = SubnetID::new(deploy.chain_id, vec![]);

            Arc::get_mut(&mut ipc_provider_config)
                .expect("no other clones, so this Arc is unique")
                .add_subnet(Subnet {
                    id: subnet_id,
                    config: SubnetConfig::Fevm(EVMSubnet {
                        provider_http,
                        provider_timeout: None,
                        auth_token: None,
                        gateway_addr: Address::from(EthAddress::from(deployed_contracts.gateway)),
                        registry_addr: Address::from(EthAddress::from(deployed_contracts.registry)),
                    }),
                });

            ipc_provider_config
                .write_to_file_async(global.config_path())
                .await?;
        }

        let provider = get_ipc_provider(global)?;
        let subnet_address = create_subnet(provider, &init_config.create).await?;
        log::info!("Created subnet actor with address: {}", subnet_address);

        // ipc_provider_config.add_subnet();

        Ok(())
    }
}

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
