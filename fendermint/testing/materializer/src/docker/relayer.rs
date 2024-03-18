// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Display;

use crate::RelayerName;

use super::DockerConstruct;

/// TODO: Dockerize the relayer
pub struct DockerRelayer {
    relayer_name: RelayerName,
    container: DockerConstruct,
}

impl Display for DockerRelayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.relayer_name, f)
    }
}
