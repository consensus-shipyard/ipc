// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod objectstore_kernel;

use crate::objectstore_kernel::ObjectStoreOps;
use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use iroh::blobs::Hash;
use maybe_iroh::MaybeIroh;
use num_traits::FromPrimitive;
use std::fmt::Display;

pub const SYSCALL_MODULE_NAME: &str = "objectstore";
pub const HASHRM_SYSCALL_FUNCTION_NAME: &str = "hash_rm";

const ENV_IROH_ADDR: &str = "IROH_RPC_ADDR";
const HASHRM_SYSCALL_ERROR_CODE: u32 = 101; // TODO(sander): Is the okay?

fn syscall_error<D: Display>(error_number: u32) -> impl FnOnce(D) -> ExecutionError {
    move |e| {
        ExecutionError::Syscall(SyscallError::new(
            ErrorNumber::from_u32(error_number).unwrap(),
            e,
        ))
    }
}

fn hash_source(bytes: &[u8]) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|e| syscall_error(HASHRM_SYSCALL_ERROR_CODE)(e))
}

pub fn hash_rm(context: Context<'_, impl ObjectStoreOps>, hash_offset: u32) -> Result<()> {
    let hash_bytes = context.memory.try_slice(hash_offset, 32)?;
    let hash = Hash::from_bytes(hash_source(hash_bytes)?);
    let iroh_addr =
        std::env::var(ENV_IROH_ADDR).map_err(|e| syscall_error(HASHRM_SYSCALL_ERROR_CODE)(e))?;
    let mut iroh = MaybeIroh::from_addr(iroh_addr);

    // Don't block the chain with this.
    tokio::spawn(async move {
        match iroh.client().await {
            Ok(iroh) => match iroh.blobs().delete_blob(hash).await {
                Ok(_) => tracing::debug!(hash = ?hash, "removed content from Iroh"),
                Err(e) => {
                    tracing::error!(hash = ?hash, error = e.to_string(), "removing content from Iroh failed")
                }
            },
            Err(e) => {
                tracing::error!(hash = ?hash, error = e.to_string(), "removing content from Iroh failed")
            }
        }
    });
    Ok(())
}
