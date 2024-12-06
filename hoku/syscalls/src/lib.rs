// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::sync::Arc;

use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use hoku_kernel_ops::HokuOps;
use iroh::blobs::Hash;
use iroh_manager::IrohManager;
use once_cell::sync::Lazy;
use tokio::{spawn, sync::Mutex};

pub const MODULE_NAME: &str = "hoku";
pub const HASHRM_SYSCALL_FUNCTION_NAME: &str = "hash_rm";

const ENV_IROH_ADDR: &str = "IROH_RPC_ADDR";
static IROH_INSTANCE: Lazy<Arc<Mutex<IrohManager>>> = Lazy::new(|| {
    let iroh_addr = std::env::var(ENV_IROH_ADDR).ok();
    Arc::new(Mutex::new(IrohManager::from_addr(iroh_addr)))
});

fn hash_source(bytes: &[u8]) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|e| ExecutionError::Syscall(SyscallError::new(ErrorNumber::IllegalArgument, e)))
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
