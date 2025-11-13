// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::bytes::B256;
use fvm_shared::{address::Address, MethodNum};
use recall_actor_sdk::evm::TryIntoEVMEvent;
use recall_sol_facade::{blob_reader as sol, primitives::U256, types::H160};

pub struct ReadRequestOpened<'a> {
    pub id: &'a B256,
    pub blob_hash: &'a B256,
    pub read_offset: u64,
    pub read_length: u64,
    pub callback: Address,
    pub method_num: MethodNum,
}
impl TryIntoEVMEvent for ReadRequestOpened<'_> {
    type Target = sol::Events;

    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let callback_address: H160 = self.callback.try_into()?;
        Ok(sol::Events::ReadRequestOpened(sol::ReadRequestOpened {
            id: self.id.0.into(),
            blobHash: self.blob_hash.0.into(),
            readOffset: U256::from(self.read_offset),
            readLength: U256::from(self.read_length),
            callbackAddress: callback_address.into(),
            callbackMethod: U256::from(self.method_num),
        }))
    }
}

pub struct ReadRequestPending<'a> {
    pub id: &'a B256,
}
impl<'a> ReadRequestPending<'a> {
    pub fn new(id: &'a B256) -> Self {
        Self { id }
    }
}
impl TryIntoEVMEvent for ReadRequestPending<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        Ok(sol::Events::ReadRequestPending(sol::ReadRequestPending {
            id: self.id.0.into(),
        }))
    }
}

pub struct ReadRequestClosed<'a> {
    pub id: &'a B256,
}
impl<'a> ReadRequestClosed<'a> {
    pub fn new(id: &'a B256) -> Self {
        Self { id }
    }
}
impl TryIntoEVMEvent for ReadRequestClosed<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        Ok(sol::Events::ReadRequestClosed(sol::ReadRequestClosed {
            id: self.id.0.into(),
        }))
    }
}
