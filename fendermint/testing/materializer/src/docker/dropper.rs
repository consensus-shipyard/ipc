// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use bollard::{
    container::{RemoveContainerOptions, StopContainerOptions},
    Docker,
};

/// Timemout before we kill the container if it doesn't want to stop.
const KILL_TIMEOUT_SECS: i64 = 5;

/// Commands to destroy docker constructs when they go out of scope.
pub enum DropCommand {
    DropNetwork(String),
    DropContainer(String),
}

pub type DropHandle = tokio::sync::mpsc::UnboundedSender<DropCommand>;

/// Start a background task to remove docker constructs.
///
/// The loop will exit when all clones of the sender channel have been dropped.
pub fn start(docker: Docker) -> DropHandle {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::task::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                DropCommand::DropNetwork(id) => {
                    eprintln!("dropping docker network {id}");
                    if let Err(e) = docker.remove_network(&id).await {
                        eprintln!("failed to remove docker network: {e}");
                        tracing::error!(
                            error = e.to_string(),
                            id,
                            "failed to remove docker network"
                        );
                    }
                }
                DropCommand::DropContainer(id) => {
                    eprintln!("dropping docker container {id}");

                    if let Err(e) = docker
                        .stop_container(
                            &id,
                            Some(StopContainerOptions {
                                t: KILL_TIMEOUT_SECS,
                            }),
                        )
                        .await
                    {
                        tracing::error!(
                            error = e.to_string(),
                            id,
                            "failed to stop docker container"
                        );
                    }

                    if let Err(e) = docker
                        .remove_container(
                            &id,
                            Some(RemoveContainerOptions {
                                force: true,
                                v: true,
                                ..Default::default()
                            }),
                        )
                        .await
                    {
                        eprintln!("failed to remove container: {e}");

                        tracing::error!(
                            error = e.to_string(),
                            id,
                            "failed to remove docker container"
                        );
                    }
                }
            }
        }
    });

    tx
}
