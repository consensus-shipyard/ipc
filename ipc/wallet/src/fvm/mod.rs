// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT
mod key;
mod serialization;
pub mod wallet;

use crate::CrownJewels;
pub use key::*;
pub use wallet::helpers::*;
pub use wallet::*;

pub type FvmCrownJewels = CrownJewels<String, FvmKeyInfo, PersistentKeyInfo>;

pub const ENCRYPTED_KEYSTORE_NAME: &str = "keystore";
pub const KEYSTORE_NAME: &str = "keystore.json";
