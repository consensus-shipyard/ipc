// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
mod check;
mod exec;
mod genesis;
mod query;

pub use check::FvmCheckState;
pub use exec::FvmExecState;
pub use genesis::FvmGenesisState;
pub use query::FvmQueryState;

#[derive(Clone)]
pub struct ReadOnlyBlockstore<DB>(DB);

impl<DB> Blockstore for ReadOnlyBlockstore<DB>
where
    DB: Blockstore,
{
    fn get(&self, k: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        self.0.get(k)
    }

    fn put_keyed(&self, _k: &Cid, _block: &[u8]) -> anyhow::Result<()> {
        panic!("never intended to use put on the read-only blockstore")
    }
}
