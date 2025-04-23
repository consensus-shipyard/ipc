// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use serde::{Deserialize, Serialize};

use super::Credit;

/// Params for buying credits.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuyCreditParams(pub Address);

/// Set credit sponsor.
/// If not present, the sponsor is unset.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetSponsorParams(pub Option<Address>);

/// Params for updating credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct UpdateGasAllowanceParams {
    /// Account address that initiated the update.
    pub from: Address,
    /// Optional account address that is sponsoring the update.
    pub sponsor: Option<Address>,
    /// Token amount to add, which can be negative.
    pub add_amount: TokenAmount,
}

/// Params for approving credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ApproveCreditParams {
    /// Account address that is receiving the approval.
    pub to: Address,
    /// Optional restriction on caller addresses, e.g., a bucket.
    /// The receiver will only be able to use the approval via an allowlisted caller.
    /// If not present, any caller is allowed.
    pub caller_allowlist: Option<HashSet<Address>>,
    /// Optional credit approval limit.
    /// If specified, the approval becomes invalid once the used credits reach the
    /// specified limit.
    pub credit_limit: Option<Credit>,
    /// Optional gas fee limit.
    /// If specified, the approval becomes invalid once the used gas fees reach the
    /// specified limit.
    pub gas_fee_limit: Option<TokenAmount>,
    /// Optional credit approval time-to-live epochs.
    /// If specified, the approval becomes invalid after this duration.
    pub ttl: Option<ChainEpoch>,
}

/// Params for revoking credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RevokeCreditParams {
    /// Account address whose approval is being revoked.
    pub to: Address,
    /// Optional caller address to remove from the caller allowlist.
    /// If not present, the entire approval is revoked.
    pub for_caller: Option<Address>,
}

/// Params for looking up a credit approval.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetCreditApprovalParams {
    /// Account address that made the approval.
    pub from: Address,
    /// Account address that received the approval.
    pub to: Address,
}

/// Params for looking up credit allowance.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetGasAllowanceParams(pub Address);
