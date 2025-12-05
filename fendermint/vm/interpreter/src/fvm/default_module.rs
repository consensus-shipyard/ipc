// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Module selection for compile-time feature-based configuration.
//!
//! This module defines which module implementation to use based on
//! the features enabled at compile time.

use fendermint_module::NoOpModuleBundle;

/// The module implementation selected at compile time.
///
/// For now, always uses the NoOpModuleBundle. The storage-node module
/// integration will be completed in a follow-up step once the module
/// interface is stable.
///
/// TODO: Uncomment when storage-node module is ready
/// #[cfg(feature = "storage-node")]
/// pub type DefaultModule = storage_node_module::StorageNodeModule;
pub type DefaultModule = NoOpModuleBundle;
