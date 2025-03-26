// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_app::service::node::run as run_node;
use fendermint_app_settings::Settings;
use ipc_observability::config::TracingSettings;
use ipc_observability::traces::set_global_tracing_subscriber;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::spawn;

use std::process::Stdio;
use tokio::process::Command;

use ipc_runner::*;

#[tokio::main]
async fn main() {
    let home_dir = Path::new("~/.fendermint");
    let config_dir = home_dir.join("config");
    let run_mode = "dev";
    let settings = Settings::new(&config_dir, &home_dir, run_mode).unwrap();

    let _trace_file_guard = set_global_tracing_subscriber(&TracingSettings::default());

    let node_handle = spawn(run_node(settings.clone()));

    let comet_bft_handle = spawn(run_cometbft());

    let (r1, r2) = tokio::join!(node_handle, comet_bft_handle);

    let _ = r1.expect("Failed to run node");
    let _ = r2.expect("Failed to run cometbft");
}

async fn run_cometbft() -> anyhow::Result<()> {
    // Spawn the binary with stdout piped.
    let mut child = Command::new(BINARY_PATH)
        .arg("start")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute binary");

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();

        // Read and print lines as they become available.
        while let Some(line) = reader.next_line().await? {
            println!("cometbft: {}", line);
        }
    }

    // Wait for the process to exit.
    let status = child.wait().await?;
    println!("cometbft exited with status: {:?}", status);

    Ok(())
}
