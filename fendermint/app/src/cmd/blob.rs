// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use bytes::Bytes;
use fendermint_actor_blobs_shared::blobs::{BlobStatus, FinalizeBlobParams, SubscriptionId};
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_actor_blobs_shared::method::Method;
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::tx::{BoundClient, TxClient};
use fendermint_vm_core::chainid;
use fvm_shared::address::Address;
use num_traits::Zero;
use serde_json::json;

use crate::cmd;
use crate::cmd::key::read_secret_key;
use crate::cmd::rpc::print_json;
use crate::options::blob::{BlobArgs, BlobCommands};

cmd! {
  BlobArgs(self) {
    match &self.command {
      BlobCommands::FinalizeBlob {
        url,
        secret_key,
        subscriber,
        hash,
        id,
        status,
        gas_limit,
      } => {
        finalize_blob(
          url.clone(),
          secret_key,
          *subscriber,
          hash,
          id,
          *status,
          *gas_limit,
        )
        .await
      }
    }
  }
}

async fn finalize_blob(
    url: tendermint_rpc::Url,
    secret_key_path: &std::path::Path,
    subscriber: Address,
    hash_str: &str,
    id: &str,
    status: u8,
    gas_limit: u64,
) -> anyhow::Result<()> {
    // Read the secret key
    let sk = read_secret_key(secret_key_path)?;

    // Parse the hash (assume it's hex)
    let hash_bytes = if hash_str.starts_with("0x") {
        hex::decode(&hash_str[2..])
    } else {
        hex::decode(hash_str)
    }
    .context("Failed to parse blob hash as hex")?;

    if hash_bytes.len() != 32 {
        anyhow::bail!("Blob hash must be 32 bytes");
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes);
    let blob_hash = B256(hash_array);

    // Convert status to BlobStatus
    let blob_status = match status {
        2 => BlobStatus::Resolved,
        3 => BlobStatus::Failed,
        _ => anyhow::bail!("Invalid status: {}. Use 2 for Resolved, 3 for Failed", status),
    };

    // Create the finalize blob params
    let subscription_id = SubscriptionId::new(id)
        .map_err(|e| anyhow::anyhow!("Failed to create subscription ID: {}", e))?;

    let params = FinalizeBlobParams {
        source: B256([0u8; 32]), // Dummy source for POC
        subscriber,
        hash: blob_hash,
        size: 0, // Size not needed for finalization
        id: subscription_id,
        status: blob_status.clone(),
    };

    // Encode params
    let params_bytes = fvm_ipld_encoding::to_vec(&params)
        .context("Failed to encode finalize blob params")?;

    // Create client with message factory
    let client = FendermintClient::new_http(url.clone(), None)?;
    let chain_id = chainid::from_str_hashed("ipc")?; // Default chain name

    // Create message factory with sequence 0 (will be fetched automatically)
    let mf = SignedMessageFactory::new(sk, subscriber, 0, chain_id);
    let mut bound_client = client.bind(mf);

    // Create calldata: method number (8 bytes) + params
    let method_num = Method::FinalizeBlob as u64;
    let mut calldata = method_num.to_be_bytes().to_vec();
    calldata.extend_from_slice(&params_bytes);

    let gas_params = fendermint_rpc::message::GasParams {
        gas_limit,
        gas_fee_cap: Zero::zero(),
        gas_premium: Zero::zero(),
    };

    // Create the chain message using the message factory directly
    let msg = bound_client
        .message_factory_mut()
        .fevm_invoke(
            BLOBS_ACTOR_ADDR.into(),
            Bytes::from(calldata),
            Zero::zero(),
            gas_params,
        )?;

    // Broadcast with commit mode
    use fendermint_rpc::tx::TxCommit;
    let response = TxClient::<TxCommit>::perform(
        &bound_client,
        msg,
        |_| Ok(()),
    )
    .await?;

    println!("✅ Blob finalized successfully!");
    println!("   Transaction hash: {:?}", response.response.hash);
    println!("   Height: {}", response.response.height);
    println!("   Gas used: {}", response.response.deliver_tx.gas_used);
    println!("   Blob status: {:?}", blob_status.clone());

    let json = json!({
        "hash": hex::encode(response.response.hash),
        "height": response.response.height.value(),
        "gas_used": response.response.deliver_tx.gas_used,
        "status": format!("{:?}", blob_status),
    });

    print_json(&json)
}
