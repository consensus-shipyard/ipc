// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use std::collections::VecDeque;

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    pub blockhashes: VecDeque<Cid>,
    pub params: ConstructorParams,
}

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    pub lookback_len: u64,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PushBlock = 2,
    LookbackLen = 3,
    BlockCID = 4,
}
