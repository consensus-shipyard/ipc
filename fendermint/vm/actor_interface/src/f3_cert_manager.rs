// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// F3 Certificate Manager actor - manages F3 certificates for proof-based parent finality
define_singleton!(F3_CERT_MANAGER {
    id: 1000,
    code_id: 1000
});

// Re-export types from the actor
pub use fendermint_actor_f3_cert_manager::types::{
    ConstructorParams, F3Certificate, GetCertificateResponse, GetInstanceInfoResponse, PowerEntry,
    UpdateCertificateParams,
};
pub use fendermint_actor_f3_cert_manager::Method;
