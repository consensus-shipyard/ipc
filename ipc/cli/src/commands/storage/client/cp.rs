// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Copy command for storage operations
//!
//! Supports three modes:
//! - local -> ipc:// : Upload to storage
//! - ipc:// -> local : Download from storage
//! - ipc:// -> ipc:// : Copy between buckets

use anyhow::{anyhow, Context, Result};
use clap::Args;
use fs_err as fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::eam::EthAddress;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;

use crate::commands::storage::{
    bucket,
    client::download_object_data,
    client_context::{resolve_rpc_url, resolve_write_context},
    config::resolve_client_gateway_url,
    gateway::GatewayClient,
    path,
};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct CopyArgs {
    /// Source path (local or ipc://)
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Destination path (local or ipc://)
    #[arg(value_name = "DEST")]
    pub dest: String,

    /// Gateway URL (overrides config and env var)
    #[arg(long)]
    pub gateway: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Recursive copy (for directories)
    #[arg(short, long)]
    pub recursive: bool,

    /// Overwrite existing objects
    #[arg(long)]
    pub overwrite: bool,
}

pub struct CopyStorage;

#[async_trait]
impl CommandLineHandler for CopyStorage {
    type Arguments = CopyArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let source_is_storage = path::is_storage_path(&args.source);
        let dest_is_storage = path::is_storage_path(&args.dest);

        match (source_is_storage, dest_is_storage) {
            (false, true) => {
                // Local -> Storage (upload)
                upload_to_storage(global, args).await
            }
            (true, false) => {
                // Storage -> Local (download)
                download_from_storage(global, args).await
            }
            (true, true) => {
                // Storage -> Storage (copy between buckets)
                copy_between_buckets(global, args).await
            }
            (false, false) => Err(anyhow!(
                "At least one path must be a storage path (ipc://...)"
            )),
        }
    }
}

/// Upload a local file to storage
async fn upload_to_storage(global: &GlobalArguments, args: &CopyArgs) -> Result<()> {
    let local_path = Path::new(&args.source);
    let storage_path = path::StoragePath::parse(&args.dest)?;

    if storage_path.is_bucket_root() {
        return Err(anyhow!(
            "Destination must include a key/path, not just bucket address"
        ));
    }

    // Handle recursive directory upload
    if local_path.is_dir() {
        if !args.recursive {
            return Err(anyhow!("Cannot copy directory without -r/--recursive flag"));
        }
        return upload_directory(global, local_path, &storage_path, args).await;
    }

    // Upload single file
    upload_file(global, local_path, &storage_path, args).await
}

