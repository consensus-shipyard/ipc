// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Gateway module for querying pending blobs from the FVM blockchain
//!
//! This module provides a polling gateway that constantly queries the blobs actor
//! for pending blobs that need to be resolved.

pub mod objects_service;

use anyhow::{Context, Result};
use bls_signatures::{aggregate, Serialize as BlsSerialize, Signature as BlsSignature};
use bytes::Bytes;
use ethers::abi::{encode as abi_encode, Token as AbiToken};
use ethers::types::U256 as EthU256;
use fendermint_actor_blobs_shared::blobs::{
    BlobStatus, GetAddedBlobsParams, SubscriptionId,
};
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_actor_blobs_shared::method::Method::{
    GetActiveOperators, GetAddedBlobs, GetOperatorInfo,
};
use fendermint_actor_blobs_shared::operators::{
    GetActiveOperatorsReturn, GetOperatorInfoParams, OperatorInfo,
};
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::tx::{BoundClient, TxClient, TxCommit};
use fendermint_vm_actor_interface::eam::EthAddress as FvmEthAddress;
use fendermint_vm_actor_interface::system;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// keccak256("finalizeBlob(bytes32,address,bytes32,uint64,string,uint8,bytes,uint128)")
const FINALIZE_BLOB_SELECTOR: [u8; 4] = [0xf6, 0x94, 0x17, 0x21];

/// Convert an FVM Address to a 20-byte Ethereum address (H160) for ABI encoding.
fn fvm_addr_to_h160(addr: &Address) -> Result<ethers::types::H160> {
    use fvm_shared::address::Payload;
    match addr.payload() {
        Payload::Delegated(d) if d.namespace() == 10 && d.subaddress().len() == 20 => {
            Ok(ethers::types::H160::from_slice(d.subaddress()))
        }
        Payload::ID(id) => {
            let eth = FvmEthAddress::from_id(*id);
            Ok(ethers::types::H160::from_slice(&eth.0))
        }
        _ => anyhow::bail!("cannot convert address {} to Ethereum H160", addr),
    }
}

/// A blob item with its hash, size, and subscribers
/// Note: We use B256 for both hash and source to match the actor's return type exactly.
/// The actor returns BlobRequest = (B256, u64, HashSet<(Address, SubscriptionId, B256)>)
pub type BlobItem = (B256, u64, HashSet<(Address, SubscriptionId, B256)>);

/// Cached operator information
struct OperatorCache {
    /// List of active operator addresses in order (for bitmap indexing)
    operators: Vec<Address>,
    /// Operator info by address (BLS pubkey, RPC URL)
    operator_info: HashMap<Address, OperatorInfo>,
    /// When this cache was last refreshed
    last_refresh: Instant,
}

impl OperatorCache {
    fn new() -> Self {
        Self {
            operators: Vec::new(),
            operator_info: HashMap::new(),
            // Set to a time far in the past to force refresh on first use
            last_refresh: Instant::now() - Duration::from_secs(3600),
        }
    }

    fn is_stale(&self, max_age: Duration) -> bool {
        self.last_refresh.elapsed() > max_age
    }
}

/// Signature collection state for a single blob
struct BlobSignatureCollection {
    /// When we first saw this blob
    first_seen: Instant,
    /// Number of collection attempts
    retry_count: u32,
    /// Signatures already collected: operator_index -> signature
    collected_signatures: HashMap<usize, BlsSignature>,
    /// Operator indices we've already attempted (to avoid re-querying)
    attempted_operators: HashSet<usize>,
    /// Blob metadata needed for finalization
    blob_metadata: BlobMetadata,
}

/// Metadata about a blob needed for finalization
#[derive(Clone)]
pub struct BlobMetadata {
    /// Subscriber address that requested the blob
    subscriber: Address,
    /// Blob size in bytes
    size: u64,
    /// Subscription ID
    subscription_id: SubscriptionId,
    /// Source Iroh node ID
    source: B256,
}

