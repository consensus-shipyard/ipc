// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod sys;

use fvm_shared::error::ErrorNumber;

pub fn hash_rm(hash: [u8; 32]) -> Result<(), ErrorNumber> {
    unsafe { sys::hash_rm(hash.as_ptr()) }
}
