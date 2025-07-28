// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Generate and seal the genesis file for a subnet.

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use serde::Deserialize;

use fendermint_app::cmd::genesis::{new_genesis_from_parent, seal_genesis};
use fendermint_app::options::genesis::{GenesisFromParentArgs, SealGenesisArgs};
use fendermint_app::options::parse::{parse_network_version, parse_token_amount};

use fvm_shared::{econ::TokenAmount, version::NetworkVersion};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::config::Subnet;

use crate::{CommandLineHandler, Config, GlobalArguments};

/// Sanitize a SubnetID for use in filenames by stripping leading slashes
/// and replacing all other '/' with '_'.
fn sanitize_subnet_id(id: &SubnetID) -> String {
    let s = id.to_string();
    s.trim_start_matches('/').replace('/', "_")
}

/// Deserialize a string into `NetworkVersion`
fn de_network_version<'de, D>(deserializer: D) -> Result<NetworkVersion, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_network_version(&s).map_err(serde::de::Error::custom)
}

/// Deserialize a string into `TokenAmount`
fn de_token_amount<'de, D>(deserializer: D) -> Result<TokenAmount, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_token_amount(&s).map_err(serde::de::Error::custom)
}

/// Genesis parameters, configurable via CLI flags or YAML/JSON.
#[derive(Debug, Clone, Args, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GenesisConfig {
    /// Network version, governs which set of built-in actors to use.
    #[arg(long, short = 'v', default_value = "21", value_parser = parse_network_version)]
    #[serde(
        default = "GenesisConfig::default_network_version",
        deserialize_with = "de_network_version"
    )]
    pub network_version: NetworkVersion,

    /// Base fee for running transactions (in attoFIL).
    #[arg(long, short = 'f', default_value = "1000", value_parser = parse_token_amount)]
    #[serde(
        default = "GenesisConfig::default_base_fee",
        deserialize_with = "de_token_amount"
    )]
    pub base_fee: TokenAmount,

    /// Number of decimals to use when converting FIL to Power.
    #[arg(long, default_value = "3")]
    #[serde(default = "GenesisConfig::default_power_scale")]
    pub power_scale: i8,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        GenesisConfig {
            network_version: GenesisConfig::default_network_version(),
            base_fee: GenesisConfig::default_base_fee(),
            power_scale: GenesisConfig::default_power_scale(),
        }
    }
}

impl GenesisConfig {
    const fn default_network_version() -> NetworkVersion {
        NetworkVersion::V21
    }

    fn default_base_fee() -> TokenAmount {
        parse_token_amount("1000").unwrap()
    }

    const fn default_power_scale() -> i8 {
        3
    }
}

/// CLI arguments for `create-genesis`
#[derive(Debug, Clone, Args)]
pub struct CreateGenesisArgs {
    /// Subnet ID to generate genesis for (must be a key in `config.subnets`).
    #[arg(long, help = "Key of the child subnet in config")]
    pub subnet: String,

    /// Optional output directory; if omitted, uses the global config dir.
    #[arg(long, value_name = "DIR", help = "Output directory for genesis files")]
    pub out_dir: Option<PathBuf>,

    /// Genesis parameters: network-version, base-fee, power-scale.
    #[command(flatten)]
    pub config: GenesisConfig,
}

/// CLI handler for the `create-genesis` command.
pub struct CreateGenesis;

#[async_trait]
impl CommandLineHandler for CreateGenesis {
    type Arguments = CreateGenesisArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create genesis with args: {:?}", args);

        let ipc_config =
            Config::from_file(global.config_path()).context("loading network config file")?;

        let subnet_id = SubnetID::from_str(&args.subnet).context("invalid subnet SubnetID")?;

        let parent_id = subnet_id
            .parent()
            .context("parent is required for genesis")?;
        let parent = ipc_config
            .subnets
            .get(&parent_id)
            .cloned()
            .context("parent subnet not found in config store")?;

        let out_dir = args.out_dir.clone().unwrap_or_else(|| global.config_dir());

        create_genesis(&args.config, &subnet_id, &parent, &out_dir).await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct CreatedGenesis {
    pub genesis: PathBuf,
    pub sealed: PathBuf,
}

/// Generates and seals the genesis file for the subnet.
pub(crate) async fn create_genesis(
    cfg: &GenesisConfig,
    subnet_id: &SubnetID,
    parent: &Subnet,
    dir: &Path,
) -> Result<CreatedGenesis> {
    log::info!("Creating genesis");

    let safe_id = sanitize_subnet_id(subnet_id);
    let genesis_file = dir.join(format!("genesis_{}.json", safe_id));
    new_genesis_from_parent(
        &genesis_file,
        &GenesisFromParentArgs {
            subnet_id: subnet_id.clone(),
            parent_endpoint: parent.rpc_http().clone(),
            parent_auth_token: parent.auth_token(),
            parent_gateway: parent.gateway_addr(),
            parent_registry: parent.registry_addr(),
            network_version: cfg.network_version,
            base_fee: cfg.base_fee.clone(),
            power_scale: cfg.power_scale,
        },
    )
    .await?;
    log::info!("Genesis created at: {}", genesis_file.display());

    log::info!("Sealing genesis");
    let sealed = dir.join(format!("genesis_sealed_{}.json", safe_id));
    seal_genesis(
        &genesis_file,
        &SealGenesisArgs {
            output_path: sealed.clone(),
            custom_actors_path: None,
            builtin_actors_path: None,
            artifacts_path: None,
        },
    )
    .await?;
    log::info!("Genesis sealed and stored at: {}", sealed.display());

    Ok(CreatedGenesis {
        genesis: genesis_file,
        sealed,
    })
}
