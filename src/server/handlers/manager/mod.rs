// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
pub mod create;
pub mod join;
pub mod kill;
pub mod leave;
pub mod list_subnets;
pub mod subnet;

use crate::config::Subnet;
use anyhow::{anyhow, Result};

pub(crate) fn check_subnet(subnet: &Subnet) -> Result<()> {
    if subnet.auth_token.is_none() {
        log::error!("must provide auth token");
        return Err(anyhow!("Internal server error"));
    }
    Ok(())
}
