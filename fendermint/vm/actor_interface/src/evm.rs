// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_shared::METHOD_CONSTRUCTOR;
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

define_code!(EVM { code_id: 14 });

#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Resurrect = 2,
    GetBytecode = 3,
    GetBytecodeHash = 4,
    GetStorageAt = 5,
    InvokeContractDelegate = 6,
    // This hardcoded value is taken from https://github.com/filecoin-project/ref-fvm/blob/f4f3f340ba29b3800cd8272e34023606def23855/testing/integration/src/testkit/fevm.rs#L88-L89
    // where it's used because of a ciruclar dependency (frc42_dispatch needs fvm_shared).
    // Here we can use it if we want, however the release cycle is a bit lagging, preventing us from using the latest ref-fvm at the moment.
    //InvokeContract = frc42_dispatch::method_hash!("InvokeEVM"),
    InvokeContract = 3844450837,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct BytecodeReturn {
    pub code: Option<Cid>,
}
