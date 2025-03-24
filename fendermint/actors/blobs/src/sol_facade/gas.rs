// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Error;
use fvm_shared::address::Address;
use recall_actor_sdk::TryIntoEVMEvent;
use recall_sol_facade::gas as sol;
use recall_sol_facade::types::H160;

pub struct GasSponsorSet {
    sponsor: Address,
}
impl GasSponsorSet {
    pub fn mew(sponsor: Address) -> Self {
        Self { sponsor }
    }
}
impl TryIntoEVMEvent for GasSponsorSet {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let sponsor: H160 = self.sponsor.try_into()?;
        Ok(sol::Events::GasSponsorSet(sol::GasSponsorSet {
            sponsor: sponsor.into(),
        }))
    }
}

pub struct GasSponsorUnset {}
impl GasSponsorUnset {
    pub fn new() -> Self {
        Self {}
    }
}
impl TryIntoEVMEvent for GasSponsorUnset {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, Error> {
        Ok(sol::Events::GasSponsorUnset(sol::GasSponsorUnset {}))
    }
}
