// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::get_subnet_config;
use crate::{CommandLineHandler, GlobalArguments};
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_identity::EvmKeyStore;
use ipc_provider::checkpoint::BottomUpCheckpointManager;
use ipc_provider::new_evm_keystore_from_path;
use ipc_sdk::subnet_id::SubnetID;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

const DEFAULT_POLLING_INTERVAL: u64 = 15;

/// The command to run the bottom up relayer in the background.
pub(crate) struct BottomUpRelayer;

#[async_trait]
impl CommandLineHandler for BottomUpRelayer {
    type Arguments = BottomUpRelayerArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("start bottom up relayer with args: {:?}", arguments);

        let config_path = global.config_path();

        let mut keystore = new_evm_keystore_from_path(&config_path)?;
        let submitter = match (arguments.submitter.as_ref(), keystore.get_default()?) {
            (Some(submitter), _) => Address::from_str(submitter)?,
            (None, Some(addr)) => {
                log::info!("using default address: {addr:?}");
                Address::try_from(addr)?
            }
            _ => {
                return Err(anyhow!("no submitter address provided"));
            }
        };

        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let parent = subnet
            .parent()
            .ok_or_else(|| anyhow!("root does not have parent"))?;

        let child = get_subnet_config(&config_path, &subnet)?;
        let parent = get_subnet_config(&config_path, &parent)?;

        let manager = BottomUpCheckpointManager::new_evm_manager(
            parent.clone(),
            child.clone(),
            Arc::new(RwLock::new(keystore)),
        )
        .await?;

        let interval = Duration::from_secs(
            arguments
                .checkpoint_interval_sec
                .unwrap_or(DEFAULT_POLLING_INTERVAL),
        );
        manager.run(submitter, interval).await;

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Start the bottom up relayer daemon")]
pub(crate) struct BottomUpRelayerArgs {
    #[arg(long, short, help = "The subnet id of the checkpointing subnet")]
    pub subnet: String,
    #[arg(long, short, help = "The number of seconds to submit checkpoint")]
    pub checkpoint_interval_sec: Option<u64>,
    #[arg(long, short, help = "The hex encoded address of the submitter")]
    pub submitter: Option<String>,
}
