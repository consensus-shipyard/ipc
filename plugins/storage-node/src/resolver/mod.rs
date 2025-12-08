// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Storage resolver for Iroh content resolution.
//!
//! This module was moved from fendermint/vm/storage_resolver/ to achieve
//! true plugin isolation. It handles resolution of storage blobs and read
//! requests using the Iroh network.

pub mod iroh;
pub mod observe;
pub mod pool;

pub use iroh::IrohResolver;
pub use pool::{ResolvePool, ResolveKey, ResolveSource, TaskType};
