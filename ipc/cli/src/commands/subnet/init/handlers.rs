// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::deploy::deploy_contracts as deploy_contracts_cmd;
use crate::commands::subnet::approve::{approve_subnet as approve_subnet_cmd, ApproveSubnetArgs};
use crate::commands::subnet::create::create_subnet as create_subnet_cmd;
use crate::commands::subnet::create_genesis::create_genesis;
use crate::commands::subnet::join::join_subnet;
use crate::commands::subnet::set_federated_power::set_federated_power;
use crate::commands::wallet::import::import_wallet;
use crate::ipc_config_store::IpcConfigStore;

use crate::commands::subnet::init::config::{
    ActivateConfig, DeployConfig, SubnetCreateConfig, SubnetInitConfig, WalletImportArgs,
};
use crate::{get_ipc_provider, GlobalArguments};
use anyhow::{Context, Result};
use fendermint_vm_actor_interface::init::builtin_actor_eth_addr;
use fendermint_vm_actor_interface::ipc::{self};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::new_evm_keystore_from_config;
use ipc_provider::IpcProvider;
use ipc_types::EthAddress;
use std::str::FromStr;
use url::Url;

/// Orchestrates the end-to-end `subnet init` workflow.
pub async fn handle_init(global: &GlobalArguments, init_cfg: &SubnetInitConfig) -> Result<()> {
    // Load IPC config store
    let ipc_config_store = IpcConfigStore::load_or_init(global).await?;

    // 1) Optionally import wallets
    if let Some(wallets_cfg) = &init_cfg.import_wallets {
        let provider = get_ipc_provider(global)?;
        import_wallets(wallets_cfg, &provider)?;
    }

    // 2) Optionally deploy contracts
    if let Some(deploy_cfg) = &init_cfg.deploy {
        deploy_contracts(deploy_cfg, &ipc_config_store).await?;
    }

    let mut provider = get_ipc_provider(global)?;

    // 3) Create and approve subnet
    let created =
        create_and_approve_subnet(&init_cfg.create, &ipc_config_store, &mut provider).await?;

    // 4) Optionally activate and generate genesis
    if let Some(act_cfg) = &init_cfg.activate {
        activate_subnet(act_cfg, &created, &mut provider).await?;
        if let Some(gen_cfg) = &init_cfg.genesis {
            let dir = global.config_dir();
            let parent = ipc_config_store
                .get_subnet(&created.parent_subnet_id)
                .await
                .context("parent subnet not found in config store")?;

            create_genesis(gen_cfg, &created.subnet_id, &parent, &dir).await?;
        }
    } else {
        log::info!("No activation configured; skipping activation and genesis");
    }

    Ok(())
}

/// Deploys gateway & registry contracts, then records them.
async fn deploy_contracts(cfg: &DeployConfig, store: &IpcConfigStore) -> Result<()> {
    log::info!("Deploying contracts");

    let rpc_url: Url = cfg.url.parse().context("invalid RPC URL")?;
    let keystore = new_evm_keystore_from_config(&store.snapshot().await)?;
    let deployed = deploy_contracts_cmd(keystore, cfg).await?;
    log::info!("Deployed contracts: {:?}", deployed);

    let subnet_id = SubnetID::new_root(cfg.chain_id);
    store
        .add_subnet(
            subnet_id,
            rpc_url,
            EthAddress::from(deployed.gateway).into(),
            EthAddress::from(deployed.registry).into(),
        )
        .await?;

    Ok(())
}

/// Activates a subnet based on the configured mode.
async fn activate_subnet(
    cfg: &ActivateConfig,
    created: &CreatedSubnet,
    provider: &mut IpcProvider,
) -> Result<()> {
    log::info!("Activating subnet `{}`", created.subnet_id);

    match cfg {
        ActivateConfig::Federated { power } | ActivateConfig::Static { power } => {
            log::info!("Setting federated power");
            let args = power
                .clone()
                .into_args(created.subnet_id.to_string(), created.creator.clone())?;
            set_federated_power(provider, &args).await?;
            log::info!("Federated power set");
        }
        ActivateConfig::Collateral { validators } => {
            for v in validators {
                log::info!("Joining subnet `{}` as `{}`", created.subnet_id, v.from);
                let args = v.clone().into_args(created.subnet_id.to_string());
                join_subnet(provider, &args).await?;
                log::info!("Joined subnet `{}` as `{}`", created.subnet_id, v.from);
            }
        }
    }

    log::info!("Subnet `{}` activated", created.subnet_id);

    Ok(())
}

/// Represents results of a newly created subnet.
struct CreatedSubnet {
    subnet_id: SubnetID,
    parent_subnet_id: SubnetID,
    creator: String,
}

/// Creates and approves the subnet on-chain.
async fn create_and_approve_subnet(
    cfg: &SubnetCreateConfig,
    store: &IpcConfigStore,
    provider: &mut IpcProvider,
) -> Result<CreatedSubnet> {
    log::info!("Creating subnet");
    let actor_addr = create_subnet_cmd(provider.clone(), cfg).await?;

    let parent_id = SubnetID::from_str(&cfg.parent)?;
    let parent = store
        .get_subnet(&parent_id)
        .await
        .context("parent subnet not found in config store")?;

    let rpc_url = parent.rpc_http().clone();
    let subnet_id = SubnetID::new_from_parent(&parent_id, actor_addr);

    store
        .add_subnet(
            subnet_id.clone(),
            rpc_url,
            builtin_actor_eth_addr(ipc::GATEWAY_ACTOR_ID).into(),
            builtin_actor_eth_addr(ipc::SUBNETREGISTRY_ACTOR_ID).into(),
        )
        .await?;

    let creator = cfg.from.clone().context("subnet creator not specified")?;

    let approve_args = ApproveSubnetArgs {
        subnet: subnet_id.to_string(),
        from: Some(creator.clone()),
    };
    approve_subnet_cmd(provider, &approve_args).await?;

    Ok(CreatedSubnet {
        subnet_id,
        parent_subnet_id: parent_id,
        creator,
    })
}

/// Imports wallets into the IPC keystore
fn import_wallets(all_imports: &Vec<WalletImportArgs>, provider: &IpcProvider) -> Result<()> {
    log::info!("Importing wallets");
    for args in all_imports {
        let imported_wallet = import_wallet(provider, args).context("failed to import wallet")?;
        log::info!("Imported wallet: {}", imported_wallet.address);
    }
    log::info!("Wallets imported");
    Ok(())
}
