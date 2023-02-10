// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::{Codec, KVCollection, KVRead, KVReadable, KVStore, KVTransaction, KVWritable, KVWrite};
use quickcheck::{Arbitrary, Gen};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::thread;

/// We'll see how this works out. We would have to wrap any KVStore
/// with something that can handle strings as namespaces.
type TestNamespace = &'static str;

/// Test operations on some collections with known types,
/// so we can have the simplest possible model implementation.
#[derive(Clone, Debug)]
enum TestOpKV<K, V> {
    Get(K),
    Put(K, V),
    Del(K),
}

#[derive(Clone, Debug)]
enum TestOpNs {
    S2I(TestNamespace, TestOpKV<String, u8>),
    I2S(TestNamespace, TestOpKV<u8, String>),
    Rollback,
}

#[derive(Clone, Debug)]
pub struct TestData {
    ops: Vec<TestOpNs>,
}

/// Generate commands from a limited set of keys so there's a
/// high probability that we get/delete what we put earlier.
impl Arbitrary for TestOpNs {
    fn arbitrary(g: &mut Gen) -> Self {
        use TestOpKV::*;
        use TestOpNs::*;
        match u8::arbitrary(g) % 100 {
            i if i < 49 => {
                let ns = g.choose(&["spam", "eggs"]).unwrap();
                let k = *g.choose(&["foo", "bar"]).unwrap();
                match u8::arbitrary(g) % 10 {
                    i if i < 3 => S2I(ns, Get(k.to_owned())),
                    i if i < 9 => S2I(ns, Put(k.to_owned(), Arbitrary::arbitrary(g))),
                    _ => S2I(ns, Del(k.to_owned())),
                }
            }
            i if i < 98 => {
                let ns = g.choose(&["fizz", "buzz"]).unwrap();
                let k = u8::arbitrary(g) % 2;
                match u8::arbitrary(g) % 10 {
                    i if i < 3 => I2S(ns, Get(k)),
                    i if i < 9 => {
                        let sz = u8::arbitrary(g) % 5;
                        let s = (0..sz).map(|_| char::arbitrary(g)).collect();
                        I2S(ns, Put(k, s))
                    }
                    _ => I2S(ns, Del(k)),
                }
            }
            _ => Rollback,
        }
    }
}

impl Arbitrary for TestData {
    fn arbitrary(g: &mut Gen) -> Self {
        TestData {
            ops: Arbitrary::arbitrary(g),
        }
    }
}

/// Test data for multiple transactions, interspersed.
#[derive(Clone, Debug)]
pub struct TestDataMulti<const N: usize> {
    ops: Vec<(usize, TestOpNs)>,
}

impl<const N: usize> Arbitrary for TestDataMulti<N> {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut ops = Vec::new();
        for i in 0..N {
            let data = TestData::arbitrary(g);
            ops.extend(data.ops.into_iter().map(|op| (i32::arbitrary(g), i, op)));
        }
        ops.sort_by_key(|(r, _, _)| *r);

        TestDataMulti {
            ops: ops.into_iter().map(|(_, i, op)| (i, op)).collect(),
        }
    }
}

struct TestDataStore;

impl KVStore for TestDataStore {
    type Namespace = TestNamespace;
    type Repr = Vec<u8>;
}

#[derive(Default)]
struct Model {
    s2i: HashMap<TestNamespace, HashMap<String, u8>>,
    i2s: HashMap<TestNamespace, HashMap<u8, String>>,
}

struct Collections<S: KVStore> {
    s2i: HashMap<TestNamespace, KVCollection<S, String, u8>>,
    i2s: HashMap<TestNamespace, KVCollection<S, u8, String>>,
}

impl<S: KVStore> Default for Collections<S> {
    fn default() -> Self {
        Self {
            s2i: HashMap::new(),
            i2s: HashMap::new(),
        }
    }
}

