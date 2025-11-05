// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Constants for Recall actors

use fvm_shared::address::Address;

/// ADM (Autonomous Data Management) actor address
/// Actor ID 17 is reserved for ADM in Recall networks
pub const ADM_ACTOR_ADDR: Address = Address::new_id(17);

