// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub const BUILTIN_SOL_ACTOR_ARTIFACTS: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/super.json"));
