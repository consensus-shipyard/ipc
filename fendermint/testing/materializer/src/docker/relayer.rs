// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::RelayerName;

use super::DockerConstruct;

/// TODO: Dockerize the relayer
pub struct DockerRelayer {
    relayer_name: RelayerName,
    container: DockerConstruct,
}
