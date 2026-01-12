// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{extract_send_result, runtime::Runtime, ActorError};
use fvm_shared::{address::Address, bigint::Zero, econ::TokenAmount, error::ExitCode, METHOD_SEND};

use crate::util::{to_id_address, to_id_and_delegated_address};

/// Helper stuct for managing actor message caller and sponsor addresses.
#[derive(Debug)]
pub struct Caller {
    /// Caller ID-address.
    id_addr: Address,
    /// Caller delegated address.
    delegated_addr: Option<Address>,
    /// Caller's sponsor ID-address.
    sponsor_id_addr: Option<Address>,
    /// Caller's sponsor delegated address.
    sponsor_delegated_addr: Option<Address>,
    /// Whether the caller actor was created.
    created: bool,
}

/// Caller option (authenticate or create).
#[derive(Debug, Default)]
pub enum CallerOption {
    #[default]
    None,
    /// The target address must be the runtime's message origin or caller.
    Auth,
    /// Create the target address if it's not found.
    Create,
}

impl Caller {
    /// Returns a new caller.
    /// TODO: Remove origin authentication after the solidity facades are complete.
    pub fn new(
        rt: &impl Runtime,
        address: Address,
        sponsor: Option<Address>,
        option: CallerOption,
    ) -> Result<Self, ActorError> {
        let mut created = false;
        let id_addr = match to_id_address(rt, address, false) {
            Ok(addr) => Ok(addr),
            Err(e)
                if matches!(option, CallerOption::Create)
                    && e.exit_code() == ExitCode::USR_NOT_FOUND =>
            {
                create_actor(rt, address)?;
                created = true;
                to_id_address(rt, address, false)
            }
            Err(e) => Err(e),
        }?;

        let caller = match sponsor {
            Some(sponsor) => {
                let sponsor_id_addr = to_id_address(rt, sponsor, false)?;
                Self {
                    id_addr,
                    delegated_addr: None,
                    sponsor_id_addr: Some(sponsor_id_addr),
                    sponsor_delegated_addr: None,
                    created,
                }
            }
            None => Self {
                id_addr,
                delegated_addr: None,
                sponsor_id_addr: None,
                sponsor_delegated_addr: None,
                created,
            },
        };
        Ok(caller)
    }

    /// Returns a new caller.
    /// Caller and sponsor must have a delegated address.
    /// TODO: Remove origin authentication after the solidity facades are complete.
    pub fn new_delegated(
        rt: &impl Runtime,
        address: Address,
        sponsor: Option<Address>,
        option: CallerOption,
    ) -> Result<Self, ActorError> {
        let mut created = false;
        let (id_addr, delegated_addr) = match to_id_and_delegated_address(rt, address) {
            Ok(addrs) => Ok(addrs),
            Err(e)
                if matches!(option, CallerOption::Create)
                    && e.exit_code() == ExitCode::USR_NOT_FOUND =>
            {
                create_actor(rt, address)?;
                created = true;
                to_id_and_delegated_address(rt, address)
            }
            Err(e) => Err(e),
        }?;

        let caller = match sponsor {
            Some(sponsor) => {
                let (sponsor_id_addr, sponsor_delegated_addr) =
                    to_id_and_delegated_address(rt, sponsor)?;
                Self {
                    id_addr,
                    delegated_addr: Some(delegated_addr),
                    sponsor_id_addr: Some(sponsor_id_addr),
                    sponsor_delegated_addr: Some(sponsor_delegated_addr),
                    created,
                }
            }
            None => Self {
                id_addr,
                delegated_addr: Some(delegated_addr),
                sponsor_id_addr: None,
                sponsor_delegated_addr: None,
                created,
            },
        };
        Ok(caller)
    }

    /// Returns the caller delegated address.
    pub fn address(&self) -> Address {
        self.delegated_addr.unwrap_or(self.id_addr)
    }

    /// Returns the caller address that should be used with actor state methods.
    pub fn state_address(&self) -> Address {
        self.id_addr
    }

    /// Returns the sponsor address that should be used with actor state methods.
    pub fn sponsor_state_address(&self) -> Option<Address> {
        self.sponsor_id_addr
    }

    /// Returns the sponsor delegated address.
    pub fn sponsor_address(&self) -> Option<Address> {
        self.sponsor_delegated_addr
    }

    /// Returns the address that should be used with events.
    pub fn event_address(&self) -> Address {
        self.sponsor_delegated_addr.unwrap_or(self.address())
    }

    /// Returns whether the caller actor was created.
    pub fn created(&self) -> bool {
        self.created
    }
}

/// Creates a new placeholder actor by sending zero tokens to the address.
fn create_actor(rt: &impl Runtime, address: Address) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(&address, METHOD_SEND, None, TokenAmount::zero()))?;
    Ok(())
}