impl BlobSignatureCollection {
    fn new(metadata: BlobMetadata) -> Self {
        Self {
            first_seen: Instant::now(),
            retry_count: 0,
            collected_signatures: HashMap::new(),
            attempted_operators: HashSet::new(),
            blob_metadata: metadata,
        }
    }
}

/// Default encoding parameters (must match actor defaults).
const DEFAULT_DATA_SHARDS: usize = 4;
const DEFAULT_PARITY_SHARDS: usize = 2;
/// Must match erasure-encoding DEFAULT_MAX_CHUNK_SIZE.
const MAX_CHUNK_SIZE: u64 = 16 * 1024 * 1024;

/// Compute the set of unique assigned operator indices for a blob using the
/// deterministic assignment formula from erasure-encoding.
fn assigned_operator_indices(
    blob_hash: &B256,
    blob_size: u64,
    data_shards: usize,
    parity_shards: usize,
    num_operators: usize,
) -> HashSet<usize> {
    if num_operators == 0 {
        return HashSet::new();
    }
    let shards_per_chunk = data_shards + parity_shards;
    let num_chunks = if blob_size == 0 {
        1
    } else {
        blob_size.div_ceil(MAX_CHUNK_SIZE) as usize
    };
    // rotation_offset = blob_hash (big-endian) % num_operators
    let rotation_offset = {
        let mut remainder: u64 = 0;
        for &byte in &blob_hash.0 {
            remainder = (remainder * 256 + byte as u64) % num_operators as u64;
        }
        remainder as usize
    };
    let mut indices = HashSet::new();
    for chunk_idx in 0..num_chunks {
        for shard_idx in 0..shards_per_chunk {
            let shard_global = chunk_idx * shards_per_chunk + shard_idx;
            let node_index = (shard_global + rotation_offset) % num_operators;
            indices.insert(node_index);
        }
    }
    indices
}

/// Default gas parameters for transactions
fn default_gas_params() -> GasParams {
    GasParams {
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::from_atto(1_000_000),
        gas_premium: TokenAmount::from_atto(100_000),
    }
}

/// Gateway for polling added blobs from the chain
///
/// Uses the fendermint RPC client to query the blobs actor for newly added blobs
/// and submit finalization transactions.
pub struct BlobGateway<C> {
    client: C,
    /// How many added blobs to fetch per query
    batch_size: u32,
    /// Polling interval
    poll_interval: Duration,
    /// Cached operator data (refreshed periodically)
    operator_cache: OperatorCache,
    /// Track blobs awaiting signature collection and finalization
    pending_finalization: HashMap<B256, BlobSignatureCollection>,
}

impl<C> BlobGateway<C>
where
    C: fendermint_rpc::QueryClient + Send + Sync,
{
    /// Create a new blob gateway
    pub fn new(client: C, batch_size: u32, poll_interval: Duration) -> Self {
        Self {
            client,
            batch_size,
            poll_interval,
            operator_cache: OperatorCache::new(),
            pending_finalization: HashMap::new(),
        }
    }

    /// Query added blobs from the chain once
    pub async fn query_added_blobs(&self) -> Result<Vec<BlobItem>> {
        debug!("Querying added blobs (batch_size: {})", self.batch_size);

        // Create the query message to the blobs actor
        let params = GetAddedBlobsParams(self.batch_size);
        let params =
            RawBytes::serialize(params).context("failed to serialize GetAddedBlobsParams")?;

        let msg = Message {
            version: Default::default(),
            from: system::SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0,
            value: TokenAmount::zero(),
            method_num: GetAddedBlobs as u64,
            params,
            gas_limit: 10_000_000_000, // High gas limit for read-only query
            gas_fee_cap: TokenAmount::zero(),
            gas_premium: TokenAmount::zero(),
        };

        // Execute the query using the FendermintClient
        let response = self
            .client
            .call(msg, FvmQueryHeight::default())
            .await
            .context("failed to execute GetAddedBlobs call")?;

        if response.value.code.is_err() {
            anyhow::bail!("GetAddedBlobs query failed: {}", response.value.info);
        }

        // Decode the return data
        let return_data = fendermint_rpc::response::decode_data(&response.value.data)
            .context("failed to decode response data")?;

        let blobs = fvm_ipld_encoding::from_slice::<Vec<BlobItem>>(&return_data)
            .context("failed to decode added blobs response")?;

        info!("Found {} added blobs", blobs.len());
        Ok(blobs)
    }
}

