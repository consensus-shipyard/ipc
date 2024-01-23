// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::Array;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::{clock::ChainEpoch, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

// The state stores the blockhashes of the last `lookback_len` epochs
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // the AMT root cid of blockhashes
    pub blockhashes: Cid,

    // the maximum size of blockhashes before removing the oldest epoch
    pub lookback_len: u64,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS, lookback_len: u64) -> anyhow::Result<Self> {
        let empty_blockhashes_cid =
            match Array::<(), _>::new_with_bit_width(store, BLOCKHASHES_AMT_BITWIDTH).flush() {
                Ok(cid) => cid,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "chainmetadata actor failed to create empty Amt: {}",
                        e
                    ))
                }
            };

        Ok(Self {
            blockhashes: empty_blockhashes_cid,
            lookback_len,
        })
    }
}

// the default lookback length is 256 epochs
pub const DEFAULT_LOOKBACK_LEN: u64 = 256;

// the default bitwidth of the blockhashes AMT
pub const BLOCKHASHES_AMT_BITWIDTH: u32 = 3;

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    pub lookback_len: u64,
}

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct PushBlockParams {
    pub epoch: ChainEpoch,
    pub block: Cid,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PushBlock = frc42_dispatch::method_hash!("PushBlock"),
    LookbackLen = frc42_dispatch::method_hash!("LookbackLen"),
    BlockCID = frc42_dispatch::method_hash!("BlockCID"),
}
