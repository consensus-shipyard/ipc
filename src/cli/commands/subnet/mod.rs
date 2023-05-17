// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

pub use crate::cli::commands::subnet::create::{CreateSubnet, CreateSubnetArgs};
pub use crate::cli::commands::subnet::join::{JoinSubnet, JoinSubnetArgs};
pub use crate::cli::commands::subnet::kill::{KillSubnet, KillSubnetArgs};
pub use crate::cli::commands::subnet::leave::{LeaveSubnet, LeaveSubnetArgs};
use crate::cli::commands::subnet::list_subnets::{ListSubnets, ListSubnetsArgs};
use crate::cli::commands::subnet::net_addr::{SetValidatorNetAddr, SetValidatorNetAddrArgs};
use crate::cli::commands::subnet::send_value::{SendValue, SendValueArgs};
use crate::cli::{CommandLineHandler, GlobalArguments};
use clap::{Args, Subcommand};

use self::rpc::{RPCSubnet, RPCSubnetArgs};

pub mod create;
pub mod join;
pub mod kill;
pub mod leave;
pub mod list_subnets;
pub mod net_addr;
pub mod rpc;
pub mod send_value;

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
            Commands::Join(args) => JoinSubnet::handle(global, args).await,
            Commands::Rpc(args) => RPCSubnet::handle(global, args).await,
            Commands::Leave(args) => LeaveSubnet::handle(global, args).await,
            Commands::Kill(args) => KillSubnet::handle(global, args).await,
            Commands::SendValue(args) => SendValue::handle(global, args).await,
            Commands::SetValidatorNetAddr(args) => SetValidatorNetAddr::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create(CreateSubnetArgs),
    List(ListSubnetsArgs),
    Join(JoinSubnetArgs),
    Rpc(RPCSubnetArgs),
    Leave(LeaveSubnetArgs),
    Kill(KillSubnetArgs),
    SendValue(SendValueArgs),
    SetValidatorNetAddr(SetValidatorNetAddrArgs),
}
