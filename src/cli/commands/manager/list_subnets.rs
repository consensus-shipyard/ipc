// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List subnets cli command

use async_trait::async_trait;
use clap::Args;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::list_subnets::ListSubnetsParams;
use serde::Deserialize;

/// The command to create a new subnet actor.
pub(crate) struct ListSubnets;

#[async_trait]
impl CommandLineHandler for ListSubnets {
    type Arguments = ListSubnetsArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list subnets with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = ListSubnetsParams {
            gateway_address: arguments.gateway_address.clone(),
            subnet_id: arguments.subnet_id.clone(),
        };

        let subnets = json_rpc_client
            .request::<HashMap<String, SubnetInfoWrapper>>(
                json_rpc_methods::LIST_CHILD_SUBNETS,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("found child subnets: {subnets:?}");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List child subnets")]
pub(crate) struct ListSubnetsArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The gateway address to query subnets")]
    pub gateway_address: String,
    #[arg(long, short, help = "The subnet id to query child subnets")]
    pub subnet_id: String,
}

/// A simplified wrapper for Subnet Info response. The SubnetInfo struct is deserialized differently
/// as that struct is targeting deserialization from Actor. SubnetInfoWrapper is targeting ipc-agent
/// rpc server, it is using different data structure and casing, i.e. id in actor is represented as
/// a map, but in ipc-agent rpc server, it is a string.
#[derive(Debug, Deserialize)]
struct SubnetInfoWrapper {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    stake: String,
    #[allow(dead_code)]
    circ_supply: String,
    #[allow(dead_code)]
    status: i32,
}
