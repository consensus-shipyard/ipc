// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The Daemon command line handler that prints the info about IPC Agent.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use clap::Args;
use tokio::try_join;
use tokio_graceful_shutdown::{IntoSubsystem, Toplevel};

use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::ReloadableConfig;
use crate::manager::checkpoint::CheckpointSubsystem;
use crate::server::jsonrpc::JsonRPCServer;

/// The number of seconds to wait for a subsystem to start before returning an error.
const SUBSYSTEM_WAIT_TIME_SECS: Duration = Duration::from_secs(10);

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

        // Start checkpoint subsystem.
        let ch_subsys = CheckpointSubsystem::new(reloadable_config.clone());
        let subsys_handle = tokio::spawn({
            Toplevel::new()
                .start("Checkpoint subsystem", ch_subsys.into_subsystem())
                .catch_signals()
                .handle_shutdown_requests(SUBSYSTEM_WAIT_TIME_SECS)
        });

        // Start JSON-RPC server.
        // TODO: convert JsonRPCServer to a subsystem.
        let server = JsonRPCServer::new(reloadable_config.clone());
        let server_handle = tokio::spawn(async move { server.run().await });

        // Drive the futures. Return if some encounters an error.
        let results = try_join!(subsys_handle, server_handle)?;
        results.0?;
        results.1?;

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Launch the ipc agent daemon process")]
pub(crate) struct LaunchDaemonArgs {}
