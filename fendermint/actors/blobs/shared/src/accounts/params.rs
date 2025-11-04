// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use serde::{Deserialize, Serialize};

use super::AccountStatus;

/// Params for setting account status.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct SetAccountStatusParams {
    /// Address to set the account status for.
    pub subscriber: Address,
    /// Status to set.
    pub status: AccountStatus,
}

/// Params for getting an account.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetAccountParams(pub Address);
