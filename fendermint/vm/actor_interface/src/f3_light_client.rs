// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! F3 Light Client actor interface.
//!
//! This module provides the interface to the F3 Light Client actor, which manages
//! F3 light client state for proof-based parent finality. The actor tracks:
//! - Current F3 instance ID
//! - Finalized epochs chain
//! - Validator power table
//!
//! External components fetch F3 certificates from the parent chain, extract the
//! light client state, and update the actor. The actor then provides this state
//! for use in finality proofs and verification.
define_singleton!(F3_LIGHT_CLIENT {
    id: 1000,
    code_id: 1000
});

// Re-export types from the actor
pub use fendermint_actor_f3_light_client::types::{
    ConstructorParams, GetStateResponse, LightClientState, PowerEntry, UpdateStateParams,
};
pub use fendermint_actor_f3_light_client::Method;
