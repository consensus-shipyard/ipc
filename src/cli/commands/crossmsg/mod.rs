// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::cli::commands::crossmsg::fund::Fund;
use crate::cli::commands::crossmsg::propagate::Propagate;
use crate::cli::commands::crossmsg::release::Release;
use crate::cli::commands::crossmsg::whitelist::WhitelistPropagator;
use crate::cli::{CommandLineHandler, GlobalArguments};
use fund::FundArgs;
use propagate::PropagateArgs;
use release::ReleaseArgs;
use whitelist::WhitelistPropagatorArgs;

use clap::{Args, Subcommand};

pub mod fund;
pub mod propagate;
pub mod release;
pub mod whitelist;

#[derive(Debug, Args)]
#[command(name = "crossmsg", about = "cross network messages related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct CrossMsgsCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl CrossMsgsCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Fund(args) => Fund::handle(global, args).await,
            Commands::Release(args) => Release::handle(global, args).await,
            Commands::Propagate(args) => Propagate::handle(global, args).await,
            Commands::WhitelistPropagator(args) => WhitelistPropagator::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Fund(FundArgs),
    Release(ReleaseArgs),
    Propagate(PropagateArgs),
    WhitelistPropagator(WhitelistPropagatorArgs),
}
