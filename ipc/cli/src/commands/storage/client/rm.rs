// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Remove command for deleting objects from storage

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;

use async_trait::async_trait;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::eam::EthAddress;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;

use crate::commands::storage::{bucket, client_context::resolve_write_context, path};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Recursive delete (for prefix-based deletion)
    #[arg(short, long)]
    pub recursive: bool,

    /// Force deletion without confirmation
    #[arg(short, long)]
    pub force: bool,
}

pub struct RemoveStorage;

#[async_trait]
impl CommandLineHandler for RemoveStorage {
    type Arguments = RemoveArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let storage_path = path::StoragePath::parse(&args.path)?;

        if storage_path.is_bucket_root() {
            return Err(anyhow!(
                "Cannot delete entire bucket. Specify a key or prefix."
            ));
        }

        // Handle recursive deletion
        if args.recursive {
            return delete_recursive(global, &storage_path, args).await;
        }

        // Single file deletion
        delete_file(global, &storage_path, args).await
    }
}

async fn delete_file(
    global: &GlobalArguments,
    storage_path: &path::StoragePath,
    args: &RemoveArgs,
) -> Result<()> {
    // Confirm deletion unless --force
    if !args.force {
        print!("Delete {}? [y/N] ", storage_path.to_uri());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted");
            return Ok(());
        }
    }

    let write_ctx = resolve_write_context(global, args.config.clone())?;
    let rpc_url = write_ctx.rpc_url;
    let secret_key = write_ctx.secret_key;

    // Create FendermintClient and bound client
    let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

    // Query chain ID from the network
    let chain_id = bucket::query_chain_id(&fm_client)
        .await
        .with_context(|| format!("Failed to query chain ID from rpc={}", rpc_url))?;

    let pub_key = secret_key.public_key();
    let eth_addr = EthAddress::new_secp256k1(&pub_key.serialize())
        .context("failed to derive delegated address")?;
    let addr =
        Address::new_delegated(10, &eth_addr.0).context("failed to construct f410 address")?;
    let state = fm_client
        .actor_state(
            &addr,
            fendermint_vm_message::query::FvmQueryHeight::default(),
        )
        .await
        .with_context(|| format!("Failed to get actor state for sender {} via rpc={}", addr, rpc_url))?;
    let sequence = state.value.map(|(_, s)| s.sequence).ok_or_else(|| {
        anyhow!(
            "sender actor {} does not exist on-chain at {}. Fund/initialize this delegated \
             address first.",
            addr,
            rpc_url
        )
    })?;

    let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
    let mut bound_client = fm_client.bind(mf);

    // Delete object
    println!("Deleting {}...", storage_path.key);

    bucket::delete_object(
        &mut bound_client,
        storage_path.bucket_address,
        storage_path.key.clone(),
    )
    .await
    .with_context(|| {
        format!(
            "Failed to delete object (bucket={} key={} sender={} rpc={})",
            storage_path.bucket_address, storage_path.key, addr, rpc_url
        )
    })?;

    println!("✓ Deleted: {}", storage_path.key);

    Ok(())
}

async fn delete_recursive(
    global: &GlobalArguments,
    storage_path: &path::StoragePath,
    args: &RemoveArgs,
) -> Result<()> {
    let write_ctx = resolve_write_context(global, args.config.clone())?;
    let rpc_url = write_ctx.rpc_url;
    let secret_key = write_ctx.secret_key;

    // List all objects with the prefix
    let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

    let prefix = storage_path.key.clone();
    let mut deleted_count = 0u64;
    let mut failed_count = 0u64;
    let mut start_key = None;

    loop {
        let list_result = bucket::list_objects(
            &fm_client,
            storage_path.bucket_address,
            Some(prefix.clone()),
            None, // no delimiter for recursive
            start_key,
            100, // batch size
        )
        .await
        .with_context(|| {
            format!(
                "Failed to list objects for recursive delete (bucket={} prefix={} rpc={})",
                storage_path.bucket_address, prefix, rpc_url
            )
        })?;

        if list_result.objects.is_empty() {
            break;
        }

        // Confirm deletion unless --force
        if !args.force && deleted_count == 0 {
            println!(
                "Found {} objects to delete with prefix: {}",
                list_result.objects.len(),
                prefix
            );
            print!("Continue? [y/N] ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted");
                return Ok(());
            }
        }

        // Delete each object
        let chain_id = bucket::query_chain_id(&fm_client)
            .await
            .with_context(|| format!("Failed to query chain ID from rpc={}", rpc_url))?;

        let pub_key = secret_key.public_key();
        let eth_addr = EthAddress::new_secp256k1(&pub_key.serialize())
            .context("failed to derive delegated address")?;
        let addr =
            Address::new_delegated(10, &eth_addr.0).context("failed to construct f410 address")?;
        let state = fm_client
            .actor_state(
                &addr,
                fendermint_vm_message::query::FvmQueryHeight::default(),
            )
            .await
            .with_context(|| format!("Failed to get actor state for sender {} via rpc={}", addr, rpc_url))?;
        let sequence = state.value.map(|(_, s)| s.sequence).ok_or_else(|| {
            anyhow!(
                "sender actor {} does not exist on-chain at {}. Fund/initialize this delegated \
                 address first.",
                addr,
                rpc_url
            )
        })?;

        let mf = SignedMessageFactory::new(secret_key.clone(), addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.clone().bind(mf);

        for (key, _) in &list_result.objects {
            let key_str = String::from_utf8_lossy(key).to_string();
            match bucket::delete_object(
                &mut bound_client,
                storage_path.bucket_address,
                key_str.clone(),
            )
            .await
            {
                Ok(()) => {
                    println!("✓ Deleted: {}", key_str);
                    deleted_count += 1;
                }
                Err(e) => {
                    eprintln!("⚠ Skipped {}: {:#}", key_str, e);
                    failed_count += 1;
                }
            }
        }

        // Check if there are more pages
        if list_result.next_key.is_none() {
            break;
        }

        start_key = list_result
            .next_key
            .map(|k| String::from_utf8_lossy(&k).to_string());
    }

    if deleted_count == 0 && failed_count > 0 {
        return Err(anyhow!(
            "Could not delete any of the {} matching objects (blobs may still be pending finalization)",
            failed_count
        ));
    }

    println!(
        "\nDeleted {} objects{}",
        deleted_count,
        if failed_count > 0 {
            format!(" ({} skipped — blobs pending finalization)", failed_count)
        } else {
            String::new()
        }
    );

    Ok(())
}
