// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! # fil_actor_adm - ADM Actor Types
//!
//! This crate provides the types and interface for the ADM (Autonomous Data Management) actor.
//! It's designed to be a lightweight dependency for actors that need to interact with ADM.

use serde::{Deserialize, Serialize};

/// Types of machines that can be managed by ADM
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    /// S3-like object storage with key-value semantics
    Bucket,
    /// MMR accumulator for timestamping
    Timehub,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Bucket => write!(f, "bucket"),
            Kind::Timehub => write!(f, "timehub"),
        }
    }
}

