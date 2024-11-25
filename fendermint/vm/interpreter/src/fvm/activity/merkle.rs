// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use ipc_actors_abis::checkpointing_facet::ValidatorData;
use ipc_observability::lazy_static;
use merkle_tree_rs::format::Raw;
use merkle_tree_rs::standard::StandardMerkleTree;

pub type Hash = ethers::types::H256;

lazy_static!(
    /// ABI types of the Merkle tree which contains validator addresses and their voting power.
    pub static ref VALIDATOR_SUMMARY_FIELDS: Vec<String> = vec!["address".to_owned(), "uint64".to_owned()];
);

/// The merkle tree based proof verification to interact with solidity contracts
pub(crate) struct MerkleProofGen {
    tree: StandardMerkleTree<Raw>,
}

impl MerkleProofGen {
    pub fn pack_validator(v: &ValidatorData) -> Vec<String> {
        vec![format!("{:?}", v.validator), v.blocks_committed.to_string()]
    }

    pub fn root(&self) -> Hash {
        self.tree.root()
    }

    pub fn new(values: &[ValidatorData]) -> anyhow::Result<Self> {
        let values = values.iter().map(Self::pack_validator).collect::<Vec<_>>();

        let tree = StandardMerkleTree::of(&values, &VALIDATOR_SUMMARY_FIELDS)
            .context("failed to construct Merkle tree")?;
        Ok(MerkleProofGen { tree })
    }
}
