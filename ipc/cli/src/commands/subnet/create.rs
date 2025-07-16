// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet cli command handler.

use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address as FvmAddress;
use fvm_shared::clock::ChainEpoch;

use ethers::types::Address as EthAddress;

use ipc_api::subnet::{Asset, AssetKind, PermissionMode};
use ipc_api::subnet_id::SubnetID;

use crate::commands::get_ipc_provider;
use crate::commands::subnet::ZERO_ADDRESS;
use crate::{f64_to_token_amount, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};
use serde::{self, Deserialize};

const DEFAULT_ACTIVE_VALIDATORS: u16 = 100;

/// The command to create a new subnet actor.
pub struct CreateSubnet;

impl CreateSubnet {
    pub async fn create(
        global: &GlobalArguments,
        args: &CreateSubnetArgs,
    ) -> anyhow::Result<String> {
        let provider = get_ipc_provider(global)?;
        let created_subnet_address = create_subnet(provider, &args.config).await?;
        Ok(created_subnet_address.to_string())
    }
}

fn parse_supply_source(conf: &SubnetCreateConfig) -> anyhow::Result<Asset> {
    let token_address = if let Some(addr) = &conf.supply_source_address {
        Some(require_fil_addr_from_str(addr)?)
    } else {
        None
    };
    Ok(Asset {
        kind: conf.supply_source_kind,
        token_address,
    })
}

fn parse_collateral_source(conf: &SubnetCreateConfig) -> anyhow::Result<Asset> {
    let Some(ref kind) = conf.collateral_source_kind else {
        return Ok(Asset::default());
    };

    let token_address = if let Some(addr) = &conf.collateral_source_address {
        Some(require_fil_addr_from_str(addr)?)
    } else {
        None
    };

    Ok(Asset {
        kind: *kind,
        token_address,
    })
}

#[async_trait]
impl CommandLineHandler for CreateSubnet {
    type Arguments = CreateSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create subnet with args: {:?}", arguments);

        let address = CreateSubnet::create(global, arguments).await?;

        log::info!(
            "created subnet actor with id: {}/{}",
            arguments.config.parent,
            address
        );

        Ok(())
    }
}

pub(crate) async fn create_subnet(
    mut provider: ipc_provider::IpcProvider,
    config: &SubnetCreateConfig,
) -> anyhow::Result<FvmAddress> {
    let parent = SubnetID::from_str(&config.parent)?;

    let from = match &config.from {
        Some(address) => Some(require_fil_addr_from_str(address)?),
        None => None,
    };

    let supply_source = parse_supply_source(config)?;
    let collateral_source = parse_collateral_source(config)?;

    let raw_addr = config
        .validator_gater
        .clone()
        .unwrap_or(ZERO_ADDRESS.to_string());
    let validator_gater = require_fil_addr_from_str(&raw_addr)?;

    let raw_addr = config
        .validator_rewarder
        .clone()
        .unwrap_or(ZERO_ADDRESS.to_string());
    let validator_rewarder = require_fil_addr_from_str(&raw_addr)?;
    let addr = provider
        .create_subnet(
            from,
            parent,
            config.min_validators,
            f64_to_token_amount(config.min_validator_stake)?,
            config.bottomup_check_period,
            config
                .active_validators_limit
                .unwrap_or(DEFAULT_ACTIVE_VALIDATORS),
            f64_to_token_amount(config.min_cross_msg_fee)?,
            config.permission_mode,
            supply_source,
            collateral_source,
            validator_gater,
            validator_rewarder,
            config.genesis_subnet_ipc_contracts_owner,
        )
        .await?;

    Ok(addr)
}

/// Shared subnet‐create config for both CLI flags and YAML.
///
/// - Clap will pick up each `#[arg(long, help=...)]`
/// - Serde will map kebab-case YAML keys to the same fields
#[derive(Debug, Clone, Args, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SubnetCreateConfig {
    /// The address that creates the subnet (defaults to your global sender).
    #[arg(long, help = "The address that creates the subnet")]
    pub from: Option<String>,

    /// The parent subnet namespace (e.g. `/r314159/...`).
    #[arg(long, help = "The parent subnet to create the new actor in")]
    pub parent: String,

    /// The minimum FIL stake required per validator (in whole FIL).
    #[arg(
        long,
        help = "Minimum collateral per validator (whole FIL; min 1 nanoFIL)"
    )]
    pub min_validator_stake: f64,

    /// Minimum number of validators required to bootstrap the subnet.
    #[arg(long, help = "Minimum number of validators to bootstrap the subnet")]
    pub min_validators: u64,

    /// The bottom-up checkpoint period (in number of epochs).
    #[arg(long, help = "Bottom-up checkpoint period in epoch count")]
    pub bottomup_check_period: ChainEpoch,

    /// Maximum number of active validators in the subnet.
    #[arg(long, help = "Max number of active validators in subnet")]
    pub active_validators_limit: Option<u16>,

    /// Minimum fee for cross-network messages (FIL).
    #[arg(
        long,
        default_value = "0.000001",
        help = "Min fee for cross-net messages (whole FIL; min 1 nanoFIL)"
    )]
    pub min_cross_msg_fee: f64,

    /// The permission mode: collateral, federated, or static.
    #[arg(
        long,
        value_enum,
        help = "Permission mode for the subnet: collateral, federated, or static"
    )]
    pub permission_mode: PermissionMode,

    /// Source of new tokens: native or erc20.
    #[arg(
        long,
        value_enum,
        help = "Kind of supply source on parent: native or erc20"
    )]
    pub supply_source_kind: AssetKind,

    /// ERC-20 contract address (if `supply-source-kind` == `erc20`).
    #[arg(long, help = "Address of supply source on parent (omit if native)")]
    pub supply_source_address: Option<String>,

    /// Validator gating contract address (optional).
    #[arg(long, help = "Validator gating contract address, if any")]
    pub validator_gater: Option<String>,

    /// Validator rewarder contract address.
    #[arg(long, help = "Validator rewarder contract address")]
    pub validator_rewarder: Option<String>,

    /// Collateral source kind: native or erc20.
    #[arg(
        long,
        value_enum,
        help = "Kind of collateral source on parent: native or erc20"
    )]
    pub collateral_source_kind: Option<AssetKind>,

    /// ERC-20 collateral contract (if `collateral-source-kind` == `erc20`).
    #[arg(long, help = "Collateral source address on parent (omit if native)")]
    pub collateral_source_address: Option<String>,

    /// Owner for all IPC diamond contracts at genesis (subnet-local address).
    #[arg(long, help = "Genesis owner for IPC diamond contracts on this subnet")]
    pub genesis_subnet_ipc_contracts_owner: EthAddress,
}

#[derive(Debug, Args)]
#[command(name = "create", about = "Create a new subnet actor")]
pub struct CreateSubnetArgs {
    #[command(flatten)]
    pub config: SubnetCreateConfig,
}
