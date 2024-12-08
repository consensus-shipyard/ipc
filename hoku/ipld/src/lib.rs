// Copyright 2024 Hoku Contributors
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

#[cfg(feature = "fil-actor")]
use crate::hash_algorithm::FvmHashSha256;
#[cfg(not(feature = "fil-actor"))]
use fvm_ipld_hamt::Sha256;

mod hash_algorithm;
pub mod map;

pub const HAMT_BIT_WIDTH: u32 = 5;

#[cfg(feature = "fil-actor")]
type Hasher = FvmHashSha256;

#[cfg(not(feature = "fil-actor"))]
type Hasher = Sha256;
