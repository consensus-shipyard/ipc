// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Handler for the `ipc-cli subnet init` command.
//!
//! This module implements the "subnet init" workflow, which reads a YAML config
//! and executes each phase: deploy (optional), create on-chain, genesis pull/seal,
//! and emits metadata for downstream steps.

use crate::commands::deploy::{deploy_contracts, DeployConfig};
use crate::commands::subnet::approve::{approve_subnet, ApproveSubnetArgs};
use crate::commands::subnet::create::{create_subnet, SubnetCreateConfig};
use crate::commands::subnet::join::{join_subnet, JoinSubnetArgs};
use crate::commands::subnet::set_federated_power::{set_federated_power, SetFederatedPowerArgs};
use crate::{get_ipc_provider, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};
use anyhow::Context;
use async_trait::async_trait;
use clap::Args;
use fendermint_app::cmd::genesis::{new_genesis_from_parent, seal_genesis};
use fendermint_app::options::genesis::{GenesisFromParentArgs, SealGenesisArgs};
use fendermint_vm_actor_interface::init::builtin_actor_eth_addr;
use fendermint_vm_actor_interface::ipc::{self, subnet};
use fvm_shared::{address::Address, econ::TokenAmount, version::NetworkVersion};
use ipc_api::subnet::PermissionMode;
use ipc_api::subnet_id::SubnetID;
use ipc_provider::{
    config::{EVMSubnet, Subnet, SubnetConfig},
    new_evm_keystore_from_config,
};
use ipc_types::EthAddress;
use num_bigint::BigInt;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use ethers::types::H160;
use ethers::utils::keccak256;
use hex::FromHex;

/// What your user writes in YAML under `activation.federated`
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SetFederatedPowerConfig {
    pub validator_pubkeys: Vec<String>,
    pub validator_power: Vec<u64>,
}

