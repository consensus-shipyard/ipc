// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Cat command for displaying file contents from storage

use anyhow::{anyhow, Result};
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;

use async_trait::async_trait;
use fendermint_rpc::client::FendermintClient;

use crate::commands::storage::{
    client::download_object_data,
    client_context::resolve_rpc_url,
    config::resolve_client_gateway_url,
    gateway::GatewayClient,
    path,
};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct CatArgs {
    /// Storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Gateway URL (overrides config and env var)
    #[arg(long)]
    pub gateway: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub struct CatStorage;

#[async_trait]
impl CommandLineHandler for CatStorage {
    type Arguments = CatArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let storage_path = path::StoragePath::parse(&args.path)?;

        if storage_path.is_bucket_root() {
            return Err(anyhow!(
                "Path must include a file key, not just a bucket address"
            ));
        }

        let gateway_url =
            resolve_client_gateway_url(args.gateway.as_deref(), args.config.clone(), false)?;
        let rpc_url = resolve_rpc_url(args.config.clone())?;

        let gateway = GatewayClient::new(gateway_url.clone())?;
        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        let data = download_object_data(
            &gateway,
            &gateway_url,
            &fm_client,
            &rpc_url,
            storage_path.bucket_address,
            &storage_path.key,
        )
        .await?;

        io::stdout().write_all(&data)?;
        io::stdout().flush()?;

        Ok(())
    }
}
