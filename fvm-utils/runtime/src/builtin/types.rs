// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::MethodNum;

/// Init actor Exec Params, see https://github.com/filecoin-project/builtin-actors/blob/master/actors/init/src/types.rs#L17
#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub struct InitExecParams {
    pub code_cid: Cid,
    pub constructor_params: RawBytes,
}

/// Init actor Exec Params, see https://github.com/filecoin-project/builtin-actors/blob/master/actors/init/src/types.rs
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct InitExecReturn {
    /// ID based address for created actor
    pub id_address: Address,
    /// Reorg safe address for actor
    pub robust_address: Address,
}

impl Default for InitExecReturn {
    fn default() -> Self {
        Self {
            id_address: Address::new_id(0),
            robust_address: Address::new_id(0),
        }
    }
}

/// Init actor exec method number, see https://github.com/filecoin-project/builtin-actors/blob/fb759f87fcd5de0a98cb61966cd27f680df83364/actors/init/src/lib.rs#L32
pub const INIT_EXEC_METHOD_NUM: MethodNum = 2;
