// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The Daemon command line handler that prints the info about IPC Agent.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use clap::Args;

use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::ReloadableConfig;
use crate::server::jsonrpc::JsonRPCServer;

/// The command to start the ipc agent json rpc server in the foreground.
pub(crate) struct LaunchDaemon;

#[async_trait]
impl CommandLineHandler for LaunchDaemon {
    type Arguments = LaunchDaemonArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!(
            "launching json rpc server with args: {:?} and global params: {:?}",
            arguments,
            global
        );

        let reloadable_config = Arc::new(ReloadableConfig::new(global.config_path())?);

        let server = JsonRPCServer::new(reloadable_config);
        server.run().await?;

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Launch the ipc agent daemon process")]
pub(crate) struct LaunchDaemonArgs {}
