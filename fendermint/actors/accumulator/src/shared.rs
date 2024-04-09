// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use fvm_ipld_amt::Amt;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{
    de::DeserializeOwned,
    ser::Serialize,
    to_vec,
    tuple::{Deserialize_tuple, Serialize_tuple},
    CborStore, DAG_CBOR,
};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const ACCUMULATOR_ACTOR_NAME: &str = "accumulator";
const BIT_WIDTH: u32 = 3;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Push = frc42_dispatch::method_hash!("Push"),
    Root = frc42_dispatch::method_hash!("Root"),
    Peaks = frc42_dispatch::method_hash!("Peaks"),
    Count = frc42_dispatch::method_hash!("Count"),
}

/// Compute the hash of a pair of CIDs.
/// The hash is the CID of a new block containing the concatenation of the two CIDs.
/// We do not include the index of the element(s) because incoming data should already be "nonced".
fn hash_pair(left: Option<&Cid>, right: Option<&Cid>) -> anyhow::Result<Cid> {
    if let (Some(left), Some(right)) = (left, right) {
        // Encode the CIDs into a binary format
        let data = to_vec(&[left, right])?;
        // Compute the CID for the block
        let mh_code = Code::Blake2b256;
        let mh = mh_code.digest(&data);
        let cid = Cid::new_v1(DAG_CBOR, mh);
        Ok(cid)
    } else {
        Err(anyhow::anyhow!("hash_pair requires two CIDs"))
    }
}

/// Compute and store the hash of a pair of CIDs.
/// The hash is the CID of a new block containing the concatenation of the two CIDs.
/// We do not include the index of the element(s) because incoming data should already be "nonced".
fn hash_and_put_pair<BS: Blockstore>(
    store: &BS,
    left: Option<&Cid>,
    right: Option<&Cid>,
) -> anyhow::Result<Cid> {
    if let (Some(left), Some(right)) = (left, right) {
        // Compute the CID for the block
        store.put_cbor(&[left, right], Code::Blake2b256)
    } else {
        Err(anyhow::anyhow!("hash_pair requires two CIDs"))
    }
}

/// Return the new peaks of the accumulator after adding `new_leaf`.
fn push<BS: Blockstore, S: DeserializeOwned + Serialize>(
    store: &BS,
    leaf_count: u64,
    peaks: &mut Amt<Cid, &BS>,
    obj: S,
) -> anyhow::Result<Cid> {
    // Create new leaf
    let leaf = store.put_cbor(&obj, Code::Blake2b256)?;
    // Push the new leaf onto the peaks
    peaks.set(peaks.count(), leaf)?;
    // Count trailing ones in the binary representation of leaf_count + 1
    // This works because adding a leaf fills the next available spot,
    // and the binary representation of this index will have trailing ones
    // where merges are required.
    let mut new_peaks = (!leaf_count).trailing_zeros();
    while new_peaks > 0 {
        // Pop the last two peaks and push their hash
        let right = peaks.delete(peaks.count() - 1)?;
        let left = peaks.delete(peaks.count() - 1)?;
        // Push the new peak onto the peaks array
        peaks.set(
            peaks.count(),
            hash_and_put_pair(store, left.as_ref(), right.as_ref())?,
        )?;
        new_peaks -= 1;
    }
    Ok(peaks.flush()?)
}

/// Collect the peaks and combine to compute the root commitment.
fn bag_peaks<BS: Blockstore>(peaks: &Amt<Cid, &BS>) -> anyhow::Result<Cid> {
    let peaks_count = peaks.count();
    if peaks_count == 0 {
        return Ok(Cid::default());
    }
    if peaks_count == 1 {
        return Ok(peaks.get(0)?.unwrap().to_owned());
    }
    let mut root = hash_pair(peaks.get(peaks_count - 2)?, peaks.get(peaks_count - 1)?)?;
    for i in 2..peaks_count {
        root = hash_pair(peaks.get(peaks_count - 1 - i)?, Some(&root))?;
    }
    Ok(root)
}