impl<S> Collections<S>
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<String> + Codec<u8>,
{
    fn s2i(&mut self, ns: TestNamespace) -> &KVCollection<S, String, u8> {
        self.s2i.entry(ns).or_insert_with(|| KVCollection::new(ns))
    }

    fn i2s(&mut self, ns: TestNamespace) -> &KVCollection<S, u8, String> {
        self.i2s.entry(ns).or_insert_with(|| KVCollection::new(ns))
    }
}

/// State machine test for an implementation of a `KVWritable` using a sequence of random ops.
pub fn check_writable<S>(sut: &impl KVWritable<S>, data: TestData) -> bool
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<String> + Codec<u8>,
{
    let mut model = Model::default();
    // Creating a collection doesn't add much to the test but at least we exercise this path.
    let mut colls = Collections::<S>::default();
    // Start the transaction.
    let mut tx = sut.write();
    let mut ok = true;
    for d in data.ops {
        match d {
            TestOpNs::S2I(ns, op) => {
                let coll = colls.s2i(ns);
                if !apply_both(&mut tx, &mut model.s2i, coll, ns, op) {
                    ok = false;
                }
            }
            TestOpNs::I2S(ns, op) => {
                let coll = colls.i2s(ns);
                if !apply_both(&mut tx, &mut model.i2s, coll, ns, op) {
                    ok = false;
                }
            }
            TestOpNs::Rollback => {
                //println!("ROLLBACK");
                model = Model::default();
                tx.rollback().unwrap();
                tx = sut.write();
            }
        }
    }
    tx.rollback().unwrap();
    ok
}

/// Check that two write transactions don't see each others' changes.
///
/// This test assumes that write transactions can be executed concurrently, that
/// they don't block each other. If that's not the case don't call this test.
pub fn check_write_isolation<S>(sut: &impl KVWritable<S>, data: TestDataMulti<2>) -> bool
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<String> + Codec<u8>,
{
    let mut colls = Collections::<S>::default();
    let mut model1 = Model::default();
    let mut model2 = Model::default();
    let mut tx1 = sut.write();
    let mut tx2 = sut.write();
    let mut ok = true;
    for (i, op) in data.ops {
        let tx = if i == 0 { &mut tx1 } else { &mut tx2 };
        let model = if i == 0 { &mut model1 } else { &mut model2 };
        match op {
            TestOpNs::S2I(ns, op) => {
                let coll = colls.s2i(ns);
                if !apply_both(tx, &mut model.s2i, coll, ns, op) {
                    ok = false;
                }
            }
            TestOpNs::I2S(ns, op) => {
                let coll = colls.i2s(ns);
                if !apply_both(tx, &mut model.i2s, coll, ns, op) {
                    ok = false;
                }
            }
            TestOpNs::Rollback => {}
        }
    }
    tx1.rollback().unwrap();
    tx2.rollback().unwrap();
    ok
}

/// Check that two write transactions don't see each others' changes when executed on different threads.
pub fn check_write_isolation_concurrent<S, B>(sut: &B, data1: TestData, data2: TestData) -> bool
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<String> + Codec<u8>,
    B: KVWritable<S> + Clone + Send + 'static,
{
    let sut2 = sut.clone();
    let t = thread::spawn(move || check_writable(&sut2, data2));
    let c1 = check_writable(sut, data1);
    let c2 = t.join().unwrap();
    c1 && c2
}

