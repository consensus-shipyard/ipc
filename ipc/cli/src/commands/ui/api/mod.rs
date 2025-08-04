// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! API modules for the UI service
//!
//! This module organizes API endpoints into logical groups.

pub mod types;
pub mod deployment;
pub mod subnet;
pub mod wallet;
pub mod gateway;
pub mod transactions;

pub use types::*;
pub use deployment::*;
pub use subnet::*;
pub use wallet::*;
pub use gateway::*;
pub use transactions::*;