// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

// The state is a stores `blockhashes` in an AMT containing the blockhashes of the
// last `lookback_len` epochs
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // the AMT root cid of blockhashes
    pub blockhashes: Cid,

    // the maximum size of blockhashes before removing the oldest epoch
    pub lookback_len: u64,
}

// the default lookback length is 256 epochs
pub const DEFAULT_LOOKBACK_LEN: u64 = 256;

// the default bitwidth of the blockhashes AMT
pub const BLOCKHASHES_AMT_BITWIDTH: u32 = 3;

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    pub lookback_len: u64,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PushBlock = 2,
    LookbackLen = frc42_dispatch::method_hash!("LookbackLen"),
    BlockCID = frc42_dispatch::method_hash!("BlockCID"),
}
