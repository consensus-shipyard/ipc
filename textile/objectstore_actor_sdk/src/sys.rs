// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_sdk::sys::fvm_syscalls;

fvm_syscalls! {
    module = "objectstore";

    pub fn cid_rm(cid: *const u8, cid_len: u32) -> Result<()>;
    pub fn hash_rm(hash_ptr: *const u8) -> Result<()>;
}
