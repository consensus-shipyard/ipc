// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT
mod serialization;
pub mod wallet;
mod key;

use crate::CrownJewels;
pub use wallet::*;
pub use wallet::helpers::*;
pub use key::*;

pub type FvmCrownJewels = CrownJewels<String, FvmKeyInfo, PersistentKeyInfo>;
