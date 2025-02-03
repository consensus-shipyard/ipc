// Copyright 2025 Recall Contributors
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
    pub max_object_size: u64,
    pub listen: SocketAddress,
    pub tracing: TracingSettings,
    pub metrics: MetricsSettings,
}
