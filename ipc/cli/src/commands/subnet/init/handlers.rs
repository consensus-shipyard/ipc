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
    let created_genesis = if let Some(act_cfg) = &init_cfg.activate {
        activate_subnet(act_cfg, &created, &mut provider).await?;
        if let Some(gen_cfg) = &init_cfg.genesis {
            let dir = global.config_dir();
            let parent = ipc_config_store
                .get_subnet(&created.parent_subnet_id)
                .await
                .context("parent subnet not found in config store")?;

            let genesis = create_genesis(gen_cfg, &created.subnet_id, &parent, &dir).await?;
            Some((genesis, gen_cfg))
        } else {
            None
        }
    } else {
        log::info!("No activation configured; skipping activation and genesis");
        None
    };

    // 5) Generate additional files
    if let Some((genesis, gen_cfg)) = &created_genesis {
        // Use the global config directory
        let dir = global.config_dir();

        let node_config_path = generate_node_config(
            &created.subnet_id,
            &created.parent_subnet_id,
            &genesis.sealed,
            init_cfg.activate.as_ref(),
            &dir,
        )
        .await?;

        log::info!(
            "Node configuration ready for customization at: {}",
            node_config_path.display()
        );
        log::info!(
            "Run 'ipc node init --config {}' to initialize your node",
            node_config_path.display()
        );

        let subnet_info_path = generate_subnet_info(
            &created.subnet_id,
            &created.parent_subnet_id,
            &ipc_config_store.snapshot().await,
            genesis,
            gen_cfg,
            init_cfg.activate.as_ref(),
            &dir,
        )
        .await?;

        log::info!(
            "Subnet information saved to: {}",
            subnet_info_path.display()
        );
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

/// Generate a basic node.yaml configuration with sensible defaults
pub async fn generate_node_config(
    subnet_id: &SubnetID,
    parent_id: &SubnetID,
    genesis_path: &std::path::Path,
    activation_info: Option<&ActivateConfig>,
    output_dir: &std::path::Path,
) -> anyhow::Result<std::path::PathBuf> {
    use crate::commands::node::config::{GenesisSource, NodeInitConfig};
    use crate::commands::subnet::init::config::JoinConfig;
    use crate::commands::wallet::import::WalletImportArgs;

    let safe_id = sanitize_subnet_id(subnet_id);
    let node_config_path = output_dir.join(format!("node_{}.yaml", safe_id));

    // Determine if this is a collateral-based subnet
    let is_collateral = matches!(activation_info, Some(ActivateConfig::Collateral { .. }));

    // Create join config if it's a collateral subnet
    let join_config = if is_collateral {
        Some(JoinConfig {
            from: "YOUR_VALIDATOR_ADDRESS".to_string(), // Placeholder for user to fill
            collateral: 1.0,                            // Default collateral amount
            initial_balance: Some(10.0),                // Default initial balance
        })
    } else {
        None
    };

    // Determine genesis source based on activation status
    let genesis_source = if activation_info.is_some() {
        // Subnet is activated - use existing genesis file
        // genesis_path actually contains the sealed genesis path, so we need to derive the genesis path
        let sealed_path = genesis_path.to_path_buf();

        // Derive the genesis path from the sealed path by replacing "genesis_sealed_" with "genesis_"
        let genesis_path = if let Some(file_name) = sealed_path.file_name() {
            if let Some(file_str) = file_name.to_str() {
                if file_str.starts_with("genesis_sealed_") {
                    let genesis_name = file_str.replace("genesis_sealed_", "genesis_");
                    sealed_path.with_file_name(genesis_name)
                } else {
                    // Fallback: assume the sealed path is correct
                    sealed_path.clone()
                }
            } else {
                sealed_path.clone()
            }
        } else {
            sealed_path.clone()
        };

        GenesisSource::Path(crate::commands::subnet::create_genesis::CreatedGenesis {
            genesis: genesis_path,
            sealed: sealed_path,
        })
    } else {
        // Subnet is NOT activated - create new genesis
        GenesisSource::Create(crate::commands::subnet::create_genesis::GenesisConfig {
            network_version: fvm_shared::version::NetworkVersion::V21,
            base_fee: fvm_shared::econ::TokenAmount::from_atto(1000),
            power_scale: 3,
        })
    };

    // Create basic node config with sensible defaults
    let node_config = NodeInitConfig {
        home: "~/.node-ipc".into(),
        subnet: subnet_id.to_string(),
        parent: parent_id.to_string(),
        genesis: genesis_source,
        key: WalletImportArgs {
            wallet_type: "evm".to_string(),
            path: None,
            private_key: None, // Will generate a new key
        },
        join: join_config,
        p2p: Some(crate::commands::node::config::P2pConfig {
            external_ip: Some("127.0.0.1".to_string()), // Default external IP for user to modify
            ports: None,                                // Let user configure ports
            peers: None,                                // Let user configure peers
        }),
        cometbft_overrides: None,
        fendermint_overrides: None,
    };

    // Serialize NodeInitConfig to YAML
    let yaml_content =
        serde_yaml::to_string(&node_config).context("failed to serialize node config to YAML")?;

    // Write to file
    tokio::fs::write(&node_config_path, yaml_content)
        .await
        .context("failed to write node config file")?;

    log::info!(
        "Node configuration generated at: {}",
        node_config_path.display()
    );

    Ok(node_config_path)
}

/// Generate subnet information JSON file
pub async fn generate_subnet_info(
    subnet_id: &SubnetID,
    parent_id: &SubnetID,
    ipc_config: &ipc_provider::config::Config,
    created_genesis: &crate::commands::subnet::create_genesis::CreatedGenesis,
    genesis_config: &crate::commands::subnet::create_genesis::GenesisConfig,
    activation_info: Option<&ActivateConfig>,
    output_dir: &std::path::Path,
) -> anyhow::Result<std::path::PathBuf> {
    use crate::commands::subnet::init::info::{
        SubnetActivationInfo, SubnetContractInfo, SubnetGeneralInfo, SubnetGenesisInfo, SubnetInfo,
    };

    let safe_id = sanitize_subnet_id(subnet_id);
    let subnet_info_path = output_dir.join(format!("subnet-{}.json", safe_id));

    // Get subnet and parent info from config
    let subnet = ipc_config
        .subnets
        .get(subnet_id)
        .context("subnet not found in config")?;
    let parent = ipc_config
        .subnets
        .get(parent_id)
        .context("parent subnet not found in config")?;

    // Create subnet info structure
    let subnet_info = SubnetInfo {
        subnet_info: SubnetGeneralInfo {
            subnet_id: subnet_id.to_string(),
            parent_id: parent_id.to_string(),
            name: None, // Could be added to config if needed
            created_at: chrono::Utc::now().to_rfc3339(),
            network: "testnet".to_string(), // TODO: Get from config when available
        },
        contracts: SubnetContractInfo {
            gateway_address: subnet.gateway_addr().to_string(),
            registry_address: subnet.registry_addr().to_string(),
            parent_gateway: parent.gateway_addr().to_string(),
            parent_registry: parent.registry_addr().to_string(),
        },
        genesis: SubnetGenesisInfo {
            genesis_path: created_genesis.genesis.to_string_lossy().to_string(),
            sealed_genesis_path: created_genesis.sealed.to_string_lossy().to_string(),
            network_version: genesis_config.network_version.into(),
            base_fee: genesis_config.base_fee.to_string(),
            power_scale: genesis_config.power_scale,
        },
        activation: activation_info.map(|act| SubnetActivationInfo {
            mode: match act {
                ActivateConfig::Federated { .. } => "federated".to_string(),
                ActivateConfig::Static { .. } => "static".to_string(),
                ActivateConfig::Collateral { .. } => "collateral".to_string(),
            },
            validators: match act {
                ActivateConfig::Federated { power } | ActivateConfig::Static { power } => {
                    power.validator_pubkeys.clone()
                }
                ActivateConfig::Collateral { validators } => {
                    validators.iter().map(|v| v.from.clone()).collect()
                }
            },
            stakes: match act {
                ActivateConfig::Collateral { validators } => Some(
                    validators
                        .iter()
                        .map(|v| v.collateral.to_string())
                        .collect(),
                ),
                _ => None,
            },
        }),
    };

    // Serialize to JSON with pretty printing
    let json_content = serde_json::to_string_pretty(&subnet_info)
        .context("failed to serialize subnet info to JSON")?;

    // Write to file
    tokio::fs::write(&subnet_info_path, json_content)
        .await
        .context("failed to write subnet info file")?;

    log::info!(
        "Subnet information saved to: {}",
        subnet_info_path.display()
    );

    // Print subnet information to console
    print_subnet_info_to_console(&subnet_info);

    Ok(subnet_info_path)
}

/// Print subnet information to console with clear formatting
fn print_subnet_info_to_console(subnet_info: &crate::commands::subnet::init::info::SubnetInfo) {
    println!("\n🌐 Subnet Information Generated:");
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    📋 Subnet Details                        │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ Subnet ID:    {}         │",
        subnet_info.subnet_info.subnet_id
    );
    println!(
        "│ Parent ID:    {}         │",
        subnet_info.subnet_info.parent_id
    );
    println!(
        "│ Network:      {}         │",
        subnet_info.subnet_info.network
    );
    println!(
        "│ Created:      {}         │",
        subnet_info.subnet_info.created_at
    );
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│                   🔧 Contract Addresses                    │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ Gateway:      {}         │",
        subnet_info.contracts.gateway_address
    );
    println!(
        "│ Registry:     {}         │",
        subnet_info.contracts.registry_address
    );
    println!(
        "│ Parent GW:    {}         │",
        subnet_info.contracts.parent_gateway
    );
    println!(
        "│ Parent Reg:   {}         │",
        subnet_info.contracts.parent_registry
    );
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│                   📜 Genesis Information                    │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!(
        "│ Genesis:      {}         │",
        subnet_info.genesis.genesis_path
    );
    println!(
        "│ Sealed:       {}         │",
        subnet_info.genesis.sealed_genesis_path
    );
    println!(
        "│ Version:      {}         │",
        subnet_info.genesis.network_version
    );
    println!("│ Base Fee:     {}         │", subnet_info.genesis.base_fee);
    println!(
        "│ Power Scale:  {}         │",
        subnet_info.genesis.power_scale
    );

    if let Some(activation) = &subnet_info.activation {
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│                  ⚡ Activation Details                     │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ Mode:        {}         │", activation.mode);
        if !activation.validators.is_empty() {
            println!("│ Validators:  {}         │", activation.validators.len());
            for validator in &activation.validators {
                println!("│   - {}         │", validator);
            }
        }
    }

    println!("└─────────────────────────────────────────────────────────────┘");
    println!(
        "📁 Subnet info saved to: subnet-{}.json",
        subnet_info.subnet_info.subnet_id.replace("/", "_")
    );
    println!(
        "📄 Node config saved to: node_{}.yaml",
        subnet_info.subnet_info.subnet_id.replace("/", "_")
    );
    println!();
}

/// Sanitize subnet ID for use in filenames
fn sanitize_subnet_id(id: &SubnetID) -> String {
    id.to_string().replace("/", "_").replace(":", "_")
}
