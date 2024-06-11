// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::Deserialize;
use serde_with::serde_as;

use crate::SocketAddress;

/// Object API facade settings.
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ObjectsSettings {
    pub listen: SocketAddress,
}
