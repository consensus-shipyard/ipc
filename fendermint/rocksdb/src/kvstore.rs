// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::anyhow;
use fendermint_storage::Decode;
use fendermint_storage::Encode;
use fendermint_storage::KVResult;
use fendermint_storage::KVTransaction;
use fendermint_storage::KVWritable;
use fendermint_storage::KVWrite;
use fendermint_storage::{KVError, KVRead, KVReadable, KVStore};
use rocksdb::BoundColumnFamily;
use rocksdb::ErrorKind;
use rocksdb::OptimisticTransactionDB;
use rocksdb::SnapshotWithThreadMode;
use rocksdb::Transaction;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::thread;

use crate::RocksDb;

/// Cache column families to avoid further cloning on each access.
struct ColumnFamilyCache<'a> {
    db: &'a OptimisticTransactionDB,
    cfs: RefCell<BTreeMap<String, Arc<BoundColumnFamily<'a>>>>,
}

impl<'a> ColumnFamilyCache<'a> {
    fn new(db: &'a OptimisticTransactionDB) -> Self {
        Self {
            db,
            cfs: Default::default(),
        }
    }

    /// Look up a column family and pass it to a closure.
    /// Return an error if it doesn't exist.
    fn with_cf_handle<F, T>(&self, name: &str, f: F) -> KVResult<T>
    where
        F: FnOnce(&Arc<BoundColumnFamily<'a>>) -> KVResult<T>,
    {
        let mut cfs = self.cfs.borrow_mut();
        let cf = match cfs.get(name) {
            Some(cf) => cf,
            None => match self.db.cf_handle(name) {
                None => {
                    return Err(KVError::Unexpected(
                        anyhow!("column family {name} doesn't exist").into(),
                    ))
                }
                Some(cf) => {
                    cfs.insert(name.to_owned(), cf);
                    cfs.get(name).unwrap()
                }
            },
        };
        f(cf)
    }
}

/// For reads, we can just take a snapshot of the DB.
pub struct RocksDbReadTx<'a> {
    cache: ColumnFamilyCache<'a>,
    snapshot: SnapshotWithThreadMode<'a, OptimisticTransactionDB>,
}

/// For writes, we use a transaction which we'll either commit or roll back at the end.
pub struct RocksDbWriteTx<'a> {
    cache: ColumnFamilyCache<'a>,
    tx: ManuallyDrop<Transaction<'a, OptimisticTransactionDB>>,
}

impl<S> KVReadable<S> for RocksDb
where
    S: KVStore<Repr = Vec<u8>>,
    S::Namespace: AsRef<str>,
{
    type Tx<'a> = RocksDbReadTx<'a>
    where
        Self: 'a;

    fn read(&self) -> Self::Tx<'_> {
        let snapshot = self.db.snapshot();
        RocksDbReadTx {
            cache: ColumnFamilyCache::new(&self.db),
            snapshot,
        }
    }
}

impl<S> KVWritable<S> for RocksDb
where
    S: KVStore<Repr = Vec<u8>>,
    S::Namespace: AsRef<str>,
{
    type Tx<'a> = RocksDbWriteTx<'a>
    where
        Self: 'a;

    fn write(&self) -> Self::Tx<'_> {
        RocksDbWriteTx {
            cache: ColumnFamilyCache::new(&self.db),
            tx: ManuallyDrop::new(self.db.transaction()),
        }
    }
}

impl<'a, S> KVRead<S> for RocksDbReadTx<'a>
where
    S: KVStore<Repr = Vec<u8>>,
    S::Namespace: AsRef<str>,
{
    fn get<K, V>(&self, ns: &S::Namespace, k: &K) -> KVResult<Option<V>>
    where
        S: Encode<K> + Decode<V>,
    {
        self.cache.with_cf_handle(ns.as_ref(), |cf| {
            let key = S::to_repr(k)?;

            let res = self
                .snapshot
                .get_cf(cf, key.as_ref())
                .map_err(to_kv_error)?;

            match res {
                Some(bz) => Ok(Some(S::from_repr(&bz)?)),
                None => Ok(None),
            }
        })
    }
}

impl<'a, S> KVRead<S> for RocksDbWriteTx<'a>
where
    S: KVStore<Repr = Vec<u8>>,
    S::Namespace: AsRef<str>,
{
    fn get<K, V>(&self, ns: &S::Namespace, k: &K) -> KVResult<Option<V>>
    where
        S: Encode<K> + Decode<V>,
    {
        self.cache.with_cf_handle(ns.as_ref(), |cf| {
            let key = S::to_repr(k)?;

            let res = self.tx.get_cf(cf, key.as_ref()).map_err(to_kv_error)?;

            match res {
                Some(bz) => Ok(Some(S::from_repr(&bz)?)),
                None => Ok(None),
            }
        })
    }
}

impl<'a, S> KVWrite<S> for RocksDbWriteTx<'a>
where
    S: KVStore<Repr = Vec<u8>>,
    S::Namespace: AsRef<str>,
{
    fn put<K, V>(&mut self, ns: &S::Namespace, k: &K, v: &V) -> KVResult<()>
    where
        S: Encode<K> + Encode<V>,
    {
        self.cache.with_cf_handle(ns.as_ref(), |cf| {
            let k = S::to_repr(k)?;
            let v = S::to_repr(v)?;

            self.tx
                .put_cf(cf, k.as_ref(), v.as_ref())
                .map_err(to_kv_error)?;

            Ok(())
        })
    }

    fn delete<K>(&mut self, ns: &S::Namespace, k: &K) -> KVResult<()>
    where
        S: Encode<K>,
    {
        self.cache.with_cf_handle(ns.as_ref(), |cf| {
            let k = S::to_repr(k)?;

            self.tx.delete_cf(cf, k.as_ref()).map_err(to_kv_error)?;

            Ok(())
        })
    }
}

impl<'a> KVTransaction for RocksDbWriteTx<'a> {
    fn commit(self) -> KVResult<()> {
        // This method cleans up the transaction without running the panicky destructor.
        let mut this = ManuallyDrop::new(self);
        let tx = unsafe { ManuallyDrop::take(&mut this.tx) };
        tx.commit().map_err(to_kv_error)
    }

    fn rollback(self) -> KVResult<()> {
        self.tx.rollback().map_err(to_kv_error)
    }
}

impl<'a> Drop for RocksDbWriteTx<'a> {
    fn drop(&mut self) {
        if !thread::panicking() {
            panic!("Transaction prematurely dropped. Must call `.commit()` or `.rollback()`.");
        }
    }
}

fn to_kv_error(e: rocksdb::Error) -> KVError {
    if e.kind() == ErrorKind::Busy {
        KVError::Conflict
    } else {
        KVError::Unexpected(Box::new(e))
    }
}
