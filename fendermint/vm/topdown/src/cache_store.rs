use anyhow::{anyhow, Ok};
use fendermint_rocksdb::RocksDb;
use rocksdb::{BoundColumnFamily, IteratorMode, OptimisticTransactionDB};
use std::sync::Arc;

use crate::{BlockHeight, ParentViewPayload};

/// A [`Blockstore`] implementation that writes to a specific namespace, not the default like above.
#[derive(Clone)]
pub struct CacheStore {
    db: Arc<OptimisticTransactionDB>,
    ns: String,
}

impl CacheStore {
    pub fn new(db: RocksDb, ns: String) -> anyhow::Result<Self> {
        // All namespaces are pre-created during open.
        if !db.has_cf_handle(&ns) {
            Err(anyhow!("namespace {ns} does not exist!"))
        } else {
            Ok(Self { db: db.db, ns })
        }
    }

    // Unfortunately there doesn't seem to be a way to avoid having to
    // clone another instance for each operation :(
    fn cf(&self) -> anyhow::Result<Arc<BoundColumnFamily>> {
        self.db
            .cf_handle(&self.ns)
            .ok_or_else(|| anyhow!("namespace {} does not exist!", self.ns))
    }
}

impl CacheStore {
    /*pub fn get(&self, height: BlockHeight) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(self.db.get_cf(&self.cf()?, height.to_be_bytes())?)
    } */

    pub fn put(
        &self,
        height: BlockHeight,
        value: Option<Option<ParentViewPayload>>,
    ) -> anyhow::Result<()> {
        let bytes = fvm_ipld_encoding::to_vec(&value)?;

        Ok(self.db.put_cf(&self.cf()?, height.to_be_bytes(), bytes)?)
    }

    pub fn delete(&self, height: BlockHeight) -> anyhow::Result<()> {
        Ok(self.db.delete_cf(&self.cf()?, height.to_be_bytes())?)
    }

    pub fn delete_all(&self) -> anyhow::Result<()> {
        let iter = self.db.iterator_cf(&self.cf()?, IteratorMode::Start);
        for item in iter {
            let (key, _) = item?;
            self.db.delete_cf(&self.cf()?, key)?;
        }

        Ok(())
    }

    pub fn delete_below(&self, height: BlockHeight) -> anyhow::Result<()> {
        let iter = self.db.iterator_cf(&self.cf()?, IteratorMode::Start);
        for item in iter {
            let (key, _) = item?;
            let key = BlockHeight::from_be_bytes(key[0..8].try_into().unwrap());
            if key < height {
                self.db.delete_cf(&self.cf()?, key.to_be_bytes())?;
            }
        }

        Ok(())
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        let mut count = 0;
        let iter = self.db.iterator_cf(&self.cf()?, IteratorMode::Start);
        for _ in iter {
            count += 1;
        }

        Ok(count)
    }

    pub fn upper_bound(&self) -> anyhow::Result<Option<BlockHeight>> {
        let iter = self.db.iterator_cf(&self.cf()?, IteratorMode::End);
        if let Some(item) = iter.last() {
            let (key, _) = item?;
            Ok(Some(BlockHeight::from_be_bytes(
                key[0..8].try_into().unwrap(),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn lower_bound(&self) -> anyhow::Result<Option<BlockHeight>> {
        let mut iter = self.db.iterator_cf(&self.cf()?, IteratorMode::Start);
        if let Some(item) = iter.next() {
            let (key, _) = item?;
            Ok(Some(BlockHeight::from_be_bytes(
                key[0..8].try_into().unwrap(),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn get_value(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Option<Option<ParentViewPayload>>> {
        let value = self.db.get_cf(&self.cf()?, height.to_be_bytes())?;
        match value {
            Some(value) => Ok(Some(fvm_ipld_encoding::from_slice(&value)?)),
            None => Ok(None),
        }
    }

    pub fn append(
        &self,
        height: BlockHeight,
        block: Option<ParentViewPayload>,
    ) -> anyhow::Result<()> {
        let expected_next_key = if let Some(upper) = self.upper_bound()? {
            upper + 1
        } else {
            0
        };

        if height != expected_next_key {
            return Err(anyhow!(
                "expected next key to be {}, but got {}",
                expected_next_key,
                height
            ));
        }

        self.put(height, Some(block))
    }
}
