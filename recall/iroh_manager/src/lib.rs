// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::net::ToSocketAddrs;

use anyhow::anyhow;
use iroh::blobs::hashseq::HashSeq;
use iroh::blobs::Hash;
use iroh::client::blobs::BlobStatus;
use iroh::client::Iroh;
use num_traits::Zero;

/// Helper for managing Iroh connections.
#[derive(Clone, Debug)]
pub struct IrohManager {
    addr: Option<String>,
    client: Option<Iroh>,
}

impl IrohManager {
    /// Returns a manager for the address.
    pub fn from_addr(addr: Option<String>) -> IrohManager {
        Self { addr, client: None }
    }

    /// Returns the Iroh client.
    /// The underlying client will be created if it does not exist.  
    pub async fn client(&mut self) -> anyhow::Result<Iroh> {
        if let Some(c) = self.client.clone() {
            return Ok(c);
        }
        if let Some(addr) = self.addr.clone() {
            let addr = addr.to_socket_addrs()?.next().ok_or(anyhow!(
                "failed to convert iroh node address to a socket address"
            ))?;
            match Iroh::connect_addr(addr).await {
                Ok(client) => {
                    self.client = Some(client.clone());
                    Ok(client)
                }
                Err(e) => Err(e),
            }
        } else {
            Err(anyhow!("iroh node address is not configured"))
        }
    }
}

/// Returns the user blob hash and size from the hash sequence.
/// The user blob hash is the first hash in the sequence.
pub async fn get_blob_hash_and_size(
    iroh: &Iroh,
    seq_hash: Hash,
) -> Result<(Hash, u64), anyhow::Error> {
    // Get the hash sequence status (it needs to be available)
    let status = iroh.blobs().status(seq_hash).await.map_err(|e| {
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
        .blobs()
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
        .blobs()
        .status(blob_hash)
        .await
        .map_err(|e| anyhow!("failed to read object: {} {}", blob_hash, e))?;

    // Finally, get the size from the status
    let BlobStatus::Complete { size } = status else {
        return Err(anyhow!("object {} is not available", blob_hash));
    };

    Ok((blob_hash, size))
}
