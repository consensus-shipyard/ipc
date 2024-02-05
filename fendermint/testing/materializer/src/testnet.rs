// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::manifest::{ResourceId, ResourceName};

/// The `Testnet` parses a [Manifest] and is able to derive the steps
/// necessary to instantiate it with the help of the materializer.
pub struct Testnet {
    /// An identifier derived from the manifest file name.
    pub name: ResourceName,
}