/// Upload a single file to storage
async fn upload_file(
    global: &GlobalArguments,
    local_path: &Path,
    storage_path: &path::StoragePath,
    args: &CopyArgs,
) -> Result<()> {
    println!(
        "Uploading {} -> {}",
        local_path.display(),
        storage_path.to_uri()
    );

    // Read file data
    let data = fs::read(local_path)
        .with_context(|| format!("Failed to read file: {}", local_path.display()))?;

    let write_ctx = resolve_write_context(global, args.config.clone())?;
    let rpc_url = write_ctx.rpc_url;
    let secret_key = write_ctx.secret_key;

    // Get gateway URL
    let gateway_url =
        resolve_client_gateway_url(args.gateway.as_deref(), args.config.clone(), true)?;

    // Upload to gateway
    let gateway_client = GatewayClient::new(gateway_url.clone())?;
    let upload_response = gateway_client
        .upload_blob(data)
        .await
        .with_context(|| {
            format!(
                "Failed to upload blob to gateway (file={} gateway={} bucket={} key={})",
                local_path.display(),
                gateway_url,
                storage_path.bucket_address,
                storage_path.key
            )
        })?;

    println!(
        "Uploaded blob: {} ({} bytes, {} chunks)",
        upload_response.hash, upload_response.original_len, upload_response.num_chunks
    );

    // Convert hash to B256 (auto-detects hex or base32)
    let blob_hash =
        bucket::hash_to_b256(&upload_response.hash).context("Invalid blob hash from gateway")?;

    // Get node info for source ID (auto-detects hex or base32)
    let node_info = gateway_client.get_node_info().await.with_context(|| {
        format!(
            "Failed to fetch gateway node info from {} (needed for source node id)",
            gateway_url
        )
    })?;
    let source =
        bucket::hash_to_b256(&node_info.node_id).context("Invalid node ID from gateway")?;

    // Register object on-chain
    println!("Registering object on-chain...");

    let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

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

    // When --overwrite is set, remove the existing key before addObject (the EVM
    // addObject selector does not accept an overwrite flag).
    let key_exists = if args.overwrite {
        let listed = bucket::list_objects(
            &fm_client,
            storage_path.bucket_address,
            Some(storage_path.key.clone()),
            None,
            None,
            2,
        )
        .await
        .context("Failed to check if destination key exists")?;
        listed
            .objects
            .iter()
            .any(|(k, _)| k.as_slice() == storage_path.key.as_bytes())
    } else {
        false
    };

    let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
    let mut bound_client = fm_client.bind(mf);

    let mut deleted_old = false;
    if key_exists {
        match bucket::delete_object(
            &mut bound_client,
            storage_path.bucket_address,
            storage_path.key.clone(),
        )
        .await
        {
            Ok(()) => {
                deleted_old = true;
            }
            Err(e) => {
                eprintln!(
                    "⚠ Could not delete existing key before overwrite ({}); will attempt add anyway",
                    e
                );
            }
        }
    }

    let add_result = bucket::add_object(
        &mut bound_client,
        storage_path.bucket_address,
        source,
        storage_path.key.clone(),
        blob_hash,
        blob_hash, // recovery_hash (same as blob hash for now)
        upload_response.original_len as u64,
        HashMap::new(), // metadata
        upload_response.data_shards as u16,
        upload_response.parity_shards as u16,
    )
    .await;

    match add_result {
        Ok(()) => {}
        Err(e) if key_exists && !deleted_old => {
            let msg = e.to_string();
            if msg.contains("key exists") {
                println!(
                    "⚠ Object already registered on-chain with same key (blob pending finalization). \
                     Skipping re-registration."
                );
            } else {
                return Err(e).context("Failed to register object on-chain");
            }
        }
        Err(e) => {
            return Err(e).context("Failed to register object on-chain");
        }
    }

    println!(
        "✓ Successfully uploaded and registered: {}",
        storage_path.key
    );

    Ok(())
}

/// Upload a directory recursively
async fn upload_directory(
    global: &GlobalArguments,
    local_dir: &Path,
    storage_base: &path::StoragePath,
    args: &CopyArgs,
) -> Result<()> {
    println!("Uploading directory {} recursively...", local_dir.display());

    // Walk directory and upload each file
    for entry in walkdir::WalkDir::new(local_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let rel_path = entry.path().strip_prefix(local_dir)?;
            let rel_path_str = rel_path.to_string_lossy().to_string();

            // Construct storage key
            let storage_key = if storage_base.key.is_empty() {
                rel_path_str
            } else {
                format!(
                    "{}/{}",
                    storage_base.key.trim_end_matches('/'),
                    rel_path_str
                )
            };

            let file_storage_path = path::StoragePath {
                bucket_address: storage_base.bucket_address,
                key: storage_key,
            };

            // Upload file
            upload_file(global, entry.path(), &file_storage_path, args).await?;
        }
    }

    println!("✓ Directory upload complete");
    Ok(())
}

/// Download a file or directory from storage to local.
///
/// A path is treated as a directory prefix when:
/// - the key is empty (bucket root), OR
/// - the key ends with `/` (explicit directory marker), OR
/// - `-r`/`--recursive` is given (user intends to download everything under the prefix)
async fn download_from_storage(global: &GlobalArguments, args: &CopyArgs) -> Result<()> {
    let storage_path = path::StoragePath::parse(&args.source)?;
    let local_path = Path::new(&args.dest);

    let is_dir_path =
        storage_path.is_bucket_root() || storage_path.key.ends_with('/') || args.recursive;

    if is_dir_path {
        if !args.recursive {
            return Err(anyhow!(
                "Source looks like a directory prefix. Use -r/--recursive to download it."
            ));
        }
        return download_directory(global, &storage_path, local_path, args).await;
    }

    download_file(&storage_path, local_path, args).await
}

