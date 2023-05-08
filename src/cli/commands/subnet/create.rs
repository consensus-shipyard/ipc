// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::create::{CreateSubnetParams, CreateSubnetResponse};

/// The command to create a new subnet actor.
pub struct CreateSubnet;

impl CreateSubnet {
    pub async fn create(
        global: &GlobalArguments,
        arguments: &CreateSubnetArgs,
    ) -> anyhow::Result<String> {
        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = CreateSubnetParams {
            from: arguments.from.clone(),
            parent: arguments.parent.clone(),
            name: arguments.name.clone(),
            min_validator_stake: arguments.min_validator_stake,
            min_validators: arguments.min_validators,
            bottomup_check_period: arguments.bottomup_check_period,
            topdown_check_period: arguments.topdown_check_period,
        };

        Ok(json_rpc_client
            .request::<CreateSubnetResponse>(
                json_rpc_methods::CREATE_SUBNET,
                serde_json::to_value(params)?,
            )
            .await?
            .address)
    }
}

#[async_trait]
impl CommandLineHandler for CreateSubnet {
    type Arguments = CreateSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create subnet with args: {:?}", arguments);

        let address = CreateSubnet::create(global, arguments).await?;

        log::info!(
            "created subnet actor with id: {}/{}",
            arguments.parent,
            address
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "create", about = "Create a new subnet actor")]
pub struct CreateSubnetArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that creates the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The parent subnet to create the new actor in")]
    pub parent: String,
    #[arg(long, short, help = "The name of the subnet")]
    pub name: String,
    #[arg(long, help = "The minimal validator stake amount (in whole FIL units)")]
    pub min_validator_stake: f64,
    #[arg(long, help = "The minimal number of validators")]
    pub min_validators: u64,
    #[arg(long, help = "The bottom up checkpoint period in number of blocks")]
    pub bottomup_check_period: ChainEpoch,
    #[arg(long, help = "The top down checkpoint period in number of blocks")]
    pub topdown_check_period: ChainEpoch,
}
