// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The checkpoint proof structs

mod v1;

pub use crate::checkpoint::proof::v1::V1Proof;
use crate::lotus::LotusClient;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// The different versions of checkpoint proofs supported
#[derive(Serialize, Deserialize, Debug)]
pub enum CheckpointProof {
    V1(V1Proof),
}

/// Create the checkpoint proof
pub async fn create_proof<L: LotusClient>(
    client: &L,
    height: ChainEpoch,
) -> anyhow::Result<CheckpointProof> {
    let v1_proof = v1::create_proof(client, height).await?;
    Ok(CheckpointProof::V1(v1_proof))
}
