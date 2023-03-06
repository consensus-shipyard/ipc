// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The Daemon command line handler that prints the info about IPC Agent.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::CommandLineHandler;
use crate::server::jsonrpc::JsonRPCServer;

/// The command to start the ipc agent json rpc server in the foreground.
pub(crate) struct LaunchDaemon;

#[async_trait]
impl CommandLineHandler for LaunchDaemon {
    type Arguments = LaunchDaemonArgs;

    async fn handle(arguments: &Self::Arguments) -> anyhow::Result<()> {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

        log::debug!("launching json rpc server with args: {:?}", arguments);

        let server = JsonRPCServer::from_config_path(&arguments.config_file)?;
        server.run().await?;

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Launch the ipc agent daemon process")]
pub(crate) struct LaunchDaemonArgs {
    #[arg(help = "The config file path to start the json rpc server")]
    config_file: String,
}
