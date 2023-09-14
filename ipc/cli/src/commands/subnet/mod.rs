// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

pub use crate::commands::subnet::create::{CreateSubnet, CreateSubnetArgs};
pub use crate::commands::subnet::join::{JoinSubnet, JoinSubnetArgs};
pub use crate::commands::subnet::kill::{KillSubnet, KillSubnetArgs};
pub use crate::commands::subnet::leave::{LeaveSubnet, LeaveSubnetArgs};
use crate::commands::subnet::list_subnets::{ListSubnets, ListSubnetsArgs};
use crate::commands::subnet::list_validators::{ListValidators, ListValidatorsArgs};
use crate::commands::subnet::net_addr::{SetValidatorNetAddr, SetValidatorNetAddrArgs};
use crate::commands::subnet::rpc::{RPCSubnet, RPCSubnetArgs};
use crate::commands::subnet::send_value::{SendValue, SendValueArgs};
use crate::commands::subnet::worker_addr::{SetValidatorWorkerAddr, SetValidatorWorkerAddrArgs};
use crate::{CommandLineHandler, GlobalArguments};
use clap::{Args, Subcommand};

pub mod create;
pub mod join;
pub mod kill;
pub mod leave;
pub mod list_subnets;
pub mod list_validators;
pub mod net_addr;
pub mod rpc;
pub mod send_value;
pub mod worker_addr;

#[derive(Debug, Args)]
#[command(
    name = "subnet",
    about = "subnet related commands such as create, join and etc"
)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct SubnetCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl SubnetCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Create(args) => CreateSubnet::handle(global, args).await,
            Commands::List(args) => ListSubnets::handle(global, args).await,
            Commands::ListValidators(args) => ListValidators::handle(global, args).await,
            Commands::Join(args) => JoinSubnet::handle(global, args).await,
            Commands::Rpc(args) => RPCSubnet::handle(global, args).await,
            Commands::Leave(args) => LeaveSubnet::handle(global, args).await,
            Commands::Kill(args) => KillSubnet::handle(global, args).await,
            Commands::SendValue(args) => SendValue::handle(global, args).await,
            Commands::SetValidatorNetAddr(args) => SetValidatorNetAddr::handle(global, args).await,
            Commands::SetValidatorWorkerAddr(args) => {
                SetValidatorWorkerAddr::handle(global, args).await
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create(CreateSubnetArgs),
    List(ListSubnetsArgs),
    ListValidators(ListValidatorsArgs),
    Join(JoinSubnetArgs),
    Rpc(RPCSubnetArgs),
    Leave(LeaveSubnetArgs),
    Kill(KillSubnetArgs),
    SendValue(SendValueArgs),
    SetValidatorNetAddr(SetValidatorNetAddrArgs),
    SetValidatorWorkerAddr(SetValidatorWorkerAddrArgs),
}