/// Implementation for transaction-capable clients (can submit finalization transactions)
impl<C> BlobGateway<C>
where
    C: fendermint_rpc::QueryClient + BoundClient + TxClient<TxCommit> + Send + Sync,
{
    /// Main entry point: run the gateway to monitor and finalize blobs
    ///
    /// This is an alias for run_signature_collection()
    pub async fn run(&mut self) -> Result<()> {
        self.run_signature_collection().await
    }

    /// Main entry point: collect signatures and finalize blobs
    ///
    /// This monitors pending blobs, collects signatures from operators,
    /// aggregates them, and calls finalize_blob on-chain.
    pub async fn run_signature_collection(&mut self) -> Result<()> {
        info!(
            "Starting signature collection loop (interval: {:?})",
            self.poll_interval
        );

        loop {
            if let Err(e) = self.signature_collection_loop().await {
                error!("Signature collection error: {}", e);
            }

            sleep(self.poll_interval).await;
        }
    }

    async fn signature_collection_loop(&mut self) -> Result<()> {
        debug!("Starting signature collection loop iteration");

        // Step 1: Refresh operator cache if stale (every 5 minutes)
        let cache_refresh_interval = Duration::from_secs(300);
        let needs_refresh = self.operator_cache.is_stale(cache_refresh_interval);
        debug!(
            "Operator cache status: {} operators, stale: {}",
            self.operator_cache.operators.len(),
            needs_refresh
        );

        if needs_refresh {
            info!("Refreshing operator cache...");
            match self.query_active_operators().await {
                Ok(operators) => {
                    self.operator_cache.operators = operators.clone();
                    self.operator_cache.operator_info.clear();

                    // Fetch operator info for each operator
                    for operator_addr in &operators {
                        match self.get_operator_info(*operator_addr).await {
                            Ok(info) => {
                                self.operator_cache
                                    .operator_info
                                    .insert(*operator_addr, info);
                            }
                            Err(e) => {
                                warn!("Failed to get info for operator {}: {}", operator_addr, e);
                            }
                        }
                    }

                    self.operator_cache.last_refresh = Instant::now();
                    info!("Operator cache refreshed: {} operators", operators.len());
                }
                Err(e) => {
                    warn!("Failed to refresh operator cache: {}", e);
                }
            }
        }

        // Step 2: Query added blobs and track them
        match self.query_added_blobs().await {
            Ok(added_blobs) => {
                for (hash, size, sources) in added_blobs {
                    // Extract metadata from sources (pick first source)
                    if let Some((subscriber, subscription_id, source)) = sources.iter().next() {
                        // Skip if already tracked
                        if self.pending_finalization.contains_key(&hash) {
                            continue;
                        }

                        let metadata = BlobMetadata {
                            subscriber: *subscriber,
                            size,
                            subscription_id: subscription_id.clone(),
                            source: *source,
                        };

                        // Track the blob for signature collection
                        // (blob will be finalized directly from Added status)
                        self.pending_finalization
                            .insert(hash, BlobSignatureCollection::new(metadata));
                    } else {
                        warn!("Blob {} has no sources, skipping", hash);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to query added blobs: {}", e);
            }
        }

        // Step 3: Try to collect signatures for tracked blobs
        let tracked_blobs: Vec<B256> = self.pending_finalization.keys().copied().collect();

        debug!(
            "Checking {} blobs for signature collection",
            tracked_blobs.len()
        );

        for hash in tracked_blobs {
            // Get collection once and check if we should skip
            let Some(collection) = self.pending_finalization.get_mut(&hash) else {
                continue;
            };

            // Skip if we just added this blob (give operators time to download)
            // Use 10 seconds for faster testing
            let elapsed = collection.first_seen.elapsed();
            if elapsed < Duration::from_secs(10) {
                debug!(
                    "Blob {} waiting for operators to download ({:.1}s / 10s)",
                    hash,
                    elapsed.as_secs_f64()
                );
                continue;
            }

            info!(
                "Blob {} ready for signature collection (waited {:.1}s)",
                hash,
                elapsed.as_secs_f64()
            );

            // Get operators from cache
            let (operators, total_operators) = (
                self.operator_cache.operators.clone(),
                self.operator_cache.operators.len(),
            );

            if total_operators == 0 {
                debug!("No operators available, skipping signature collection");
                continue;
            }

            // Compute the set of assigned operator indices for this blob
            let assigned = assigned_operator_indices(
                &hash,
                collection.blob_metadata.size,
                DEFAULT_DATA_SHARDS,
                DEFAULT_PARITY_SHARDS,
                total_operators,
            );
            let assigned_count = assigned.len();
            let threshold = (assigned_count * 2 + 2) / 3; // Ceiling of 2/3

            // Collect signatures that aren't already attempted
            let attempted_operators = collection.attempted_operators.clone();

            // Build list of (index, operator_addr, rpc_url) for assigned operators we need to query
            let mut fetch_tasks = Vec::new();
            for (index, operator_addr) in operators.iter().enumerate() {
                // Only query operators assigned to this blob
                if !assigned.contains(&index) {
                    continue;
                }

                // Skip if already collected
                if attempted_operators.contains(&index) {
                    continue;
                }

                // Get operator RPC URL from cache - skip if not found
                let Some(operator_info) = self.operator_cache.operator_info.get(operator_addr)
                else {
                    warn!("Operator {} not found in cache, skipping", operator_addr);
                    continue;
                };

                fetch_tasks.push((index, *operator_addr, operator_info.rpc_url.clone()));
            }

            // Fetch signatures from all operators in parallel
            let fetch_futures: Vec<_> = fetch_tasks
                .into_iter()
                .map(|(index, operator_addr, rpc_url)| async move {
                    let result = Self::fetch_signature_static(&rpc_url, hash).await;
                    (index, operator_addr, result)
                })
                .collect();

            // Wait for all fetches to complete
            let fetch_results = futures::future::join_all(fetch_futures).await;

            // Collect successful signatures
            let mut new_signatures: Vec<(usize, BlsSignature)> = Vec::new();
            for (index, operator_addr, result) in fetch_results {
                match result {
                    Ok(signature) => {
                        info!(
                            "Got signature from operator {} (index {})",
                            operator_addr, index
                        );
                        new_signatures.push((index, signature));
                    }
                    Err(e) => {
                        warn!(
                            "Failed to get signature from operator {}: {}",
                            operator_addr, e
                        );
                        // Don't mark as attempted - we'll retry next iteration
                    }
                }
            }

            // Apply all collected signatures at once
            let collection = self.pending_finalization.get_mut(&hash).unwrap();
            for (index, signature) in new_signatures {
                collection.collected_signatures.insert(index, signature);
                collection.attempted_operators.insert(index);
            }

            // Get collection reference for final checks
            let num_collected = collection.collected_signatures.len();

            if num_collected >= threshold {
                // Collect signatures and build bitmap
                let sigs_vec: Vec<(usize, BlsSignature)> = collection
                    .collected_signatures
                    .iter()
                    .map(|(idx, sig)| (*idx, *sig))
                    .collect();

                let mut bitmap: u128 = 0;
                for idx in collection.collected_signatures.keys() {
                    bitmap |= 1u128 << idx;
                }

                info!(
                    "Collected {}/{} signatures for blob {} (threshold: {}, total operators: {})",
                    num_collected, assigned_count, hash, threshold, total_operators
                );

                // Get metadata before calling finalize_blob
                let metadata = collection.blob_metadata.clone();

                // Aggregate signatures
                match self.aggregate_signatures(sigs_vec) {
                    Ok(aggregated_sig) => {
                        info!("Successfully aggregated signature for blob {}", hash);
                        info!("Bitmap: 0b{:b}", bitmap);

                        // Call finalize_blob with aggregated signature and bitmap
                        match self
                            .finalize_blob(hash, &metadata, aggregated_sig, bitmap)
                            .await
                        {
                            Ok(()) => {
                                // Remove from tracking after successful finalization
                                self.pending_finalization.remove(&hash);
                                info!("Blob {} finalized on-chain and removed from tracking", hash);
                            }
                            Err(e) => {
                                warn!("Failed to finalize blob {} on-chain: {:#}", hash, e);
                                // Keep in tracking to retry later
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to aggregate signatures for {}: {}", hash, e);
                    }
                }
            } else {
                // Update retry count
                collection.retry_count += 1;

                // Give up after too many retries or too much time
                if collection.retry_count > 20
                    || collection.first_seen.elapsed() > Duration::from_secs(600)
                {
                    warn!(
                        "Giving up on blob {} after {} retries / {:?} (collected {}/{})",
                        hash,
                        collection.retry_count,
                        collection.first_seen.elapsed(),
                        num_collected,
                        threshold
                    );
                } else {
                    debug!(
                        "Blob {} progress: {}/{} signatures (threshold: {})",
                        hash, num_collected, assigned_count, threshold
                    );
                }
            }
        }

        Ok(())
    }
}

/// Additional query methods for all clients (read-only operations)
impl<C> BlobGateway<C>
where
    C: fendermint_rpc::QueryClient + Send + Sync,
{
    /// Query the list of active node operators from the chain
    pub async fn query_active_operators(&self) -> Result<Vec<Address>> {
        debug!("Querying active operators");

        let msg = Message {
            version: Default::default(),
            from: system::SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0,
            value: TokenAmount::zero(),
            method_num: GetActiveOperators as u64,
            params: RawBytes::default(),
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::zero(),
            gas_premium: TokenAmount::zero(),
        };

        let response = self
            .client
            .call(msg, FvmQueryHeight::default())
            .await
            .context("failed to execute GetActiveOperators call")?;

        if response.value.code.is_err() {
            anyhow::bail!("GetActiveOperators query failed: {}", response.value.info);
        }

        let return_data = fendermint_rpc::response::decode_data(&response.value.data)
            .context("failed to decode response data")?;

        let result = fvm_ipld_encoding::from_slice::<GetActiveOperatorsReturn>(&return_data)
            .context("failed to decode active operators response")?;

        info!("Found {} active operators", result.operators.len());
        Ok(result.operators)
    }

    /// Get operator info by address
    pub async fn get_operator_info(&self, address: Address) -> Result<OperatorInfo> {
        debug!("Querying operator info for {}", address);

        let params = GetOperatorInfoParams { address };
        let params =
            RawBytes::serialize(params).context("failed to serialize GetOperatorInfoParams")?;

        let msg = Message {
            version: Default::default(),
            from: system::SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0,
            value: TokenAmount::zero(),
            method_num: GetOperatorInfo as u64,
            params,
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::zero(),
            gas_premium: TokenAmount::zero(),
        };

        let response = self
            .client
            .call(msg, FvmQueryHeight::default())
            .await
            .context("failed to execute GetOperatorInfo call")?;

        if response.value.code.is_err() {
            anyhow::bail!("GetOperatorInfo query failed: {}", response.value.info);
        }

        let return_data = fendermint_rpc::response::decode_data(&response.value.data)
            .context("failed to decode response data")?;

        let result = fvm_ipld_encoding::from_slice::<Option<OperatorInfo>>(&return_data)
            .context("failed to decode operator info response")?;

        result.ok_or_else(|| anyhow::anyhow!("Operator not found"))
    }

    /// Collect signatures from all active operators for a given blob hash
    ///
    /// Returns a tuple of (signatures_with_index, bitmap) where:
    /// - signatures_with_index: Vec of (operator_index, BLS signature)
    /// - bitmap: u128 bitmap indicating which operators signed
    pub async fn collect_signatures(
        &self,
        blob_hash: B256,
    ) -> Result<(Vec<(usize, BlsSignature)>, u128)> {
        info!("Collecting signatures for blob {}", blob_hash);

        // Get active operators
        let operators = self.query_active_operators().await?;

        if operators.is_empty() {
            anyhow::bail!("No active operators found");
        }

        let mut signatures = Vec::new();
        let mut bitmap: u128 = 0;

        // Query each operator's RPC for the signature
        for (index, operator_addr) in operators.iter().enumerate() {
            match self.get_operator_info(*operator_addr).await {
                Ok(operator_info) => {
                    match self
                        .fetch_signature_from_operator(&operator_info.rpc_url, blob_hash)
                        .await
                    {
                        Ok(signature) => {
                            signatures.push((index, signature));
                            bitmap |= 1u128 << index;
                            info!(
                                "Got signature from operator {} (index {})",
                                operator_addr, index
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to get signature from operator {} ({}): {}",
                                operator_addr, operator_info.rpc_url, e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get info for operator {}: {}", operator_addr, e);
                }
            }
        }

        if signatures.is_empty() {
            anyhow::bail!("No signatures collected from any operator");
        }

        info!(
            "Collected {} signatures out of {} operators",
            signatures.len(),
            operators.len()
        );

        Ok((signatures, bitmap))
    }

    /// Fetch a signature from an operator's RPC endpoint
    async fn fetch_signature_from_operator(
        &self,
        rpc_url: &str,
        blob_hash: B256,
    ) -> Result<BlsSignature> {
        Self::fetch_signature_static(rpc_url, blob_hash).await
    }

    /// Static version of fetch_signature_from_operator for parallel execution
    async fn fetch_signature_static(rpc_url: &str, blob_hash: B256) -> Result<BlsSignature> {
        let url = format!("{}/signature/{}", rpc_url, blob_hash);
        debug!("Fetching signature from {}", url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("failed to create HTTP client")?;

        let response = client
            .get(&url)
            .send()
            .await
            .context("failed to send HTTP request")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP request failed with status: {}", response.status());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .context("failed to parse JSON response")?;

        let signature_hex = json["signature"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'signature' field in response"))?;

        let signature_bytes =
            hex::decode(signature_hex).context("failed to decode signature hex")?;

        let signature = BlsSignature::from_bytes(&signature_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse BLS signature: {:?}", e))?;

        Ok(signature)
    }

    /// Aggregate BLS signatures into a single signature
    pub fn aggregate_signatures(
        &self,
        signatures: Vec<(usize, BlsSignature)>,
    ) -> Result<BlsSignature> {
        if signatures.is_empty() {
            anyhow::bail!("Cannot aggregate empty signature list");
        }

        info!("Aggregating {} signatures", signatures.len());

        let sigs: Vec<BlsSignature> = signatures.into_iter().map(|(_, sig)| sig).collect();
        let aggregated = aggregate(&sigs)
            .map_err(|e| anyhow::anyhow!("Failed to aggregate signatures: {:?}", e))?;

        Ok(aggregated)
    }
}

/// Transaction methods for clients that can submit transactions
impl<C> BlobGateway<C>
where
    C: fendermint_rpc::QueryClient + BoundClient + TxClient<TxCommit> + Send + Sync,
{
    /// Call finalize_blob on-chain via the EVM facade (InvokeContract).
    ///
    /// This encodes the parameters as ABI calldata and uses `fevm_invoke` so that
    /// the transaction works with both f1 (native) and f410 (delegated) sender addresses.
    pub async fn finalize_blob(
        &mut self,
        blob_hash: B256,
        metadata: &BlobMetadata,
        aggregated_signature: BlsSignature,
        signer_bitmap: u128,
    ) -> Result<()> {
        info!("Finalizing blob {} on-chain", blob_hash);

        let signature_bytes = aggregated_signature.as_bytes().to_vec();

        let subscriber_h160 = fvm_addr_to_h160(&metadata.subscriber)
            .context("failed to convert subscriber to Ethereum address")?;

        let status_u8: u8 = blob_status_to_u8(BlobStatus::Resolved);

        let calldata = encode_finalize_blob_calldata(
            &metadata.source,
            subscriber_h160,
            &blob_hash,
            metadata.size,
            &String::from(metadata.subscription_id.clone()),
            status_u8,
            &signature_bytes,
            signer_bitmap,
        );

        debug!("FinalizeBlob ABI calldata ({} bytes)", calldata.len());

        let gas = default_gas_params();
        debug!(
            "Gas params: limit={}, fee_cap={}, premium={}",
            gas.gas_limit, gas.gas_fee_cap, gas.gas_premium
        );

        let res = match TxClient::<TxCommit>::fevm_invoke(
            &mut self.client,
            BLOBS_ACTOR_ADDR,
            calldata,
            TokenAmount::zero(),
            gas,
        )
        .await
        {
            Ok(res) => res,
            Err(e) => {
                error!(
                    "FinalizeBlob fevm_invoke failed for {}: {:?}",
                    blob_hash, e
                );
                return Err(e).context("failed to send FinalizeBlob transaction");
            }
        };

        if res.response.check_tx.code.is_err() {
            anyhow::bail!(
                "FinalizeBlob check_tx failed (code {:?}): log={} info={}",
                res.response.check_tx.code,
                res.response.check_tx.log,
                res.response.check_tx.info,
            );
        }

        if res.response.deliver_tx.code.is_err() {
            anyhow::bail!(
                "FinalizeBlob deliver_tx failed (code {:?}): log={} info={}",
                res.response.deliver_tx.code,
                res.response.deliver_tx.log,
                res.response.deliver_tx.info,
            );
        }

        info!(
            "Successfully finalized blob {} on-chain (tx: {})",
            blob_hash, res.response.hash
        );
        Ok(())
    }
}

fn blob_status_to_u8(status: BlobStatus) -> u8 {
    match status {
        BlobStatus::Added => 0,
        BlobStatus::Pending => 1,
        BlobStatus::Resolved => 2,
        BlobStatus::Failed => 3,
    }
}

fn encode_finalize_blob_calldata(
    source: &B256,
    subscriber: ethers::types::H160,
    blob_hash: &B256,
    size: u64,
    subscription_id: &str,
    status: u8,
    aggregated_signature: &[u8],
    signer_bitmap: u128,
) -> Bytes {
    let args = abi_encode(&[
        AbiToken::FixedBytes(source.0.to_vec()),
        AbiToken::Address(subscriber),
        AbiToken::FixedBytes(blob_hash.0.to_vec()),
        AbiToken::Uint(EthU256::from(size)),
        AbiToken::String(subscription_id.to_string()),
        AbiToken::Uint(EthU256::from(status)),
        AbiToken::Bytes(aggregated_signature.to_vec()),
        AbiToken::Uint(EthU256::from(signer_bitmap)),
    ]);
    let mut calldata = Vec::with_capacity(4 + args.len());
    calldata.extend_from_slice(&FINALIZE_BLOB_SELECTOR);
    calldata.extend_from_slice(&args);
    Bytes::from(calldata)
}
