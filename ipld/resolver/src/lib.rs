// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
// TODO (IPC-38): Remove dead code allowances.
#[allow(dead_code)]
mod behaviour;
#[allow(dead_code)]
mod provider_cache;
#[allow(dead_code)]
mod provider_record;
#[allow(dead_code)]
mod service;

#[cfg(any(test, feature = "arb"))]
mod arb;
