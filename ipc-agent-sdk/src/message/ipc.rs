// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// We need to redefine the struct here due to:
/// In the actor, it is `Deserialize_tuple`, but when returned from json rpc endpoints, it's
/// actually `json` struct. The deserialization is not working because the agent is interpreting
/// the tuple as json.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ValidatorSet {
    pub validators: Option<Vec<Validator>>,
    // sequence number that uniquely identifies a validator set
    pub configuration_number: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryValidatorSetResponse {
    /// The validator set for the subnet fetched from the parent.
    pub validator_set: ValidatorSet,
    /// Minimum number of validators required by the subnet
    pub min_validators: u64,
    /// Genesis epoch at which the subnet was registered
    pub genesis_epoch: i64,
}

/// The validator struct. See `ValidatorSet` comment on why we need this duplicated definition.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Validator {
    pub addr: String,
    pub net_addr: String,
    pub worker_addr: Option<String>,
    pub weight: String,
}
