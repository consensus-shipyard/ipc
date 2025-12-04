// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

#[cfg(feature = "fil-actor")]
use crate::hash_algorithm::FvmHashSha256;
#[cfg(not(feature = "fil-actor"))]
use fvm_ipld_hamt::Sha256;

pub mod amt;
pub mod hamt;
mod hash_algorithm;

#[cfg(feature = "fil-actor")]
type Hasher = FvmHashSha256;

#[cfg(not(feature = "fil-actor"))]
type Hasher = Sha256;
