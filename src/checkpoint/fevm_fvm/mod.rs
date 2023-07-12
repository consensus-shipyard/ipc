// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Checkpoint manager with FEVM as parent and FVM as child

mod bottomup;
mod topdown;

pub use bottomup::BottomUpCheckpointManager;
pub use topdown::TopDownCheckpointManager;
