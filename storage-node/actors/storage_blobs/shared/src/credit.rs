// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::econ::TokenAmount;

mod allowance;
mod approval;
mod params;
mod token_rate;

pub use allowance::*;
pub use approval::*;
pub use params::*;
pub use token_rate::*;

/// Credit is counted the same way as tokens.
/// The smallest indivisible unit is 1 atto, and 1 credit = 1e18 atto credits.
pub type Credit = TokenAmount;
