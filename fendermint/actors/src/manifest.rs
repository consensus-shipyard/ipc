// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::{anyhow, Context};
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::CborStore;
use std::collections::HashMap;

const CHAINMETADATA_ACTOR_NAME: &str = "chainmetadata";

/// A mapping of internal actor CIDs to their respective types.
pub struct Manifest {
    pub chainmetadata_code: Cid,
    by_id: HashMap<u32, Cid>,
    by_code: HashMap<Cid, u32>,
}

pub const CHAINMETADATA_ACTOR_CODE_ID: u32 = 1;
pub const CHAINMETADATA_ACTOR_ID: u64 = 48;

impl Manifest {
    /// Load a manifest from the blockstore.
    pub fn load<B: Blockstore>(bs: &B, root_cid: &Cid, ver: u32) -> anyhow::Result<Manifest> {
        if ver != 1 {
            return Err(anyhow!("unsupported manifest version {}", ver));
        }

        let vec: Vec<(String, Cid)> = match bs.get_cbor(root_cid)? {
            Some(vec) => vec,
            None => {
                return Err(anyhow!("cannot find manifest root cid {}", root_cid));
            }
        };

        Manifest::new(vec)
    }

    /// Construct a new manifest from actor name/cid tuples.
    pub fn new(iter: impl IntoIterator<Item = (impl Into<String>, Cid)>) -> anyhow::Result<Self> {
        let mut by_name = HashMap::new();
        let mut by_id = HashMap::new();
        let mut by_code = HashMap::new();

        // Actors are indexed sequentially, starting at 1, in the order in which they appear in the
        // manifest. 0 is reserved for "everything else" (i.e., not a builtin actor).
        for ((name, code_cid), id) in iter.into_iter().zip(1u32..) {
            let name = name.into();
            by_id.insert(id, code_cid);
            by_code.insert(code_cid, id);
            by_name.insert(name, code_cid);
        }

        let chainmetadata_code = *by_name
            .get(CHAINMETADATA_ACTOR_NAME)
            .context("manifest missing chainmetadata actor")?;

        Ok(Self {
            chainmetadata_code,
            by_id,
            by_code,
        })
    }

    /// Returns the code CID for an actor, given the actor's ID.
    pub fn code_by_id(&self, id: u32) -> Option<&Cid> {
        self.by_id.get(&id)
    }

    /// Returns the the actor code's "id" if it exists. Otherwise, returns 0.
    pub fn id_by_code(&self, code: &Cid) -> u32 {
        self.by_code.get(code).copied().unwrap_or(0)
    }

    /// Returns true id the passed code CID is the chainmetadata actor.
    pub fn is_chainmetadata_actor(&self, cid: &Cid) -> bool {
        &self.chainmetadata_code == cid
    }
}