/// Check that two write transactions are serializable, their effects don't get lost and aren't interspersed.
pub fn check_write_serialization_concurrent<S, B>(sut: &B, data1: TestData, data2: TestData) -> bool
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<String> + Codec<u8>,
    B: KVWritable<S> + KVReadable<S> + Clone + Send + 'static,
{
    let apply_sut = |sut: &B, data: &TestData| {
        let mut tx = sut.write();
        for op in data.ops.iter() {
            match op {
                TestOpNs::S2I(ns, TestOpKV::Put(k, v)) => tx.put(ns, k, v).unwrap(),
                TestOpNs::S2I(ns, TestOpKV::Del(k)) => tx.delete(ns, k).unwrap(),
                TestOpNs::I2S(ns, TestOpKV::Put(k, v)) => tx.put(ns, k, v).unwrap(),
                TestOpNs::I2S(ns, TestOpKV::Del(k)) => tx.delete(ns, k).unwrap(),
                _ => (),
            }
        }
        tx.prepare_and_commit().unwrap();
    };

    let sutc = sut.clone();
    let data2c = data2.clone();
    let t = thread::spawn(move || apply_sut(&sutc, &data2c));
    apply_sut(sut, &data1);
    t.join().unwrap();

    // The changes were applied in one order or the other.
    let tx = sut.read();
    let apply_model = |a: &TestData, b: &TestData| -> bool {
        let mut model = Model::default();
        // First apply all the writes
        for op in a.ops.iter().chain(b.ops.iter()).map(|op| op.clone()) {
            match op {
                TestOpNs::S2I(ns, TestOpKV::Put(k, v)) => {
                    model.s2i.entry(ns).or_default().insert(k, v);
                }
                TestOpNs::S2I(ns, TestOpKV::Del(k)) => {
                    model.s2i.entry(ns).or_default().remove(&k);
                }
                TestOpNs::I2S(ns, TestOpKV::Put(k, v)) => {
                    model.i2s.entry(ns).or_default().insert(k, v);
                }
                TestOpNs::I2S(ns, TestOpKV::Del(k)) => {
                    model.i2s.entry(ns).or_default().remove(&k);
                }
                _ => (),
            }
        }
        // Then just the reads on the final state.
        for op in a.ops.iter().chain(b.ops.iter()) {
            match op {
                TestOpNs::S2I(ns, TestOpKV::Get(k)) => {
                    let expected = tx.get::<String, u8>(&ns, k).unwrap();
                    let found = model.s2i.get(ns).and_then(|m| m.get(k)).cloned();
                    if found != expected {
                        return false;
                    }
                }
                TestOpNs::I2S(ns, TestOpKV::Get(k)) => {
                    let expected = tx.get::<u8, String>(&ns, k).unwrap();
                    let found = model.i2s.get(ns).and_then(|m| m.get(k)).cloned();
                    if found != expected {
                        return false;
                    }
                }
                _ => (),
            }
        }
        true
    };

    let ok = apply_model(&data1, &data2) || apply_model(&data2, &data1);
    drop(tx);
    ok
}

