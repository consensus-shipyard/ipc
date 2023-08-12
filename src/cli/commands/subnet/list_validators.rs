// Copyright 2022-2023 Protocol Labs
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Get membership cli command

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::message::ipc::QueryValidatorSetResponse;
use crate::server::query_validators::QueryValidatorSetParams;
use serde::Deserialize;

/// The command to create a new subnet actor.
pub(crate) struct ListValidators;

#[async_trait]
impl CommandLineHandler for ListValidators {
    type Arguments = ListValidatorsArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list validators with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = QueryValidatorSetParams {
            gateway_address: arguments.gateway_address.clone(),
            subnet: arguments.subnet.clone(),
        };

        let valset = json_rpc_client
            .request::<QueryValidatorSetResponse>(
                json_rpc_methods::QUERY_VALIDATOR_SET,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("validators number: {}", valset.min_validators);
        log::info!("validators: {:?}", valset.validator_set);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(
    name = "list-validators",
    about = "Show the validators of the subnet"
)]
pub(crate) struct ListValidatorsArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The gateway address to query subnets")]
    pub gateway_address: String,
    #[arg(long, short, help = "The subnet id to query validators")]
    pub subnet: String,
}
