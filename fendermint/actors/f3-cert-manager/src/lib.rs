// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::state::State;
use crate::types::{
    ConstructorParams, GetCertificateResponse, GetInstanceInfoResponse, PowerEntry,
    UpdateCertificateParams,
};
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch, actor_error, ActorError};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub mod state;
pub mod types;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(F3CertManagerActor);

pub const F3_CERT_MANAGER_ACTOR_NAME: &str = "f3_cert_manager";

pub struct F3CertManagerActor;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    UpdateCertificate = frc42_dispatch::method_hash!("UpdateCertificate"),
    GetCertificate = frc42_dispatch::method_hash!("GetCertificate"),
    GetInstanceInfo = frc42_dispatch::method_hash!("GetInstanceInfo"),
    GetGenesisInstanceId = frc42_dispatch::method_hash!("GetGenesisInstanceId"),
    GetGenesisPowerTable = frc42_dispatch::method_hash!("GetGenesisPowerTable"),
}

trait F3CertManager {
    /// Update the latest F3 certificate
    fn update_certificate(
        rt: &impl Runtime,
        params: UpdateCertificateParams,
    ) -> Result<(), ActorError>;

    /// Get the latest F3 certificate
    fn get_certificate(rt: &impl Runtime) -> Result<GetCertificateResponse, ActorError>;

    /// Get F3 instance information
    fn get_instance_info(rt: &impl Runtime) -> Result<GetInstanceInfoResponse, ActorError>;

    /// Get the genesis F3 instance ID
    fn get_genesis_instance_id(rt: &impl Runtime) -> Result<u64, ActorError>;

    /// Get the genesis power table
    fn get_genesis_power_table(rt: &impl Runtime) -> Result<Vec<PowerEntry>, ActorError>;
}

impl F3CertManagerActor {
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(
            params.genesis_instance_id,
            params.genesis_power_table,
            params.genesis_certificate,
        )?;

        rt.create(&state)?;
        Ok(())
    }
}

impl F3CertManager for F3CertManagerActor {
    fn update_certificate(
        rt: &impl Runtime,
        params: UpdateCertificateParams,
    ) -> Result<(), ActorError> {
        // Only allow system actor to update certificates
        // In practice, this will be called by the consensus layer when executing ParentFinality messages
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.update_certificate(rt, params.certificate)?;
            Ok(())
        })
    }

    fn get_certificate(rt: &impl Runtime) -> Result<GetCertificateResponse, ActorError> {
        // Allow any caller to read the certificate
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        Ok(GetCertificateResponse {
            certificate: state.get_latest_certificate().cloned(),
            latest_finalized_height: state.get_latest_finalized_height(),
        })
    }

    fn get_instance_info(rt: &impl Runtime) -> Result<GetInstanceInfoResponse, ActorError> {
        // Allow any caller to read the instance info
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        Ok(GetInstanceInfoResponse {
            genesis_instance_id: state.get_genesis_instance_id(),
            genesis_power_table: state.get_genesis_power_table().to_vec(),
            latest_finalized_height: state.get_latest_finalized_height(),
        })
    }

    fn get_genesis_instance_id(rt: &impl Runtime) -> Result<u64, ActorError> {
        // Allow any caller to read the genesis instance ID
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        Ok(state.get_genesis_instance_id())
    }

    fn get_genesis_power_table(rt: &impl Runtime) -> Result<Vec<PowerEntry>, ActorError> {
        // Allow any caller to read the genesis power table
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        Ok(state.get_genesis_power_table().to_vec())
    }
}

impl ActorCode for F3CertManagerActor {
    type Methods = Method;

    fn name() -> &'static str {
        F3_CERT_MANAGER_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        UpdateCertificate => update_certificate,
        GetCertificate => get_certificate,
        GetInstanceInfo => get_instance_info,
        GetGenesisInstanceId => get_genesis_instance_id,
        GetGenesisPowerTable => get_genesis_power_table,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{F3Certificate, PowerEntry};
    use cid::Cid;
    use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::error::ExitCode;
    use multihash_codetable::{Code, MultihashDigest};

