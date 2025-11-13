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
    arguments: &SubnetCreateConfig,
) -> anyhow::Result<FvmAddress> {
    let parent = SubnetID::from_str(&arguments.parent)?;

    let from = match &arguments.from {
        Some(address) => Some(require_fil_addr_from_str(address)?),
        None => None,
    };

    let supply_source = parse_supply_source(arguments)?;
    let collateral_source = parse_collateral_source(arguments)?;

    let raw_addr = arguments
        .validator_gater
        .clone()
        .unwrap_or(ZERO_ADDRESS.to_string());
    let validator_gater = require_fil_addr_from_str(&raw_addr)?;

    let raw_addr = arguments
        .validator_rewarder
        .clone()
        .unwrap_or(ZERO_ADDRESS.to_string());
    let validator_rewarder = require_fil_addr_from_str(&raw_addr)?;

    // Fetch F3 instance ID if parent is Filecoin (for deterministic genesis)
    //
    // When --parent-filecoin-rpc is provided, we fetch the current F3 instance ID
    // and store it in the subnet actor. This ensures all nodes generate identical
    // genesis files by fetching F3 data for the SAME instance, not "latest".
    //
    // Without this, nodes running genesis at different times would fetch different
    // F3 instances, resulting in different genesis files and consensus failure.
    let genesis_f3_instance_id = if let Some(ref parent_filecoin_rpc) =
        arguments.parent_filecoin_rpc
    {
        match fetch_current_f3_instance(
            parent_filecoin_rpc,
            arguments.parent_filecoin_auth_token.as_ref(),
        )
        .await
        {
            Ok(instance_id) => {
                log::info!(
                    "Captured F3 instance ID {} for deterministic genesis",
                    instance_id
                );
                Some(instance_id)
            }
            Err(e) => {
                log::warn!(
                    "Failed to fetch F3 instance ID: {}. Subnet will be created without F3 data.",
                    e
                );
                None
            }
        }
    } else {
        log::debug!("Parent Filecoin RPC not provided - parent is likely another subnet (no F3)");
        None
    };

    let addr = provider
        .create_subnet(
            from,
            parent,
            arguments.min_validators,
            f64_to_token_amount(arguments.min_validator_stake)?,
            arguments.bottomup_check_period,
            arguments
                .active_validators_limit
                .unwrap_or(DEFAULT_ACTIVE_VALIDATORS),
            f64_to_token_amount(arguments.min_cross_msg_fee)?,
            arguments.permission_mode,
            supply_source,
            collateral_source,
            validator_gater,
            validator_rewarder,
            arguments.genesis_subnet_ipc_contracts_owner,
            arguments.chain_id,
            genesis_f3_instance_id,
        )
        .await?;

    Ok(addr)
}

/// Fetches the current F3 instance ID from Filecoin parent chain
///
/// This captures the F3 instance ID at subnet creation time and stores it in the
/// subnet actor. All nodes will later fetch this SAME instance ID when generating
/// genesis, ensuring deterministic genesis files across all nodes.
///
/// # Arguments
/// * `parent_filecoin_rpc` - Filecoin RPC endpoint (mainnet or calibration)
/// * `auth_token` - Optional auth token for the RPC endpoint
///
/// # Returns
/// The current F3 instance ID (extracted from the latest certificate)
async fn fetch_current_f3_instance(
    parent_filecoin_rpc: &url::Url,
    auth_token: Option<&String>,
) -> anyhow::Result<u64> {
    use ipc_provider::jsonrpc::JsonRpcClientImpl;
    use ipc_provider::lotus::client::LotusJsonRPCClient;
    use ipc_provider::lotus::LotusClient;

    let jsonrpc_client =
        JsonRpcClientImpl::new(parent_filecoin_rpc.clone(), auth_token.map(|s| s.as_str()));

    let lotus_client = LotusJsonRPCClient::new(jsonrpc_client, SubnetID::default());

    // Fetch the latest F3 certificate which contains the current instance ID
    let cert = lotus_client.f3_get_certificate().await?;

    match cert {
        Some(c) => {
            // Extract instance ID from the certificate (gpbft_instance field)
            Ok(c.gpbft_instance)
        }
        None => Err(anyhow::anyhow!(
            "No F3 certificate available on parent chain"
        )),
    }
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

    #[arg(
        long,
        help = "The chain id for the subnet, make sure it's unique across existing known chain ids"
    )]
    pub chain_id: u64,

    /// Parent Filecoin RPC endpoint (optional - only when parent is Filecoin)
    /// If provided, CLI will fetch F3 instance ID for deterministic genesis
    #[arg(
        long,
        help = "Parent Filecoin RPC endpoint (for F3 instance ID capture)"
    )]
    pub parent_filecoin_rpc: Option<url::Url>,

    /// Auth token for parent Filecoin RPC (optional)
    #[arg(long, help = "Auth token for parent Filecoin RPC")]
    pub parent_filecoin_auth_token: Option<String>,
}

#[derive(Debug, Args)]
#[command(name = "create", about = "Create a new subnet actor")]
pub struct CreateSubnetArgs {
    #[command(flatten)]
    pub config: SubnetCreateConfig,
}
