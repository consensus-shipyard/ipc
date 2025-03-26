// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::{CommandLineHandler, GlobalArguments};
use async_trait::async_trait;
use fs_err as fs;
use ipc_provider::config::DEFAULT_CONFIG_TEMPLATE;
use std::io::Write;

use clap::Args;

pub(crate) struct InitNode;

#[async_trait]
impl CommandLineHandler for InitNode {
    type Arguments = InitNodeArgs;

    async fn handle(global: &GlobalArguments, _arguments: &Self::Arguments) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Arguments to initialize a new node")]
pub(crate) struct InitNodeArgs {}
