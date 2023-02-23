//! The Daemon command line handler that prints the info about IPC Agent.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::CommandLineHandler;
use crate::config::Config;
use crate::server::jsonrpc::JsonRPCServer;

/// The command to start the ipc agent json rpc server in the foreground.
pub(crate) struct LaunchDaemon;

#[async_trait]
impl CommandLineHandler for LaunchDaemon {
    type Arguments = LaunchDaemonArgs;

    async fn handle(arguments: &Self::Arguments) -> anyhow::Result<()> {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

        log::debug!("launching json rpc server with args: {:?}", arguments);

        let config = Config::from_file(&arguments.config_file)?.server;
        log::info!("starting IPC-agent daemon at: {:}", config.json_rpc_address);

        let server = JsonRPCServer::new(config);
        server.run().await;

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Launch the ipc agent daemon process")]
pub(crate) struct LaunchDaemonArgs {
    #[arg(help = "The config file path to start the json rpc server")]
    config_file: String,
}