// The state represents an mmr with peaks stored in an Amt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub peaks: Cid,
    pub leaf_count: u64,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        let peaks = match Amt::<(), _>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "accumulator actor failed to create empty Amt: {}",
                    e
                ));
            }
        };
        Ok(Self {
            peaks,
            leaf_count: 0,
        })
    }

    pub fn push<BS: Blockstore, S: DeserializeOwned + Serialize>(
        &mut self,
        store: &BS,
        obj: S,
    ) -> anyhow::Result<Cid> {
        let mut amt = Amt::<Cid, &BS>::load(&self.peaks, store)?;
        self.peaks = push(store, self.leaf_count, &mut amt, obj)?;
        self.leaf_count += 1;
        // TODO:(carsonfarmer) Maybe we just want to return the root of the Amt?
        bag_peaks(&amt)
    }

    pub fn get_root<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Cid> {
        let amt = Amt::<Cid, &BS>::load(&self.peaks, store)?;
        bag_peaks(&amt)
    }

    pub fn get_peaks<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<Cid>> {
        let amt = Amt::<Cid, &BS>::load(&self.peaks, store)?;
        let mut peaks = Vec::new();
        amt.for_each(|_, cid| {
            peaks.push(cid.to_owned());
            Ok(())
        })?;
        Ok(peaks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_constructor() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let state = State::new(&store);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(
            state.peaks,
            Cid::from_str("bafy2bzacedijw74yui7otvo63nfl3hdq2vdzuy7wx2tnptwed6zml4vvz7wee")
                .unwrap()
        );
        assert_eq!(state.leaf_count, 0);
    }

    #[test]
    fn test_hash_and_put_pair() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();

        let obj1 = vec![1, 2, 3];
        let obj2 = vec![1, 2, 3];
        let cid1 = state.push(&store, obj1).expect("push1 failed");
        let cid2 = state.push(&store, obj2).expect("push2 failed");

        let pair_cid =
            hash_and_put_pair(&store, Some(&cid1), Some(&cid2)).expect("hash_and_put_pair failed");
        let merkle_node = store
            .get_cbor::<[Cid; 2]>(&pair_cid)
            .expect("get_cbor failed")
            .expect("get_cbor returned None");
        let expected = [cid1, cid2];
        assert_eq!(merkle_node, expected);
    }

    #[test]
    fn test_hash_pair() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();

        let obj1 = vec![1, 2, 3];
        let obj2 = vec![1, 2, 3];
        let cid1 = state.push(&store, obj1).expect("push1 failed");
        let cid2 = state.push(&store, obj2).expect("push2 failed");

        // Compare hash_pair and hash_and_put_pair and make sure they result in the same CID.
        let hash1 = hash_pair(Some(&cid1), Some(&cid2)).expect("hash_pair failed");
        let hash2 =
            hash_and_put_pair(&store, Some(&cid1), Some(&cid2)).expect("hash_and_put_pair failed");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_push_simple() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let obj = vec![1, 2, 3];
        assert_eq!(
            state.push(&store, obj).expect("push failed"),
            state.get_root(&store).expect("get_root failed")
        );
        assert_eq!(state.leaf_count, 1);
    }

    #[test]
    fn test_get_peaks() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        let obj = vec![1, 2, 3];
        assert!(state.push(&store, obj).is_ok());
        assert_eq!(state.leaf_count, 1);
        let peaks = state.get_peaks(&store);
        assert!(peaks.is_ok());
        let peaks = peaks.unwrap();
        assert_eq!(peaks.len(), 1);
        assert_eq!(
            peaks[0],
            Cid::from_str("bafy2bzacebltuz74cvzod3x7cx3eledj4gn5vjcer7znymoq56htf2e3cclok")
                .unwrap()
        );
    }

    #[test]
    fn test_bag_peaks() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store).unwrap();
        state.push(&store, vec![1]).unwrap();
        state.push(&store, vec![2]).unwrap();
        state.push(&store, vec![3]).unwrap();
        state.push(&store, vec![4]).unwrap();
        state.push(&store, vec![5]).unwrap();
        state.push(&store, vec![6]).unwrap();
        state.push(&store, vec![7]).unwrap();
        state.push(&store, vec![8]).unwrap();
        state.push(&store, vec![9]).unwrap();
        state.push(&store, vec![10]).unwrap();
        let root = state.push(&store, vec![11]).unwrap();
        let peaks = state.get_peaks(&store).unwrap();
        assert_eq!(peaks.len(), 3);
        assert_eq!(state.leaf_count, 11);
        assert_eq!(root, state.get_root(&store).expect("get_root failed"));
    }
}
