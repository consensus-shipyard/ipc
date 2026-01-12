// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Storage Node Integration Layer
//!
//! This module provides the integration API between storage-node functionality
//! and the IPC core codebase. All storage-node features are gated behind the
//! `storage-node` feature flag for conditional compilation.

pub mod actor_interface;
pub mod storage_env;
pub mod storage_helpers;

// Re-export commonly used types for convenience
pub use storage_env::{BlobPool, BlobPoolItem, ReadRequestPool, ReadRequestPoolItem};
pub use storage_helpers::{
    close_read_request, read_request_callback, set_read_request_pending,
};

