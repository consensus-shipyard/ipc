// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Run tests against multiple Fendermint+CometBFT docker container pairs locally,
//! where one is allowed to run for a while and export some snapshots, then another
//! is started to sync its state directly with it.
//!
//! Example:
//!
//! ```text
//! cd fendermint/testing/snapshot-test
//! cargo make
//! ```
//!
//! Make sure you installed cargo-make by running `cargo install cargo-make` first.
