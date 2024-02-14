// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::TestnetName;

pub struct DockerNetwork {
    name: TestnetName,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    id: String,
}
