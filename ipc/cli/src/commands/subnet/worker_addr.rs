// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The command to set the validator worker address

use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

/// Setting the validator worker address
pub(crate) struct SetValidatorWorkerAddr;

#[async_trait]
impl CommandLineHandler for SetValidatorWorkerAddr {
    type Arguments = SetValidatorWorkerAddrArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("set the validator worker addr args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        provider
            .set_validator_worker_addr(
                subnet,
                from,
                Address::from_str(&arguments.validator_worker_addr)?,
            )
            .await
    }
}

#[derive(Debug, Args)]
#[command(about = "Set the validator worker address")]
pub(crate) struct SetValidatorWorkerAddrArgs {
    #[arg(long, short, help = "Owner address of the validator being updated")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to set the validator")]
    pub subnet: String,
    #[arg(long, short, help = "New validator worker address")]
    pub validator_worker_addr: String,
}
