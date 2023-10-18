// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//#![allow(unused)]
//! State Machine Test for the Staking contracts.
//!
//! The test simulates random actions validators can take, such as depositing and withdrawing
//! collateral, and executes these actions on the actual Solidity contracts as well as an
//! idealised model, comparing the results and testing that invariants are maintained.
//!
//! It can be executed the following way:
//!
//! ```text
//! cargo test --release -p contract-test --test smt_staking
//! ```
use fendermint_testing::state_machine_test;

mod staking;

use staking::machine::StakingMachine;

state_machine_test!(staking, 20000 ms, 100 steps, StakingMachine::default());
