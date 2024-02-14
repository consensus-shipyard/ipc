// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::NodeName;

/// A docker node consists of multiple containers.
pub struct DockerNode {
    name: NodeName,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    cometbft_container_name: String,
    fendermint_container_name: String,
    ethapi_container_name: Option<String>,
    // TODO: Ports
}
