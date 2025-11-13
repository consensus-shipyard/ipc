// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use cid::Cid;
use fil_actors_runtime::{actor_error, ActorError};
use recall_actor_sdk::declare_abi_call;
use recall_actor_sdk::evm::{InputData, TryIntoEVMEvent};
use recall_sol_facade::primitives::U256;
use recall_sol_facade::timehub as sol;
use recall_sol_facade::types::{SolCall, SolInterface};

use crate::{Leaf, PushParams, PushReturn};

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

// ----- Calls ----- //

declare_abi_call!();

pub fn can_handle(input_data: &InputData) -> bool {
    sol::Calls::valid_selector(input_data.selector())
}

pub fn parse_input(input: &InputData) -> Result<sol::Calls, ActorError> {
    sol::Calls::abi_decode_raw(input.selector(), input.calldata(), true)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: {}", e)))
}

impl AbiCall for sol::pushCall {
    type Params = PushParams;
    type Returns = PushReturn;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        PushParams(self.cid.0.iter().as_slice().to_vec())
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let root = returns.root.to_bytes();
        let index = returns.index;
        Self::abi_encode_returns(&(root, index))
    }
}

impl AbiCall for sol::getLeafAtCall {
    type Params = u64;
    type Returns = Option<Leaf>;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        self.index
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let (timestamp, witnessed) = if let Some(leaf) = returns {
            (leaf.timestamp, leaf.witnessed.to_bytes())
        } else {
            (u64::default(), Vec::default())
        };
        Self::abi_encode_returns(&(timestamp, witnessed))
    }
}

impl AbiCall for sol::getCountCall {
    type Params = ();
    type Returns = u64;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {}
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&(returns,))
    }
}

impl AbiCall for sol::getPeaksCall {
    type Params = ();
    type Returns = Vec<Cid>;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {}
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let cids = returns.iter().map(|cid| cid.to_bytes()).collect::<Vec<_>>();
        Self::abi_encode_returns(&(cids,))
    }
}

impl AbiCall for sol::getRootCall {
    type Params = ();
    type Returns = Cid;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {}
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&(returns.to_bytes(),))
    }
}
