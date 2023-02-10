// Copyright 2022-2023 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use rocksdb::{OptimisticTransactionDB, Options, WriteBatchWithTransaction};
use std::{path::Path, sync::Arc};

mod config;
mod error;

pub use config::RocksDbConfig;
pub use error::Error;

#[derive(Clone)]
pub struct RocksDb {
    pub db: Arc<OptimisticTransactionDB>,
    options: Options,
}

/// `RocksDb` is used as the KV store. Unlike the implementation in Forest
/// which is using the `DB` type, this one is using `OptimisticTransactionDB`
/// so that we can make use of transactions that can be rolled back.
///
/// Usage:
/// ```no_run
/// use fendermint_rocksdb::{RocksDb, RocksDbConfig};
///
/// let mut db = RocksDb::open("test_db", &RocksDbConfig::default()).unwrap();
/// ```
impl RocksDb {
    pub fn open<P>(path: P, config: &RocksDbConfig) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let db_opts = config.into();
        Ok(Self {
            db: Arc::new(OptimisticTransactionDB::open(&db_opts, path)?),
            options: db_opts,
        })
    }

    pub fn get_statistics(&self) -> Option<String> {
        self.options.get_statistics()
    }

    pub fn read<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: AsRef<[u8]>,
    {
        self.db.get(key).map_err(Error::from)
    }

    pub fn write<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        Ok(self.db.put(key, value)?)
    }

    pub fn delete<K>(&self, key: K) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
    {
        Ok(self.db.delete(key)?)
    }

    pub fn exists<K>(&self, key: K) -> Result<bool, Error>
    where
        K: AsRef<[u8]>,
    {
        self.db
            .get_pinned(key)
            .map(|v| v.is_some())
            .map_err(Error::from)
    }

    pub fn bulk_write<K, V>(&self, values: &[(K, V)]) -> Result<(), Error>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let mut batch = WriteBatchWithTransaction::<true>::default();
        for (k, v) in values {
            batch.put(k, v);
        }
        Ok(self.db.write_without_wal(batch)?)
    }

    pub fn flush(&self) -> Result<(), Error> {
        self.db.flush().map_err(|e| Error::Other(e.to_string()))
    }

    /// Create a new column family, using the default options.
    ///
    /// Returns error if it already exists.
    pub fn new_cf_handle<'a>(&self, name: &'a str) -> Result<&'a str, Error> {
        if self.db.cf_handle(name).is_some() {
            return Err(Error::Other(format!(
                "column family '{name}' already exists"
            )));
        }
        self.db.create_cf(name, &self.options)?;
        Ok(name)
    }
}
