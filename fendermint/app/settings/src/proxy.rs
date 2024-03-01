// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::Deserialize;
use serde_with::serde_as;

use crate::SocketAddress;

/// Proxy API facade settings.
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ProxySettings {
    pub listen: SocketAddress,
}
