// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use fendermint_app_options::debug::{
    DebugArgs, DebugCommands, DebugExportTopDownEventsArgs, DebugIpcCommands,
};
use fendermint_vm_topdown::proxy::IPCProviderProxy;
use ipc_provider::{
    config::subnet::{EVMSubnet, SubnetConfig},
    IpcProvider,
};

use crate::cmd;

cmd! {
  DebugArgs(self) {
    match &self.command {
        DebugCommands::Ipc { command } => command.exec(()).await,
    }
  }
}

cmd! {
  DebugIpcCommands(self) {

    match self {
        DebugIpcCommands::ExportTopDownEvents(args) => {
            export_topdown_events(args).await
        }
    }
  }
}

async fn export_topdown_events(_args: &DebugExportTopDownEventsArgs) -> anyhow::Result<()> {
    todo!("integrate new RPC endpoints")
}
