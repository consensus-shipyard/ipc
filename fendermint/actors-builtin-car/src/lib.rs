// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
/// Included builtin actors bundle
pub const CAR: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/", "builtin_actors.car"));
