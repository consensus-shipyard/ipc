// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::error::ErrorNumber;

/// Deletes a blob by hash from backing storage.
pub fn delete_blob(hash: [u8; 32]) -> Result<(), ErrorNumber> {
    unsafe { sys::delete_blob(hash.as_ptr()) }
}

mod sys {
    use fvm_sdk::sys::fvm_syscalls;

    fvm_syscalls! {
        module = "recall";

        /// Deletes a blob by hash from backing storage.
        pub fn delete_blob(hash_ptr: *const u8) -> Result<()>;
    }
}
