// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Display;
use std::sync::Arc;

use crate::hoku_kernel::HokuOps;
use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use iroh::blobs::Hash;
use maybe_iroh::MaybeIroh;
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use tokio::{spawn, sync::Mutex};

pub mod hoku_kernel;

pub const SYSCALL_MODULE_NAME: &str = "blobs";
pub const HASHRM_SYSCALL_FUNCTION_NAME: &str = "hash_rm";
pub const HASHGET_SYSCALL_FUNCTION_NAME: &str = "hash_get";

const ENV_IROH_ADDR: &str = "IROH_RPC_ADDR";
const HASHRM_SYSCALL_ERROR_CODE: u32 = 101; // TODO(sander): Is the okay?
const HASHGET_SYSCALL_ERROR_CODE: u32 = 102;

static IROH_INSTANCE: Lazy<Arc<Mutex<MaybeIroh>>> = Lazy::new(|| {
    let iroh_addr = std::env::var(ENV_IROH_ADDR).ok();
    Arc::new(Mutex::new(MaybeIroh::maybe_addr(iroh_addr)))
});

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

pub fn hash_rm(context: Context<'_, impl HokuOps>, hash_offset: u32) -> Result<()> {
    let hash_bytes = context.memory.try_slice(hash_offset, 32)?;
    let hash = Hash::from_bytes(hash_source(hash_bytes)?);
    let iroh = IROH_INSTANCE.clone();

    // Don't block the chain with this.
    spawn(async move {
        let iroh_client = match iroh.lock().await.client().await {
            Ok(client) => client,
            Err(e) => {
                tracing::error!(hash = ?hash, error = e.to_string(), "failed to initialize Iroh client");
                return;
            }
        };
        // Deleting the tag will trigger deletion of the blob if it was the last reference.
        // TODO: this needs to be tagged with a "user id"
        let tag = iroh::blobs::Tag(format!("stored-{hash}").into());
        match iroh_client.tags().delete(tag.clone()).await {
            Ok(_) => tracing::debug!(tag = ?tag, hash = ?hash, "removed content from Iroh"),
            Err(e) => {
                tracing::warn!(tag = ?tag, hash = ?hash, error = e.to_string(), "deleting tag from Iroh failed");
            }
        }
    });
    Ok(())
}

pub fn hash_get(
    context: Context<'_, impl HokuOps>,
    hash_offset: u32,
    offset: u32,
) -> Result<[u8; 65536]> {
    let hash_bytes = context.memory.try_slice(hash_offset, 32)?;
    let hash = Hash::from_bytes(hash_source(hash_bytes)?);
    let iroh = IROH_INSTANCE.clone();

    let blob_bytes = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let iroh_client = match iroh.lock().await.client().await {
                Ok(client) => client,
                Err(e) => {
                    tracing::error!(hash = ?hash, error = e.to_string(), "failed to initialize Iroh client");
                    return Err(syscall_error(HASHGET_SYSCALL_ERROR_CODE)(e));
                }
            };

            let blob = match iroh_client.blobs().read(hash).await {
                Ok(blob) => blob,
                Err(e) => {
                    tracing::error!(hash = ?hash, error = e.to_string(), "failed to read blob");
                    return Err(syscall_error(HASHGET_SYSCALL_ERROR_CODE)(e));
                }
            };

            let size = blob.size();
            let length = std::cmp::min(size, 65536) as usize;

            match iroh_client.blobs().read_at_to_bytes(hash, offset as u64, Some(length - offset as usize)).await {
                Ok(bytes) => Ok(bytes),
                Err(e) => {
                    tracing::error!(hash = ?hash, error = e.to_string(), "failed to read blob bytes");
                    Err(syscall_error(HASHGET_SYSCALL_ERROR_CODE)(e))
                }
            }
        })
    })?;

    // Convert Bytes to [u8; 65536], padding with zeros if necessary
    let mut result = [0u8; 65536];
    let len = std::cmp::min(blob_bytes.len(), 65536);
    result[..len].copy_from_slice(&blob_bytes[..len]);

    Ok(result)
}