/// Download a single file from storage
async fn download_file(
    storage_path: &path::StoragePath,
    local_path: &Path,
    args: &CopyArgs,
) -> Result<()> {
    println!(
        "Downloading {} -> {}",
        storage_path.to_uri(),
        local_path.display()
    );

    let gateway_url =
        resolve_client_gateway_url(args.gateway.as_deref(), args.config.clone(), true)?;
    let rpc_url = resolve_rpc_url(args.config.clone())?;

    let gateway = GatewayClient::new(gateway_url.clone())?;
    let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

    let data = download_object_data(
        &gateway,
        &gateway_url,
        &fm_client,
        &rpc_url,
        storage_path.bucket_address,
        &storage_path.key,
    )
    .await?;

    // Create parent directories if needed
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to file
    fs::write(local_path, data)?;

    println!("✓ Downloaded {} bytes", fs::metadata(local_path)?.len());

    Ok(())
}

/// Download a directory recursively (list objects with prefix)
async fn download_directory(
    _global: &GlobalArguments,
    storage_base: &path::StoragePath,
    local_dir: &Path,
    args: &CopyArgs,
) -> Result<()> {
    println!(
        "Downloading directory {} recursively...",
        storage_base.to_uri()
    );

    let rpc_url = resolve_rpc_url(args.config.clone())?;

    let gateway_url =
        resolve_client_gateway_url(args.gateway.as_deref(), args.config.clone(), true)?;

    let gateway_client = GatewayClient::new(gateway_url.clone())?;

    // List all objects with the prefix
    let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

    let prefix = if storage_base.is_bucket_root() {
        None
    } else {
        Some(storage_base.key.clone())
    };

    let mut start_key = None;
    let mut downloaded_count = 0;

    loop {
        let list_result = bucket::list_objects(
            &fm_client,
            storage_base.bucket_address,
            prefix.clone(),
            None, // no delimiter for recursive listing
            start_key,
            100,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to list objects for recursive download (bucket={} prefix={} rpc={})",
                storage_base.bucket_address,
                prefix.as_deref().unwrap_or("<none>"),
                rpc_url
            )
        })?;

        if list_result.objects.is_empty() {
            break;
        }

        for (key_bytes, _) in &list_result.objects {
            let key_str = String::from_utf8_lossy(key_bytes).to_string();

            // Compute relative path by stripping the prefix
            let rel_path = if let Some(ref pfx) = prefix {
                key_str.strip_prefix(pfx).unwrap_or(&key_str)
            } else {
                &key_str
            };
            let rel_path = rel_path.trim_start_matches('/');

            let dest_file = local_dir.join(rel_path);

            // Create parent directories
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent)?;
            }

            // Download the object (with gateway-by-key primary + blob-by-hash fallback)
            println!("Downloading {} -> {}", key_str, dest_file.display());
            let data = download_object_data(
                &gateway_client,
                &gateway_url,
                &fm_client,
                &rpc_url,
                storage_base.bucket_address,
                &key_str,
            )
            .await
            .with_context(|| format!("Failed to download object: {}", key_str))?;

            fs::write(&dest_file, data)?;
            downloaded_count += 1;
        }

        if list_result.next_key.is_none() {
            break;
        }

        start_key = list_result
            .next_key
            .map(|k| String::from_utf8_lossy(k.as_slice()).to_string());
    }

    println!("✓ Downloaded {} files", downloaded_count);
    Ok(())
}

/// Copy an object between storage buckets
async fn copy_between_buckets(_global: &GlobalArguments, args: &CopyArgs) -> Result<()> {
    let source_path = path::StoragePath::parse(&args.source)?;
    let dest_path = path::StoragePath::parse(&args.dest)?;

    println!("Copying {} -> {}", source_path.to_uri(), dest_path.to_uri());

    // Download from source
    let temp_dir = tempfile::tempdir()?;
    let temp_file = temp_dir.path().join("temp_copy");

    let download_args = CopyArgs {
        source: args.source.clone(),
        dest: temp_file.to_string_lossy().to_string(),
        gateway: args.gateway.clone(),
        config: args.config.clone(),
        recursive: false,
        overwrite: args.overwrite,
    };

    download_file(&source_path, &temp_file, &download_args).await?;

    // Upload to destination
    let upload_args = CopyArgs {
        source: temp_file.to_string_lossy().to_string(),
        dest: args.dest.clone(),
        gateway: args.gateway.clone(),
        config: args.config.clone(),
        recursive: false,
        overwrite: args.overwrite,
    };

    upload_file(_global, &temp_file, &dest_path, &upload_args).await?;

    println!("✓ Copy complete");

    Ok(())
}
