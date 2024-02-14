// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::NodeName;

pub struct DockerRelayer {
    name: NodeName,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    relayer_container_name: String,
}
