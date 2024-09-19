// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod sys;

use fvm_shared::error::ErrorNumber;

pub fn hash_rm(hash: [u8; 32]) -> Result<(), ErrorNumber> {
    unsafe { sys::hash_rm(hash.as_ptr()) }
}

pub fn hash_get(hash: [u8; 32], offset: u32) -> Result<[u8; 65536], ErrorNumber> {
    unsafe { sys::hash_get(hash.as_ptr(), offset) }
}
