// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use fendermint_actor_machine::{
    Kind, MachineAddress, MachineState, GET_ADDRESS_METHOD, GET_METADATA_METHOD, INIT_METHOD,
    METHOD_CONSTRUCTOR,
};
use fil_actors_runtime::ActorError;
use fvm_ipld_amt::Amt;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{strict_bytes, to_vec, tuple::*, CborStore, DAG_CBOR};
use fvm_shared::address::Address;
use num_derive::FromPrimitive;
use serde::{de::DeserializeOwned, Serialize};

pub const TIMEHUB_ACTOR_NAME: &str = "timehub";
const BIT_WIDTH: u32 = 3;

fn state_error(e: fvm_ipld_amt::Error) -> ActorError {
    ActorError::illegal_state(e.to_string())
}

fn store_error(e: anyhow::Error) -> ActorError {
    ActorError::illegal_state(e.to_string())
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Init = INIT_METHOD,
    GetAddress = GET_ADDRESS_METHOD,
    GetMetadata = GET_METADATA_METHOD,
    Push = frc42_dispatch::method_hash!("Push"),
    Get = frc42_dispatch::method_hash!("Get"),
    Root = frc42_dispatch::method_hash!("Root"),
    Peaks = frc42_dispatch::method_hash!("Peaks"),
    Count = frc42_dispatch::method_hash!("Count"),
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct PushParams {
    /// Bytes of a CID to add.
    #[serde(with = "strict_bytes")]
    pub cid_bytes: Vec<u8>,
    /// Account address that initiated the call.
    pub from: Address,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct PushReturn {
    /// The new root of the timehub MMR after the object was pushed into it.
    pub root: Cid,
    /// The index of the object that was just pushed into the timehub.
    pub index: u64,
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Leaf {
    /// Timestamp of the witness in seconds since the UNIX epoch
    pub timestamp: u64,
    /// Witnessed CID
    pub witnessed: Cid,
}

/// Compute the hash of a pair of CIDs.
/// The hash is the CID of a new block containing the concatenation of the two CIDs.
/// We do not include the index of the element(s) because incoming data should already be "nonced".
fn hash_pair(left: Option<&Cid>, right: Option<&Cid>) -> anyhow::Result<Cid, ActorError> {
    if let (Some(left), Some(right)) = (left, right) {
        // Encode the CIDs into a binary format
        let data = to_vec(&[left, right])?;
        // Compute the CID for the block
        let mh_code = Code::Blake2b256;
        let mh = mh_code.digest(&data);
        let cid = Cid::new_v1(DAG_CBOR, mh);
        Ok(cid)
    } else {
        Err(ActorError::illegal_argument(
            "hash_pair requires two CIDs".into(),
        ))
    }
}

/// Compute and store the hash of a pair of CIDs.
/// The hash is the CID of a new block containing the concatenation of the two CIDs.
/// We do not include the index of the element(s) because incoming data should already be "nonced".
fn hash_and_put_pair<BS: Blockstore>(
    store: &BS,
    left: Option<&Cid>,
    right: Option<&Cid>,
) -> anyhow::Result<Cid, ActorError> {
    if let (Some(left), Some(right)) = (left, right) {
        // Compute the CID for the block
        store
            .put_cbor(&[left, right], Code::Blake2b256)
            .map_err(store_error)
    } else {
        Err(ActorError::illegal_argument(
            "hash_pair requires two CIDs".into(),
        ))
    }
}

/// Return the new peaks of the timehub after adding `new_leaf`.
fn push<BS: Blockstore, S: DeserializeOwned + Serialize>(
    store: &BS,
    leaf_count: u64,
    peaks: &mut Amt<Cid, &BS>,
    obj: S,
) -> anyhow::Result<Cid, ActorError> {
    // Create new leaf
    let leaf = store
        .put_cbor(&obj, Code::Blake2b256)
        .map_err(store_error)?;
    // Push the new leaf onto the peaks
    peaks.set(peaks.count(), leaf).map_err(state_error)?;
    // Count trailing ones in binary representation of the previous leaf_count
    // This works because adding a leaf fills the next available spot,
    // and the binary representation of this index will have trailing ones
    // where merges are required.
    let mut new_peaks = (!leaf_count).trailing_zeros();
    while new_peaks > 0 {
        // Pop the last two peaks and push their hash
        let right = peaks.delete(peaks.count() - 1).map_err(state_error)?;
        let left = peaks.delete(peaks.count() - 1).map_err(state_error)?;
        // Push the new peak onto the peak array
        peaks
            .set(
                peaks.count(),
                hash_and_put_pair(store, left.as_ref(), right.as_ref())?,
            )
            .map_err(state_error)?;
        new_peaks -= 1;
    }
    peaks.flush().map_err(state_error)
}

/// Collect the peaks and combine to compute the root commitment.
fn bag_peaks<BS: Blockstore>(peaks: &Amt<Cid, &BS>) -> anyhow::Result<Cid, ActorError> {
    let peaks_count = peaks.count();
    // Handle special cases where we have no peaks or only one peak
    if peaks_count == 0 {
        return Ok(Cid::default());
    }
    // If there is only one leaf element, we simply "promote" that to the root peak
    if peaks_count == 1 {
        return Ok(peaks.get(0).map_err(state_error)?.unwrap().to_owned());
    }
    // Walk backward through the peaks, combining them pairwise
    let mut root = hash_pair(
        peaks.get(peaks_count - 2).map_err(state_error)?,
        peaks.get(peaks_count - 1).map_err(state_error)?,
    )?;
    for i in 2..peaks_count {
        root = hash_pair(
            peaks.get(peaks_count - 1 - i).map_err(state_error)?,
            Some(&root),
        )?;
    }
    Ok(root)
}

/// Given the size of the MMR and an index into the MMR, returns a tuple where the first element
/// represents the path through the subtree that the leaf node lives in.
/// The second element represents the index of the peak containing the subtree that the leaf node
/// lives in.
fn path_for_eigen_root(leaf_index: u64, leaf_count: u64) -> anyhow::Result<Option<(u64, u64)>> {
    // Ensure `leaf_index` is within bounds.
    if leaf_index >= leaf_count {
        return Ok(None);
    }
    // XOR turns matching bits into zeros and differing bits into ones, so to determine when
    // the two "paths" converge, we simply look for the most significant 1 bit...
    let diff = leaf_index ^ leaf_count;
    // ...and then merge height of `leaf_index` and `leaf_count` occurs at ⌊log2(x ⊕ y)⌋
    let eigentree_height = u64::BITS - diff.leading_zeros() - 1;
    let merge_height = 1 << eigentree_height;
    // Compute a bitmask (all the lower bits set to 1)
    let bitmask = merge_height - 1;
    // The Hamming weight of leaf_count is the number of eigentrees in the structure.
    let eigentree_count = leaf_count.count_ones();
    // Isolates the lower bits of leaf_count up to the merge_height, and count the one-bits.
    // This is essentially the offset to the eigentree containing leaf_index
    let offset = (leaf_count & bitmask).count_ones();
    // The index is simply the total eigentree count minus the offset (minus one)
    let eigen_index = eigentree_count - offset - 1;
    // Now that we have the offset, we need to determine the path within the local eigentree
    let local_offset = leaf_index & bitmask;
    // The local_index is the local_offset plus the merge_height for the local eigentree
    let local_path = local_offset + merge_height;
    Ok(Some((local_path, eigen_index as u64)))
}

/// Returns None when the index doesn't point to a leaf.
/// If the index is valid, it will return a value or error.
fn get_at<BS: Blockstore, S: DeserializeOwned + Serialize>(
    store: &BS,
    leaf_index: u64,
    leaf_count: u64,
    peaks: &Amt<Cid, &BS>,
) -> anyhow::Result<Option<S>> {
    let (path, eigen_index) = match path_for_eigen_root(leaf_index, leaf_count)? {
        None => return Ok(None),
        Some(res) => res,
    };
    let cid = match peaks.get(eigen_index)? {
        Some(cid) => cid,
        None => return Ok(None),
    };
    // Special case where eigentree has a height of one
    if path == 1 {
        return Ok(Some(store.get_cbor::<S>(cid)?.ok_or_else(|| {
            anyhow::anyhow!("failed to get leaf for cid {}", cid)
        })?));
    }

    let mut pair = match store.get_cbor::<[Cid; 2]>(cid)? {
        Some(value) => value,
        None => anyhow::bail!("failed to get eigentree root node for cid {}", cid),
    };

    let leading_zeros = path.leading_zeros();
    let significant_bits = 64 - leading_zeros;

    // Iterate over each bit from the most significant bit to the least
    for i in 1..(significant_bits - 1) {
        let bit = ((path >> (significant_bits - i - 1)) & 1) as usize;
        let cid = &pair[bit];
        pair = store.get_cbor(cid)?.ok_or_else(|| {
            anyhow::anyhow!("failed to get eigentree intermediate node for cid {}", cid)
        })?;
    }

    let bit = (path & 1) as usize;
    let cid = &pair[bit];
    let leaf = store
        .get_cbor::<S>(cid)?
        .ok_or_else(|| anyhow::anyhow!("failed to get leaf for cid {}", cid))?;

    Ok(Some(leaf))
}

/// The state represents an MMR with peaks stored in an AMT
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The machine address set by the init actor.
    pub address: MachineAddress,
    /// The machine rubust owner address.
    pub owner: Address,
    /// Root of the AMT that is storing the peaks of the MMR
    pub peaks: Cid,
    /// Number of leaf nodes in the timehub MMR.
    pub leaf_count: u64,
    /// User-defined metadata.
    pub metadata: HashMap<String, String>,
}

impl MachineState for State {
    fn new<BS: Blockstore>(
        store: &BS,
        owner: Address,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<Self, ActorError> {
        let peaks = match Amt::<(), _>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(ActorError::illegal_state(format!(
                    "timehub actor failed to create empty Amt: {}",
                    e
                )));
            }
        };
        Ok(Self {
            address: Default::default(),
            owner,
            peaks,
            leaf_count: 0,
            metadata,
        })
    }

    fn init(&mut self, address: Address) -> anyhow::Result<(), ActorError> {
        self.address.set(address)
    }

    fn address(&self) -> MachineAddress {
        self.address.clone()
    }

    fn kind(&self) -> Kind {
        Kind::Timehub
    }

    fn owner(&self) -> Address {
        self.owner
    }

    fn metadata(&self) -> HashMap<String, String> {
        self.metadata.clone()
    }
}

impl State {
    pub fn peak_count(&self) -> u32 {
        self.leaf_count.count_ones()
    }

    pub fn leaf_count(&self) -> u64 {
        self.leaf_count
    }

    pub fn push<BS: Blockstore, S: DeserializeOwned + Serialize>(
        &mut self,
        store: &BS,
        obj: S,
    ) -> anyhow::Result<PushReturn, ActorError> {
        let mut amt = Amt::<Cid, &BS>::load(&self.peaks, store).map_err(state_error)?;
        self.peaks = push(store, self.leaf_count, &mut amt, obj)?;
        self.leaf_count += 1;

        let root = bag_peaks(&amt)?;
        Ok(PushReturn {
            root,
            index: self.leaf_count - 1,
        })
    }

    pub fn get_root<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Cid, ActorError> {
        let amt = Amt::<Cid, &BS>::load(&self.peaks, store).map_err(state_error)?;
        bag_peaks(&amt)
    }

    pub fn get_peaks<BS: Blockstore>(&self, store: &BS) -> anyhow::Result<Vec<Cid>, ActorError> {
        let amt = Amt::<Cid, &BS>::load(&self.peaks, store).map_err(state_error)?;
        let mut peaks = Vec::new();
        amt.for_each(|_, cid| {
            peaks.push(cid.to_owned());
            Ok(())
        })
        .map_err(state_error)?;
        Ok(peaks)
    }

    pub fn get_leaf_at<BS: Blockstore, S: DeserializeOwned + Serialize>(
        &self,
        store: &BS,
        index: u64,
    ) -> anyhow::Result<Option<S>, ActorError> {
        let amt = Amt::<Cid, &BS>::load(&self.peaks, store).map_err(state_error)?;
        get_at::<BS, S>(store, index, self.leaf_count, &amt)
            .map_err(|e| ActorError::serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_constructor() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let state = State::new(&store, Address::new_id(100), HashMap::new());
        assert!(state.is_ok());
        let state = state.unwrap();
        assert_eq!(
            state.peaks,
            Cid::from_str("bafy2bzacedijw74yui7otvo63nfl3hdq2vdzuy7wx2tnptwed6zml4vvz7wee")
                .unwrap()
        );
        assert_eq!(state.leaf_count(), 0);
    }

    #[test]
    fn test_hash_and_put_pair() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();

        let obj1 = vec![1, 2, 3];
        let obj2 = vec![1, 2, 3];
        let cid1 = state.push(&store, obj1).expect("push1 failed").root;
        let cid2 = state.push(&store, obj2).expect("push2 failed").root;

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
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();

        let obj1 = vec![1, 2, 3];
        let obj2 = vec![1, 2, 3];
        let cid1 = state.push(&store, obj1).expect("push1 failed").root;
        let cid2 = state.push(&store, obj2).expect("push2 failed").root;

        // Compare hash_pair and hash_and_put_pair and make sure they result in the same CID.
        let hash1 = hash_pair(Some(&cid1), Some(&cid2)).expect("hash_pair failed");
        let hash2 =
            hash_and_put_pair(&store, Some(&cid1), Some(&cid2)).expect("hash_and_put_pair failed");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_push_simple() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();
        let obj = vec![1, 2, 3];
        let res = state.push(&store, obj).expect("push failed");
        assert_eq!(res.root, state.get_root(&store).expect("get_root failed"));
        assert_eq!(res.index, 0);
        assert_eq!(state.leaf_count(), 1);
    }

    #[test]
    fn test_get_peaks() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();
        let obj = vec![1, 2, 3];
        assert!(state.push(&store, obj).is_ok());
        assert_eq!(state.leaf_count(), 1);
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
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();
        let mut root = Cid::default();
        for i in 1..=11 {
            let res = state.push(&store, vec![i]).unwrap();
            root = res.root;
            assert_eq!(res.index, i - 1);
        }
        let peaks = state.get_peaks(&store).unwrap();
        assert_eq!(peaks.len(), 3);
        assert_eq!(state.leaf_count(), 11);
        assert_eq!(root, state.get_root(&store).expect("get_root failed"));
    }

    #[test]
    fn test_get_obj_basic() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();

        state.push(&store, vec![0]).unwrap();
        assert_eq!(state.peak_count(), 1);
        assert_eq!(state.leaf_count(), 1);
        let item0 = state
            .get_leaf_at::<_, Vec<i32>>(&store, 0u64)
            .unwrap()
            .unwrap();
        assert_eq!(item0, vec![0]);

        state.push(&store, vec![1]).unwrap();
        assert_eq!(state.peak_count(), 1);
        assert_eq!(state.leaf_count(), 2);
        let item0 = state
            .get_leaf_at::<_, Vec<i32>>(&store, 0u64)
            .unwrap()
            .unwrap();
        let item1 = state
            .get_leaf_at::<_, Vec<i32>>(&store, 1u64)
            .unwrap()
            .unwrap();
        assert_eq!(item0, vec![0]);
        assert_eq!(item1, vec![1]);

        state.push(&store, vec![2]).unwrap();
        assert_eq!(state.peak_count(), 2);
        assert_eq!(state.leaf_count(), 3);
        let item0 = state
            .get_leaf_at::<_, Vec<i32>>(&store, 0u64)
            .unwrap()
            .unwrap();
        let item1 = state
            .get_leaf_at::<_, Vec<i32>>(&store, 1u64)
            .unwrap()
            .unwrap();
        let item2 = state
            .get_leaf_at::<_, Vec<i32>>(&store, 2u64)
            .unwrap()
            .unwrap();
        assert_eq!(item0, vec![0]);
        assert_eq!(item1, vec![1]);
        assert_eq!(item2, vec![2]);
    }

    #[test]
    fn test_get_obj() {
        let store = fvm_ipld_blockstore::MemoryBlockstore::default();
        let mut state = State::new(&store, Address::new_id(100), HashMap::new()).unwrap();
        for i in 0..31 {
            state.push(&store, vec![i]).unwrap();
            assert_eq!(state.leaf_count(), i + 1);

            // As more items are added to the timehub, ensure each item remains gettable at
            // each phase of the growth of the inner tree structures.
            for j in 0..i {
                let item = state
                    .get_leaf_at::<_, Vec<u64>>(&store, j)
                    .unwrap()
                    .unwrap();
                assert_eq!(item, vec![j]);
            }
        }
        assert_eq!(state.peak_count(), 5);
    }
}
