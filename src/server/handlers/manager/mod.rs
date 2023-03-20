// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
pub mod create;
pub mod fund;
pub mod join;
pub mod kill;
pub mod leave;
pub mod list_subnets;
pub mod propagate;
pub mod release;
pub mod send_value;
pub mod subnet;
pub mod whitelist;

use crate::config::Subnet;
use anyhow::{anyhow, Result};
use fvm_shared::address::Address;
use std::str::FromStr;

pub(crate) fn check_subnet(subnet: &Subnet) -> Result<()> {
    if subnet.auth_token.is_none() {
        log::error!("subnet {:?} does not have auth token", subnet.id);
        return Err(anyhow!("Internal server error"));
    }
    Ok(())
}

pub(crate) fn parse_from(subnet: &Subnet, from: Option<String>) -> Result<Address> {
    let addr = match from {
        Some(addr) => Address::from_str(&addr)?,
        None => {
            if subnet.accounts.is_empty() {
                log::error!("subnet does not have account defined, {:?}", subnet.id);
                return Err(anyhow!("Internal server error"));
            } else {
                subnet.accounts[0]
            }
        }
    };
    Ok(addr)
}