    /// Helper function to create a mock F3 certificate
    fn create_test_certificate(instance_id: u64, epoch: i64) -> F3Certificate {
        // Create a dummy CID for power table
        let power_table_cid = Cid::new_v1(0x55, Code::Blake2b256.digest(b"test_power_table"));

        F3Certificate {
            instance_id,
            epoch,
            power_table_cid,
            signature: vec![1, 2, 3, 4],        // Dummy signature
            certificate_data: vec![5, 6, 7, 8], // Dummy certificate data
        }
    }

    /// Helper function to create test power entries
    fn create_test_power_entries() -> Vec<PowerEntry> {
        vec![
            PowerEntry {
                public_key: vec![1, 2, 3],
                power: 100,
            },
            PowerEntry {
                public_key: vec![4, 5, 6],
                power: 200,
            },
        ]
    }

    /// Construct the actor and verify initialization
    pub fn construct_and_verify(
        genesis_instance_id: u64,
        genesis_power_table: Vec<PowerEntry>,
        genesis_certificate: Option<F3Certificate>,
    ) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };

        // Set caller to system actor (required for constructor)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let constructor_params = ConstructorParams {
            genesis_instance_id,
            genesis_power_table,
            genesis_certificate,
        };

        let result = rt
            .call::<F3CertManagerActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&constructor_params).unwrap(),
            )
            .unwrap();

        expect_empty(result);
        rt.verify();
        rt.reset();

        rt
    }

    #[test]
    fn test_constructor_empty_state() {
        let _rt = construct_and_verify(0, vec![], None);
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_constructor_with_genesis_data() {
        let power_entries = create_test_power_entries();
        let genesis_cert = create_test_certificate(1, 100);

        let _rt = construct_and_verify(1, power_entries, Some(genesis_cert));
        // Constructor test passed if we get here without panicking
    }

    #[test]
    fn test_update_certificate_success() {
        let rt = construct_and_verify(1, vec![], None);

        // Set caller to system actor
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let new_cert = create_test_certificate(1, 200);
        let update_params = UpdateCertificateParams {
            certificate: new_cert.clone(),
        };

        let result = rt
            .call::<F3CertManagerActor>(
                Method::UpdateCertificate as u64,
                IpldBlock::serialize_cbor(&update_params).unwrap(),
            )
            .unwrap();

        expect_empty(result);
        rt.verify();

        // Test passed if we get here without error
    }

    #[test]
    fn test_update_certificate_non_advancing_height() {
        let genesis_cert = create_test_certificate(1, 100);
        let rt = construct_and_verify(1, vec![], Some(genesis_cert));

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // Try to update with same or lower height
        let same_height_cert = create_test_certificate(1, 100); // Same height
        let update_params = UpdateCertificateParams {
            certificate: same_height_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        // Should fail with illegal argument
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }

    #[test]
    fn test_update_certificate_unauthorized_caller() {
        let rt = construct_and_verify(1, vec![], None);

        // Set caller to non-system actor
        let unauthorized_caller = Address::new_id(999);
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, unauthorized_caller);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let new_cert = create_test_certificate(1, 200);
        let update_params = UpdateCertificateParams {
            certificate: new_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        // Should fail with forbidden
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_FORBIDDEN);
    }

    #[test]
    fn test_get_certificate_empty_state() {
        let rt = construct_and_verify(1, vec![], None);

        // Any caller should be able to read
        rt.expect_validate_caller_any();

        let result = rt
            .call::<F3CertManagerActor>(Method::GetCertificate as u64, None)
            .unwrap()
            .unwrap();

        let response = result.deserialize::<GetCertificateResponse>().unwrap();
        assert!(response.certificate.is_none());
        assert_eq!(response.latest_finalized_height, 0);
    }

    #[test]
    fn test_get_certificate_with_data() {
        let genesis_cert = create_test_certificate(1, 100);
        let rt = construct_and_verify(1, vec![], Some(genesis_cert.clone()));

        rt.expect_validate_caller_any();

        let result = rt
            .call::<F3CertManagerActor>(Method::GetCertificate as u64, None)
            .unwrap()
            .unwrap();

        let response = result.deserialize::<GetCertificateResponse>().unwrap();
        assert_eq!(response.certificate, Some(genesis_cert));
        assert_eq!(response.latest_finalized_height, 100);
    }

    #[test]
    fn test_get_instance_info() {
        let power_entries = create_test_power_entries();
        let rt = construct_and_verify(42, power_entries.clone(), None);

        rt.expect_validate_caller_any();

        let result = rt
            .call::<F3CertManagerActor>(Method::GetInstanceInfo as u64, None)
            .unwrap()
            .unwrap();

        let response = result.deserialize::<GetInstanceInfoResponse>().unwrap();
        assert_eq!(response.genesis_instance_id, 42);
        assert_eq!(response.genesis_power_table, power_entries);
        assert_eq!(response.latest_finalized_height, 0);
    }

    #[test]
    fn test_certificate_progression() {
        let rt = construct_and_verify(1, vec![], None);

        // Update with first certificate
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let cert1 = create_test_certificate(1, 100);
        let update_params1 = UpdateCertificateParams {
            certificate: cert1.clone(),
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params1).unwrap(),
        );
        assert!(result.is_ok());
        rt.reset();

        // Update with second certificate (higher height)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        let cert2 = create_test_certificate(1, 200);
        let update_params2 = UpdateCertificateParams {
            certificate: cert2.clone(),
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params2).unwrap(),
        );
        assert!(result.is_ok());

        // Test passed if we get here without error
    }

    #[test]
    fn test_instance_id_progression_next_instance() {
        let genesis_cert = create_test_certificate(100, 50);
        let rt = construct_and_verify(100, vec![], Some(genesis_cert));

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // Update to next instance (100 -> 101) should succeed
        let next_instance_cert = create_test_certificate(101, 10); // Epoch can be any value
        let update_params = UpdateCertificateParams {
            certificate: next_instance_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_skip_rejected() {
        let genesis_cert = create_test_certificate(100, 50);
        let rt = construct_and_verify(100, vec![], Some(genesis_cert));

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // Try to skip instance (100 -> 102) should fail
        let skipped_cert = create_test_certificate(102, 100);
        let update_params = UpdateCertificateParams {
            certificate: skipped_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }

    #[test]
    fn test_instance_id_backward_rejected() {
        let genesis_cert = create_test_certificate(100, 50);
        let rt = construct_and_verify(100, vec![], Some(genesis_cert));

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // Try to go backward (100 -> 99) should fail
        let backward_cert = create_test_certificate(99, 100);
        let update_params = UpdateCertificateParams {
            certificate: backward_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), ExitCode::USR_ILLEGAL_ARGUMENT);
    }

    #[test]
    fn test_instance_id_matches_genesis_when_no_certificate() {
        // Start with no certificate, genesis_instance_id = 50
        let rt = construct_and_verify(50, vec![], None);

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // First certificate must match genesis_instance_id (50) or be next (51)
        let matching_cert = create_test_certificate(50, 100);
        let update_params = UpdateCertificateParams {
            certificate: matching_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_instance_id_genesis_plus_one_when_no_certificate() {
        // Start with no certificate, genesis_instance_id = 50
        let rt = construct_and_verify(50, vec![], None);

        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);

        // First certificate can also be genesis + 1 (51)
        let next_instance_cert = create_test_certificate(51, 100);
        let update_params = UpdateCertificateParams {
            certificate: next_instance_cert,
        };

        let result = rt.call::<F3CertManagerActor>(
            Method::UpdateCertificate as u64,
            IpldBlock::serialize_cbor(&update_params).unwrap(),
        );

        assert!(result.is_ok());
    }
}
