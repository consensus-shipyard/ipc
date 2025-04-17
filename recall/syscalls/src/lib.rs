// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::net::SocketAddr;

use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use iroh_blobs::Hash;
use iroh_manager::BlobsClient;
use recall_kernel_ops::RecallOps;
use tokio::sync::Mutex;

pub const MODULE_NAME: &str = "recall";
pub const HASHRM_SYSCALL_FUNCTION_NAME: &str = "hash_rm";

const ENV_IROH_RPC_ADDR: &str = "IROH_SYSCALL_RPC_ADDR";

async fn connect_rpc() -> Option<BlobsClient> {
    let bind_addr: SocketAddr = std::env::var(ENV_IROH_RPC_ADDR).ok()?.parse().ok()?;
    let addr: SocketAddr = format!("127.0.0.1:{}", bind_addr.port()).parse().ok()?;
    iroh_manager::connect_rpc(addr).await.ok()
}
static IROH_RPC_CLIENT: Mutex<Option<BlobsClient>> = Mutex::const_new(None);

fn hash_source(bytes: &[u8]) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|e| ExecutionError::Syscall(SyscallError::new(ErrorNumber::IllegalArgument, e)))
}

pub fn hash_rm(context: Context<'_, impl RecallOps>, hash_offset: u32) -> Result<()> {
    let hash_bytes = context.memory.try_slice(hash_offset, 32)?;
    let seq_hash = Hash::from_bytes(hash_source(hash_bytes)?);

    tracing::debug!("queueing blob {} for deletion", seq_hash);

    // No blocking
    tokio::task::spawn(async move {
        let mut client_lock = IROH_RPC_CLIENT.lock().await;
        if client_lock.is_none() {
            let client = connect_rpc().await;
            if client.is_none() {
                tracing::error!("unable to establish connection to iroh");
                return;
            }
            *client_lock = client;
        }
        let Some(client) = &*client_lock else {
            return;
        };
        if let Err(err) = client.tags().delete(seq_hash).await {
            tracing::warn!(hash = %seq_hash, error = err.to_string(), "deleting tag from iroh failed");
        }
    });

    Ok(())
}
