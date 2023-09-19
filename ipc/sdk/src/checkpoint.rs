// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Cross network messages related struct and utility functions.

use crate::cross::CrossMsg;
use crate::subnet_id::SubnetID;
use crate::ValidatorSet;
use anyhow::anyhow;
use cid::multihash::Code;
use cid::multihash::MultihashDigest;
use cid::Cid;
use fvm_ipld_encoding::DAG_CBOR;
use fvm_ipld_encoding::{serde_bytes, to_vec};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use lazy_static::lazy_static;
use num_traits::Zero;
use primitives::{TCid, TLink};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

lazy_static! {
    // Default CID used for the genesis checkpoint. Using
    // TCid::default() leads to corrupting the fvm datastore
    // for storing the cid of an inaccessible HAMT.
    pub static ref CHECKPOINT_GENESIS_CID: Cid =
        Cid::new_v1(DAG_CBOR, Code::Blake2b256.digest("genesis".as_bytes()));
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BottomUpCheckpoint {
    pub data: CheckData,
    #[serde(with = "serde_bytes")]
    pub sig: Vec<u8>,
}

impl BottomUpCheckpoint {
    pub fn new(id: SubnetID, epoch: ChainEpoch) -> Self {
        Self {
            data: CheckData::new(id, epoch),
            sig: Vec::new(),
        }
    }

    /// return cid for the checkpoint
    pub fn cid(&self) -> Cid {
        let mh_code = Code::Blake2b256;
        // we only use the data of the checkpoint to compute the cid, the signature
        // can change according to the source. We are only interested in the data.
        Cid::new_v1(
            fvm_ipld_encoding::DAG_CBOR,
            mh_code.digest(&to_vec(&self.data).unwrap()),
        )
    }

    /// return checkpoint epoch
    pub fn epoch(&self) -> ChainEpoch {
        self.data.epoch
    }

    /// return signature
    pub fn signature(&self) -> &Vec<u8> {
        &self.sig
    }

    /// set signature of checkpoint
    pub fn set_signature(&mut self, sig: Vec<u8>) {
        self.sig = sig;
    }

    /// return checkpoint source
    pub fn source(&self) -> &SubnetID {
        &self.data.source
    }

    /// return the cid of the previous checkpoint this checkpoint points to.
    pub fn prev_check(&self) -> &TCid<TLink<BottomUpCheckpoint>> {
        &self.data.prev_check
    }

    /// Take the cross messages out of the checkpoint. This will empty the `self.data.cross_msgs`
    /// and replace with None.
    pub fn cross_msgs(&mut self) -> Option<Vec<CrossMsg>> {
        self.data.cross_msgs.cross_msgs.clone()
    }

    /// Get the sum of values in cross messages
    pub fn total_value(&self) -> TokenAmount {
        match &self.data.cross_msgs.cross_msgs {
            None => TokenAmount::zero(),
            Some(cross_msgs) => {
                let mut value = TokenAmount::zero();
                cross_msgs.iter().for_each(|cross_msg| {
                    value += &cross_msg.msg.value;
                });
                value
            }
        }
    }

    /// Get the total fee of the cross messages
    pub fn total_fee(&self) -> &TokenAmount {
        &self.data.cross_msgs.fee
    }

    pub fn push_cross_msgs(&mut self, cross_msg: CrossMsg, fee: &TokenAmount) {
        self.data.cross_msgs.fee += fee;
        match self.data.cross_msgs.cross_msgs.as_mut() {
            None => self.data.cross_msgs.cross_msgs = Some(vec![cross_msg]),
            Some(v) => v.push(cross_msg),
        };
    }

    /// Add the cid of a checkpoint from a child subnet for further propagation
    /// to the upper layerse of the hierarchy.
    pub fn add_child_check(&mut self, commit: &BottomUpCheckpoint) -> anyhow::Result<()> {
        let cid = TCid::from(commit.cid());
        match self
            .data
            .children
            .iter_mut()
            .find(|m| commit.source() == &m.source)
        {
            // if there is already a structure for that child
            Some(ck) => {
                // check if the cid already exists
                if ck.checks.iter().any(|c| c == &cid) {
                    return Err(anyhow!(
                        "child checkpoint being committed already exists for source {}",
                        commit.source()
                    ));
                }
                // and if not append to list of child checkpoints.
                ck.checks.push(cid);
            }
            None => {
                // if none, new structure for source
                self.data.children.push(ChildCheck {
                    source: commit.data.source.clone(),
                    checks: vec![cid],
                });
            }
        };
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CheckData {
    pub source: SubnetID,
    // subnet-specific proof propagated as part of the checkpoint (initially we propagate)
    // a pointer to the tipset at the specific epoch of  the checkpoint.
    #[serde(with = "serde_bytes")]
    pub proof: Vec<u8>,
    pub epoch: ChainEpoch,
    pub prev_check: TCid<TLink<BottomUpCheckpoint>>,
    pub children: Vec<ChildCheck>,
    pub cross_msgs: BatchCrossMsgs,
}

#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct BatchCrossMsgs {
    pub cross_msgs: Option<Vec<CrossMsg>>,
    pub fee: TokenAmount,
}

impl CheckData {
    pub fn new(id: SubnetID, epoch: ChainEpoch) -> Self {
        Self {
            source: id,
            proof: Vec::new(),
            epoch,
            prev_check: (*CHECKPOINT_GENESIS_CID).into(),
            children: Vec::new(),
            cross_msgs: BatchCrossMsgs::default(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ChildCheck {
    pub source: SubnetID,
    pub checks: Vec<TCid<TLink<BottomUpCheckpoint>>>,
}

/// Validators tracks all the validator in the subnet. It is useful in handling top-down checkpoints.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Validators {
    /// The validator set that holds all the validators
    pub validators: ValidatorSet,
    /// Tracks the total weight of the validators
    pub total_weight: TokenAmount,
}

impl Validators {
    pub fn new(validators: ValidatorSet) -> Self {
        let mut weight = TokenAmount::zero();
        for v in validators.validators() {
            weight += v.weight.clone();
        }
        Self {
            validators,
            total_weight: weight,
        }
    }
}

impl Serialize for BatchCrossMsgs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(v) = self.cross_msgs.as_ref() {
            let inner = (v, &self.fee);
            serde::Serialize::serialize(&inner, serde_tuple::Serializer(serializer))
        } else {
            let inner: (&Vec<CrossMsg>, &TokenAmount) = (&vec![], &self.fee);
            serde::Serialize::serialize(&inner, serde_tuple::Serializer(serializer))
        }
    }
}

impl<'de> Deserialize<'de> for BatchCrossMsgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Inner = (Vec<CrossMsg>, TokenAmount);
        let inner = Inner::deserialize(serde_tuple::Deserializer(deserializer))?;
        Ok(BatchCrossMsgs {
            cross_msgs: if inner.0.is_empty() {
                None
            } else {
                Some(inner.0)
            },
            fee: inner.1,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::checkpoint::BottomUpCheckpoint;
    use crate::subnet_id::SubnetID;
    use cid::Cid;
    use fil_actors_runtime::cbor;
    use primitives::TCid;
    use std::str::FromStr;

    #[test]
    fn test_serialization() {
        let mut checkpoint = BottomUpCheckpoint::new(SubnetID::from_str("/r123").unwrap(), 10);
        checkpoint.data.prev_check = TCid::from(
            Cid::from_str("bafy2bzacecnamqgqmifpluoeldx7zzglxcljo6oja4vrmtj7432rphldpdmm2")
                .unwrap(),
        );

        let raw_bytes = cbor::serialize(&checkpoint, "").unwrap();
        let de = cbor::deserialize(&raw_bytes, "").unwrap();
        assert_eq!(checkpoint, de);
    }
}
