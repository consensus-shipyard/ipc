// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod objectstore_kernel;

use crate::objectstore_kernel::ObjectStoreOps;
use cid::Cid;
use fvm::kernel::{ExecutionError, Result, SyscallError};
use fvm::syscalls::Context;
use fvm_shared::error::ErrorNumber;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use log::info;
use num_traits::FromPrimitive;
use std::fmt::Display;

pub const SYSCALL_MODULE_NAME: &str = "objectstore";
pub const CIDRM_SYSCALL_FUNCTION_NAME: &str = "cid_rm";

const ENV_IPFS_ADDR: &str = "IPFS_RPC_ADDR";
const CIDRM_SYSCALL_ERROR_CODE: u32 = 101; // TODO(sander): Is the okay?

fn syscall_error<D: Display>(error_number: u32) -> impl FnOnce(D) -> ExecutionError {
    move |e| {
        ExecutionError::Syscall(SyscallError::new(
            ErrorNumber::from_u32(error_number).unwrap(),
            e,
        ))
    }
}

pub fn cid_rm(context: Context<'_, impl ObjectStoreOps>, cid_off: u32, cid_len: u32) -> Result<()> {
    let cid = context.memory.try_slice(cid_off, cid_len)?;
    let cid = Cid::try_from(cid).map_err(syscall_error(CIDRM_SYSCALL_ERROR_CODE))?;

    let ipfs_addr = std::env::var(ENV_IPFS_ADDR);
    match ipfs_addr {
        Ok(ipfs_addr) => {
            // Don't block the chain with this.
            tokio::spawn(async move {
                match IpfsClient::from_multiaddr_str(&ipfs_addr) {
                    Ok(ipfs) => match ipfs.pin_rm(&cid.to_string(), true).await {
                        Ok(_) => info!("unresolved {} from IPFS", cid),
                        Err(e) => info!("unresolving {} from IPFS failed with {}", cid, e),
                    },
                    Err(e) => info!("unresolving {} from IPFS failed with {}", cid, e),
                }
            });
            Ok(())
        }
        Err(e) => Err(syscall_error(CIDRM_SYSCALL_ERROR_CODE)(e)),
    }
}