/// Check that read transactions don't see changes from write transactions.
///
/// This test assumes that read and write transactions can be executed concurrently,
/// that they don't block each other. If that's not the case don't call this test.
pub fn check_read_isolation<S, B>(sut: &B, data: TestData) -> bool
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<String> + Codec<u8>,
    B: KVWritable<S> + KVReadable<S>,
{
    let mut model = Model::default();
    let mut colls = Collections::<S>::default();
    let mut txw = sut.write();
    let mut txr = sut.read();
    let mut gets = Vec::new();
    let mut ok = true;

    for op in data.ops.clone() {
        match op {
            TestOpNs::S2I(ns, op) => {
                let coll = colls.s2i(ns);
                apply_both(&mut txw, &mut model.s2i, coll, ns, op.clone());
                match &op {
                    TestOpKV::Get(k) => {
                        if coll.get(&txr, &k).unwrap().is_some() {
                            ok = false;
                        }
                        gets.push((ns, op));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Commit the writes, but they should still not be visible to the reads that started earlier.
    txw.prepare_and_commit().unwrap();

    for (ns, op) in &gets {
        let coll = colls.s2i(ns);
        match op {
            TestOpKV::Get(k) => {
                let found = coll.get(&txr, &k).unwrap();
                if found.is_some() {
                    ok = false;
                }
            }
            _ => unreachable!(),
        }
    }

    // Finish the reads and start another read transaction. Now the writes should be visible.
    drop(txr);
    txr = sut.read();

    for (ns, op) in &gets {
        let coll = colls.s2i(ns);
        match op {
            TestOpKV::Get(k) => {
                let found = coll.get(&txr, &k).unwrap();
                let expected = model.s2i.get(ns).and_then(|m| m.get(k)).cloned();
                if found != expected {
                    ok = false;
                }
            }
            _ => unreachable!(),
        }
    }

    ok
}

/// Apply an operation on the model and the KV store, checking that the results are the same where possible.
fn apply_both<S, K, V>(
    tx: &mut impl KVWrite<S>,
    model: &mut HashMap<TestNamespace, HashMap<K, V>>,
    coll: &KVCollection<S, K, V>,
    ns: TestNamespace,
    op: TestOpKV<K, V>,
) -> bool
where
    S: KVStore<Namespace = TestNamespace> + Clone + Codec<K> + Codec<V>,
    K: Hash + Eq,
    V: Clone + PartialEq,
{
    match op {
        TestOpKV::Get(k) => {
            let found = coll.get(tx, &k).unwrap();
            let expected = model.get(ns).and_then(|m| m.get(&k)).cloned();
            //println!("GET {:?}/{:?}: {:?} ?= {:?}", ns, k, found, expected);
            if found != expected {
                return false;
            }
        }
        TestOpKV::Put(k, v) => {
            //println!("PUT {:?}/{:?}: {:?}", ns, k, v);
            coll.put(tx, &k, &v).unwrap();
            model.entry(ns).or_default().insert(k, v);
        }
        TestOpKV::Del(k) => {
            //println!("DEL {:?}/{:?}", ns, k);
            coll.delete(tx, &k).unwrap();
            model.entry(ns).or_default().remove(&k);
        }
    }
    true
}

#[cfg(feature = "inmem")]
mod im {
    use std::borrow::Cow;

    use crate::{im::InMemoryBackend, Codec, Decode, Encode, KVError, KVResult, KVStore};
    use quickcheck_macros::quickcheck;
    use serde::{de::DeserializeOwned, Serialize};

    use super::{TestData, TestDataMulti, TestNamespace};

    #[derive(Clone)]
    struct TestKVStore;

    impl KVStore for TestKVStore {
        type Namespace = TestNamespace;
        type Repr = Vec<u8>;
    }

    impl<T: Serialize> Encode<T> for TestKVStore {
        fn to_repr(value: &T) -> KVResult<Cow<Self::Repr>> {
            fvm_ipld_encoding::to_vec(value)
                .map_err(|e| KVError::Codec(Box::new(e)))
                .map(Cow::Owned)
        }
    }
    impl<T: DeserializeOwned> Decode<T> for TestKVStore {
        fn from_repr(repr: &Self::Repr) -> KVResult<T> {
            fvm_ipld_encoding::from_slice(repr).map_err(|e| KVError::Codec(Box::new(e)))
        }
    }

    impl<T> Codec<T> for TestKVStore where TestKVStore: Encode<T> + Decode<T> {}

    #[quickcheck]
    fn writable(data: TestData) -> bool {
        let backend = InMemoryBackend::<TestKVStore>::default();
        super::check_writable(&backend, data)
    }

    #[quickcheck]
    fn write_isolation(data: TestDataMulti<2>) -> bool {
        // XXX: It isn't safe to use this backend without locking writes if writes are concurrent.
        // It's just here to try the test on something.
        let backend = InMemoryBackend::<TestKVStore>::new(false);
        super::check_write_isolation(&backend, data)
    }

    #[quickcheck]
    fn write_isolation_concurrent(data1: TestData, data2: TestData) -> bool {
        let backend = InMemoryBackend::<TestKVStore>::default();
        super::check_write_isolation_concurrent(&backend, data1, data2)
    }

    #[quickcheck]
    fn write_serialization_concurrent(data1: TestData, data2: TestData) -> bool {
        let backend = InMemoryBackend::<TestKVStore>::default();
        super::check_write_serialization_concurrent(&backend, data1, data2)
    }

    #[quickcheck]
    fn read_isolation(data: TestData) -> bool {
        let backend = InMemoryBackend::<TestKVStore>::default();
        super::check_read_isolation(&backend, data)
    }
}
