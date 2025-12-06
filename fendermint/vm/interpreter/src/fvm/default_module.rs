// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Module selection for compile-time feature-based configuration.
//!
//! This module defines which module implementation to use based on
//! the features enabled at compile time.

/// The module implementation selected at compile time.
///
/// When the `storage-node` feature is enabled, uses `StorageNodeModule`
/// which integrates the RecallExecutor and storage-node functionality.
/// Otherwise, uses the baseline `NoOpModuleBundle`.
#[cfg(not(feature = "storage-node"))]
pub type DefaultModule = fendermint_module::NoOpModuleBundle;

#[cfg(feature = "storage-node")]
pub type DefaultModule = storage_node_module::StorageNodeModule;
