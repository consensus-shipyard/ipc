use anyhow::{anyhow, Ok};
use fendermint_rocksdb::RocksDb;
use rocksdb::{BoundColumnFamily, IteratorMode, OptimisticTransactionDB};
use std::sync::Arc;

use crate::{BlockHeight, ParentViewPayload};

/// A cache k/v implementation for storing ParentViewPayload for a specific height
/// in rocksdb with a specific namespace.
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

    // creates a new instance of the cache store for testing purposes
    #[cfg(test)]
    pub fn new_test(ns: String) -> anyhow::Result<Self> {
        use fendermint_rocksdb::RocksDbConfig;
        let dir = tempfile::Builder::new().prefix(&ns).tempdir()?;
        let db = RocksDb::open(dir.path().join("rocksdb"), &RocksDbConfig::default())?;
        let _ = db.new_cf_handle(&ns)?;
        Ok(Self { db: db.db, ns })
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
    fn put(
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

    pub fn delete_key_below(&self, height: BlockHeight) -> anyhow::Result<()> {
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

    pub fn size(&self) -> anyhow::Result<usize> {
        let mut count = 0;
        let iter = self.db.iterator_cf(&self.cf()?, IteratorMode::Start);
        for _ in iter {
            count += 1;
        }

        Ok(count)
    }

    pub fn upper_bound(&self) -> anyhow::Result<Option<BlockHeight>> {
        let mut iter = self.db.iterator_cf(&self.cf()?, IteratorMode::End);
        if let Some(item) = iter.next() {
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
        tracing::info!("STORE appending block at height {}", height);

        let expected_next_key = if let Some(upper) = self.upper_bound()? {
            upper + 1
        } else {
            self.put(height, Some(block))?;
            return Ok(());
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

#[cfg(test)]
mod tests {
    use crate::BlockHeight;
    use crate::CacheStore;
    use crate::ParentViewPayload;

    fn build_payload(height: BlockHeight) -> ParentViewPayload {
        let mut p = ParentViewPayload::default();
        p.0 = height.to_be_bytes().to_vec();
        p
    }

    #[test]
    fn insert_works() {
        let cache_store = CacheStore::new_test("test".to_string()).unwrap();
        for height in 9..100 {
            cache_store
                .append(height, Some(build_payload(height)))
                .unwrap();
        }

        for height in 9..100 {
            let value = cache_store.get_value(height).unwrap().unwrap().unwrap();
            let cache_height = BlockHeight::from_be_bytes(value.0[0..8].try_into().unwrap());
            assert_eq!(height, cache_height);
        }

        assert!(cache_store.get_value(100).unwrap().is_none());
        assert_eq!(cache_store.lower_bound().unwrap(), Some(9));
        assert_eq!(cache_store.upper_bound().unwrap(), Some(99));
    }

    #[test]
    fn delete_works() {
        let cache_store = CacheStore::new_test("test".to_string()).unwrap();

        for height in 0..100 {
            cache_store
                .append(height, Some(build_payload(height)))
                .unwrap();
        }

        cache_store.delete_key_below(10).unwrap();
        assert!(cache_store.size().unwrap() == 90);
        assert_eq!(cache_store.lower_bound().unwrap(), Some(10));

        cache_store.delete_all().unwrap();
        assert!(cache_store.size().unwrap() == 0);
        assert_eq!(cache_store.lower_bound().unwrap(), None);
    }
}
