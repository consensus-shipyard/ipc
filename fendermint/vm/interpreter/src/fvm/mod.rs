// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::marker::PhantomData;

mod check;
mod exec;
mod externs;
mod query;
mod state;

#[cfg(any(test, feature = "bundle"))]
pub mod bundle;

pub use check::FvmCheckRet;
pub use exec::FvmApplyRet;
pub use fendermint_vm_message::query::FvmQuery;
pub use query::FvmQueryRet;
pub use state::{FvmCheckState, FvmExecState, FvmGenesisState, FvmQueryState};

pub type FvmMessage = fvm_shared::message::Message;

/// Interpreter working on already verified unsigned messages.
#[derive(Clone)]
pub struct FvmMessageInterpreter<DB> {
    _phantom_db: PhantomData<DB>,
}

impl<DB> FvmMessageInterpreter<DB> {
    pub fn new() -> Self {
        Self {
            _phantom_db: PhantomData,
        }
    }
}
