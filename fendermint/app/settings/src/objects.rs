// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{MetricsSettings, SocketAddress};
use ipc_observability::config::TracingSettings;
use serde::Deserialize;
use serde_with::serde_as;

/// Object API facade settings.
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ObjectsSettings {
    pub listen: SocketAddress,
    pub tracing: TracingSettings,
    pub metrics: MetricsSettings,
}