impl SetFederatedPowerConfig {
    pub fn into_args(self, subnet: String, from: String) -> anyhow::Result<SetFederatedPowerArgs> {
        let validator_addresses = self
            .validator_pubkeys
            .iter()
            .map(|hex| {
                public_key_to_address(hex)
                    .map(|addr| format!("{:#x}", addr))
                    .map_err(|e| anyhow::anyhow!("invalid pubkey {}: {}", hex, e))
            })
            .collect::<anyhow::Result<_>>()?;

        let validator_pubkeys = self
            .validator_pubkeys
            .iter()
            .map(|s| {
                s.trim_start_matches("0x")
                    .trim_start_matches("0X")
                    .to_string()
            })
            .collect();

        let validator_power: Vec<u128> = self.validator_power.into_iter().map(u128::from).collect();

        Ok(SetFederatedPowerArgs {
            from,
            subnet,
            validator_addresses,
            validator_pubkeys,
            validator_power,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct JoinSubnetConfig {
    pub from: String,
    pub collateral: f64,
    pub initial_balance: Option<f64>,
}

impl JoinSubnetConfig {
    pub fn into_args(self, subnet: String) -> JoinSubnetArgs {
        JoinSubnetArgs {
            from: Some(self.from),
            subnet,
            collateral: self.collateral,
            initial_balance: self.initial_balance,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum ActivateConfig {
    Federated {
        #[serde(flatten)]
        config: SetFederatedPowerConfig,
    },

    Static {
        #[serde(flatten)]
        config: SetFederatedPowerConfig,
    },

    Collateral {
        validators: Vec<JoinSubnetConfig>,
    },
}

/// Top‑level YAML schema for `subnet init`.
#[derive(Debug, Deserialize)]
struct SubnetInitConfig {
    /// Optional on-chain contract deployment settings
    #[serde(default)]
    deploy: Option<DeployConfig>,
    /// Subnet creation settings
    create: SubnetCreateConfig,
    /// Optional subnet activation settings
    #[serde(default)]
    activate: Option<ActivateConfig>,
}

impl SubnetInitConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(act) = &self.activate {
            use ActivateConfig::*;
            match (self.create.permission_mode, act) {
                (PermissionMode::Federated, Federated { .. })
                | (PermissionMode::Static, Static { .. })
                | (PermissionMode::Collateral, Collateral { .. }) => {}

                (pm, _) => {
                    anyhow::bail!(
                        "activation.mode `{:?}` does not match create.permission_mode `{:?}`",
                        act,
                        pm
                    );
                }
            }
        }
        Ok(())
    }
}

/// Load and parse a YAML config file into `SubnetInitConfig`.
fn load_init_config<P: AsRef<Path>>(path: P) -> anyhow::Result<SubnetInitConfig> {
    let contents = fs::read_to_string(&path)?;
    let cfg = serde_yaml::from_str::<SubnetInitConfig>(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.as_ref().display(), e))?;
    cfg.validate()?;

    Ok(cfg)
}

fn public_key_to_address(pub_key_hex: &str) -> anyhow::Result<H160> {
    let key = pub_key_hex
        .strip_prefix("0x")
        .or_else(|| pub_key_hex.strip_prefix("0X"))
        .unwrap_or(pub_key_hex);

    let pub_key_bytes = Vec::<u8>::from_hex(key)
        .map_err(|e| anyhow::anyhow!("Invalid hex for public key: {}", e))?;

    if pub_key_bytes.len() != 65 || pub_key_bytes[0] != 0x04 {
        anyhow::bail!("Invalid uncompressed public key: expected 65 bytes starting with 0x04");
    }

    let hash = keccak256(&pub_key_bytes[1..]);

    Ok(H160::from_slice(&hash[12..]))
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
            let subnet_id = SubnetID::new_root(deploy.chain_id);

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

        let mut provider = get_ipc_provider(global)?;

        // TODO Karel - find a better way to handle this. It should ne be optional.
        // 1. Use different struct for config
        // 2. Validate in validate function??
        let from = match &init_config.create.from {
            Some(addr) => addr.clone(),
            None => {
                return Err(anyhow::anyhow!(
                    "create.from must be specified for federated subnet activation"
                ))
            }
        };

        let subnet_address = create_subnet(provider.clone(), &init_config.create).await?;
        log::info!("Created subnet actor with address: {}", subnet_address);

        let parent_subnet_id = SubnetID::from_str(&init_config.create.parent)?;

        let parent_subnet = {
            let subnet = ipc_provider_config
                .subnets
                .get(&parent_subnet_id)
                .context("subnet not found")?;
            subnet.clone()
        };

        let provider_http = parent_subnet.rpc_http().clone();

        let subnet_id = SubnetID::new_from_parent(&parent_subnet_id, subnet_address);
        Arc::get_mut(&mut ipc_provider_config)
            .expect("no other clones, so this Arc is unique")
            .add_subnet(Subnet {
                id: subnet_id.clone(),
                config: SubnetConfig::Fevm(EVMSubnet {
                    provider_http: provider_http.clone(),
                    provider_timeout: None,
                    auth_token: None,
                    gateway_addr: Address::from(builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID)),
                    registry_addr: Address::from(builtin_actor_eth_addr(
                        ipc::SUBNETREGISTRY_ACTOR_ID,
                    )),
                }),
            });

        // Approve subnet
        let approve_args = ApproveSubnetArgs {
            subnet: subnet_id.to_string(),
            from: Some(from.clone()),
        };
        approve_subnet(&mut provider, &approve_args).await?;

        if let Some(activation) = init_config.activate {
            log::info!("Starting activation for subnet `{}`", subnet_id);
            match activation {
                ActivateConfig::Federated { config } | ActivateConfig::Static { config } => {
                    log::info!(
                        "Performing {:?} activation (set-federated-power)…",
                        init_config.create.permission_mode
                    );

                    let args = config.into_args(subnet_id.to_string(), from)?;

                    set_federated_power(&provider, &args).await?;
                    log::info!(
                        "{:?} activation complete for subnet `{}`",
                        init_config.create.permission_mode,
                        subnet_id
                    );
                }
                ActivateConfig::Collateral { validators } => {
                    log::info!(
                        "Performing collateral activation for {} validators…",
                        validators.len()
                    );
                    for config in validators {
                        let from = config.from.clone();
                        log::info!(
                            "Joining subnet `{}` as `{}` with {} collateral",
                            subnet_id,
                            from,
                            config.collateral
                        );

                        let args = config.into_args(subnet_id.to_string());
                        join_subnet(&mut provider, &args).await?;

                        log::info!("Successfully joined subnet `{}` for `{}`", subnet_id, from);
                    }
                    log::info!("Collateral activation complete for subnet `{}`", subnet_id);
                }
            }
        } else {
            log::info!("No activation block found; skipping activation");
        }

        let dir = dir_of(&global.config_path());
        genesis_from_parent(&parent_subnet, &subnet_id, &dir).await?;

        Ok(())
    }
}

async fn genesis_from_parent(
    parent_subnet: &Subnet,
    subnet_id: &SubnetID,
    dir: &PathBuf,
) -> anyhow::Result<()> {
    let provider_http = parent_subnet.rpc_http().clone();

    log::info!(
        "Preparing genesis fetch from parent for subnet {}",
        parent_subnet.id
    );

    let genesis_file = dir.join("genesis.json");
    log::info!("Genesis file path: {}", genesis_file.display());
    log::info!(
        "Fetching and sealing genesis from parent endpoint {} for subnet {}",
        provider_http,
        subnet_id
    );
    new_genesis_from_parent(
        &genesis_file,
        &GenesisFromParentArgs {
            subnet_id: subnet_id.clone(),
            parent_endpoint: provider_http,
            parent_auth_token: parent_subnet.auth_token(),
            parent_gateway: parent_subnet.gateway_addr(),
            parent_registry: parent_subnet.registry_addr(),
            // TODO Karel - pass these from somewhere.
            network_version: NetworkVersion::from(21),
            base_fee: TokenAmount::from_atto(BigInt::from(1000)),
            power_scale: 3,
        },
    )
    .await?;
    log::info!("Genesis successfully created at {}", genesis_file.display());

    Ok(())
}

fn dir_of(config_path: &str) -> PathBuf {
    let p = Path::new(config_path);
    // `parent()` returns an `Option<&Path>`; if there’s no “parent” we fall back to “.”
    match p.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
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
