// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
mod manifest;

pub use manifest::Manifest;

/// Included bytes for custom actor bundle, ~1.3M in size
pub const CAR: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../actors/output/",
    "custom_actors_bundle.car"
));
