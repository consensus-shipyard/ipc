// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use rocksdb::WriteBatchWithTransaction;

use crate::RocksDb;

impl Blockstore for RocksDb {
    fn get(&self, k: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(self.read(k.to_bytes())?)
    }

    fn put_keyed(&self, k: &Cid, block: &[u8]) -> anyhow::Result<()> {
        Ok(self.write(k.to_bytes(), block)?)
    }

    fn put_many_keyed<D, I>(&self, blocks: I) -> anyhow::Result<()>
    where
        Self: Sized,
        D: AsRef<[u8]>,
        I: IntoIterator<Item = (Cid, D)>,
    {
        let mut batch = WriteBatchWithTransaction::<true>::default();
        for (cid, v) in blocks.into_iter() {
            let k = cid.to_bytes();
            let v = v.as_ref();
            batch.put(k, v);
        }
        // This function is used in `fvm_ipld_car::load_car`
        // It reduces time cost of loading mainnet snapshot
        // by ~10% by not writing to WAL(write ahead log).
        // Ok(self.db.write_without_wal(batch)?)

        // For some reason with the `write_without_wal` version if I restart the application
        // it doesn't find the manifest root.
        Ok(self.db.write(batch)?)
    }
}
