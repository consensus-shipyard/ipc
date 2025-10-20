// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Common constants for FVM operations in IPC.

/// Block gas limit for IPC.
///
/// This constant was removed in FVM 4.7 as FVM no longer enforces block gas limits.
/// IPC continues to use this limit for gas estimation and block validation.
/// The value of 10 billion was chosen to provide reasonable bounds while allowing
/// for complex transactions within a block.
pub const BLOCK_GAS_LIMIT: u64 = 10_000_000_000;
