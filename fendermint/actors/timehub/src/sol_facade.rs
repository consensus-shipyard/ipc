// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use cid::Cid;
use recall_actor_sdk::evm::TryIntoEVMEvent;
use recall_sol_facade::primitives::U256;
use recall_sol_facade::timehub as sol;

pub struct EventPushed {
    index: u64,
    timestamp: u64,
    cid: Cid,
}
impl EventPushed {
    pub fn new(index: u64, timestamp: u64, cid: Cid) -> Self {
        Self {
            index,
            timestamp,
            cid,
        }
    }
}
impl TryIntoEVMEvent for EventPushed {
    type Target = sol::Events;

    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        Ok(sol::Events::EventPushed(sol::EventPushed {
            index: U256::from(self.index),
            timestamp: U256::from(self.timestamp),
            cid: self.cid.to_bytes().into(),
        }))
    }
}
