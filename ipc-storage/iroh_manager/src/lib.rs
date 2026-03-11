// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Result};
use iroh_blobs::hashseq::HashSeq;
use iroh_blobs::rpc::client::blobs::BlobStatus;
use iroh_blobs::Hash;
use num_traits::Zero;

mod manager;
mod node;

pub use self::manager::{connect as connect_rpc, BlobsRpcClient, IrohManager};
pub use self::node::IrohNode;
pub use quic_rpc::Connector;

pub type BlobsClient = iroh_blobs::rpc::client::blobs::Client;

/// Returns the user blob hash and size from the hash sequence.
/// The user blob hash is the first hash in the sequence.
pub async fn get_blob_hash_and_size(
    iroh: &BlobsClient,
    seq_hash: Hash,
) -> Result<(Hash, u64), anyhow::Error> {
    // Get the hash sequence status (it needs to be available)
    let status = iroh.status(seq_hash).await.map_err(|e| {
        anyhow!(
            "failed to get status for hash sequence object: {} {}",
            seq_hash,
            e
        )
    })?;
    let BlobStatus::Complete { size } = status else {
        return Err(anyhow!(
            "hash sequence object {} is not available",
            seq_hash
        ));
    };
    if size.is_zero() {
        return Err(anyhow!("hash sequence object {} has zero size", seq_hash));
    }

    // Read the bytes and create a hash sequence
    let res = iroh
        .read_to_bytes(seq_hash)
        .await
        .map_err(|e| anyhow!("failed to read hash sequence object: {} {}", seq_hash, e))?;
    let hash_seq = HashSeq::try_from(res)
        .map_err(|e| anyhow!("failed to parse hash sequence object: {} {}", seq_hash, e))?;

    // Get the user blob status at index 0 (it needs to be available)
    let blob_hash = hash_seq.get(0).ok_or_else(|| {
        anyhow!(
            "failed to get hash with index 0 from hash sequence object: {}",
            seq_hash
        )
    })?;
    let status = iroh
        .status(blob_hash)
        .await
        .map_err(|e| anyhow!("failed to read object: {} {}", blob_hash, e))?;

    // Finally, get the size from the status
    let BlobStatus::Complete { size } = status else {
        return Err(anyhow!("object {} is not available", blob_hash));
    };

    Ok((blob_hash, size))
}
