// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::error::ErrorNumber;

pub fn hash_rm(hash: [u8; 32]) -> Result<(), ErrorNumber> {
    unsafe { sys::hash_rm(hash.as_ptr()) }
}

mod sys {
    use fvm_sdk::sys::fvm_syscalls;

    fvm_syscalls! {
        module = "hoku";
        pub fn hash_rm(hash_ptr: *const u8) -> Result<()>;
    }
}
