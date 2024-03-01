// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use bollard::{
    container::{RemoveContainerOptions, StopContainerOptions},
    Docker,
};

/// Timeout before we kill the container if it doesn't want to stop.
const KILL_TIMEOUT_SECS: i64 = 5;

/// Commands to destroy docker constructs when they go out of scope.
pub enum DropCommand {
    DropNetwork(String),
    DropContainer(String),
}

pub type DropHandle = tokio::sync::mpsc::UnboundedSender<DropCommand>;

/// Decide whether to keep or discard constructs when they go out of scope.
#[derive(Clone, Debug)]
pub struct DropPolicy {
    pub keep_existing: bool,
    pub keep_created: bool,
}

impl DropPolicy {
    /// A completely transient network that aims to drop even what exists,
    /// assuming it only exists because it was created by it earlier, but
    /// due to some error it failed to be removed.
    pub const TRANSIENT: DropPolicy = DropPolicy {
        keep_existing: false,
        keep_created: false,
    };

    /// Keep everything around, which is good for CLI applications that
    /// set up networks that should exist until explicitly removed.
    pub const PERSISTENT: DropPolicy = DropPolicy {
        keep_existing: true,
        keep_created: true,
    };

    /// Policy which only tries to remove artifacts which were created
    /// by this materializer, but leaves existing resources around.
    pub const DROP_CREATED: DropPolicy = DropPolicy {
        keep_created: false,
        keep_existing: true,
    };

    /// Decide if something should be kept when it's out of scope.
    pub fn keep(&self, is_new: bool) -> bool {
        if is_new {
            self.keep_created
        } else {
            self.keep_existing
        }
    }
}

impl Default for DropPolicy {
    fn default() -> Self {
        Self::DROP_CREATED
    }
}

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
