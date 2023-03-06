// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet cli command handler.

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;
use url::Url;

use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::{CreateSubnetParams, CreateSubnetResponse};

/// The command to create a new subnet actor.
pub(crate) struct CreateSubnet;

#[async_trait]
impl CommandLineHandler for CreateSubnet {
    type Arguments = CreateSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("launching json rpc server with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = CreateSubnetParams {
            parent: arguments.parent.clone(),
            name: arguments.name.clone(),
            min_validator_stake: arguments.min_validator_stake,
            min_validators: arguments.min_validators,
            finality_threshold: arguments.finality_threshold,
            check_period: arguments.check_period,
        };

        let address = json_rpc_client
            .request::<CreateSubnetResponse>(
                json_rpc_methods::CREATE_SUBNET,
                serde_json::to_value(params)?,
            )
            .await?
            .address;

        log::info!("created subent actor with address: {address:}");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Create a new subnet actor")]
pub(crate) struct CreateSubnetArgs {
    #[arg(help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(help = "The parent subnet to create the new actor in")]
    pub parent: String,
    #[arg(help = "The name of the subnet")]
    pub name: String,
    #[arg(help = "The minimal validator stake amount")]
    pub min_validator_stake: u64,
    #[arg(help = "The minimal number of validators")]
    pub min_validators: u64,
    #[arg(help = "The finality threshold for MIR")]
    pub finality_threshold: ChainEpoch,
    #[arg(help = "The checkpoint period")]
    pub check_period: ChainEpoch,
}

fn get_ipc_agent_url(ipc_agent_url: &Option<String>, global: &GlobalArguments) -> Result<Url> {
    let url = match ipc_agent_url {
        Some(url) => url.parse()?,
        None => {
            let config = global.config()?;
            let addr = config.server.json_rpc_address.to_string();
            // We are resolving back to our own ipc-agent node.
            // Since it's our own node, we will use http since we
            // should be in the same network.
            format!("http://{addr:}").parse()?
        }
    };
    Ok(url)
}
