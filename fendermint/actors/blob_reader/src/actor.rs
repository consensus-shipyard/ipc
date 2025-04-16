// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::Hash;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::MethodNum;
use recall_actor_sdk::emit_evm_event;

use crate::shared::{
    CloseReadRequestParams, GetOpenReadRequestsParams, GetPendingReadRequestsParams,
    GetReadRequestStatusParams, Method, OpenReadRequestParams, ReadRequestStatus, ReadRequestTuple,
    SetReadRequestPendingParams, State, BLOB_READER_ACTOR_NAME,
};
use crate::sol_facade::{ReadRequestClosed, ReadRequestOpened, ReadRequestPending};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(ReadReqActor);

pub struct ReadReqActor;

impl ReadReqActor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(rt.store())?;
        rt.create(&state)
    }

    fn open_read_request(
        rt: &impl Runtime,
        params: OpenReadRequestParams,
    ) -> Result<Hash, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let id = rt.transaction(|st: &mut State, _rt| {
            st.open_read_request(
                rt.store(),
                params.hash,
                params.offset,
                params.len,
                params.callback_addr,
                params.callback_method,
            )
        })?;

        emit_evm_event(
            rt,
            ReadRequestOpened {
                id: &id,
                blob_hash: &params.hash,
                read_offset: params.offset.into(),
                read_length: params.len.into(),
                callback: params.callback_addr,
                method_num: params.callback_method,
            },
        )?;

        Ok(id)
    }

    fn get_read_request_status(
        rt: &impl Runtime,
        params: GetReadRequestStatusParams,
    ) -> Result<Option<ReadRequestStatus>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let status = rt
            .state::<State>()?
            .get_read_request_status(rt.store(), params.0)?;
        Ok(status)
    }

    fn get_open_read_requests(
        rt: &impl Runtime,
        params: GetOpenReadRequestsParams,
    ) -> Result<Vec<ReadRequestTuple>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_read_requests_by_status(
            rt.store(),
            ReadRequestStatus::Open,
            params.0,
        )
    }

    fn get_pending_read_requests(
        rt: &impl Runtime,
        params: GetPendingReadRequestsParams,
    ) -> Result<Vec<ReadRequestTuple>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_read_requests_by_status(
            rt.store(),
            ReadRequestStatus::Pending,
            params.0,
        )
    }

    fn set_read_request_pending(
        rt: &impl Runtime,
        params: SetReadRequestPendingParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, _| st.set_read_request_pending(rt.store(), params.0))?;
        emit_evm_event(rt, ReadRequestPending::new(&params.0))
    }

    fn close_read_request(
        rt: &impl Runtime,
        params: CloseReadRequestParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.transaction(|st: &mut State, _| st.close_read_request(rt.store(), params.0))?;
        emit_evm_event(rt, ReadRequestClosed::new(&params.0))
    }

    /// Fallback method for unimplemented method numbers.
    pub fn fallback(
        rt: &impl Runtime,
        method: MethodNum,
        _: Option<IpldBlock>,
    ) -> Result<Option<IpldBlock>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if method >= FIRST_EXPORTED_METHOD_NUMBER {
            Ok(None)
        } else {
            Err(actor_error!(unhandled_message; "invalid method: {}", method))
        }
    }
}

impl ActorCode for ReadReqActor {
    type Methods = Method;

    fn name() -> &'static str {
        BLOB_READER_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,

        // User methods
        OpenReadRequest => open_read_request,

        // System methods
        GetReadRequestStatus => get_read_request_status,
        GetOpenReadRequests => get_open_read_requests,
        GetPendingReadRequests => get_pending_read_requests,
        SetReadRequestPending => set_read_request_pending,
        CloseReadRequest => close_read_request,

        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sol_facade::ReadRequestClosed;

    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use rand::RngCore;
    use recall_actor_sdk::to_actor_event;

    pub fn new_hash(size: usize) -> (Hash, u64) {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; size];
        rng.fill_bytes(&mut data);
        (Hash(*iroh_blobs::Hash::new(&data).as_bytes()), size as u64)
    }

    pub fn construct_and_verify() -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let result = rt
            .call::<ReadReqActor>(Method::Constructor as u64, None)
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();
        rt
    }

    fn expect_emitted_open_event(rt: &MockRuntime, params: &OpenReadRequestParams, id: &Hash) {
        let event = to_actor_event(ReadRequestOpened {
            id,
            blob_hash: &params.hash,
            read_offset: params.offset.into(),
            read_length: params.len.into(),
            callback: params.callback_addr,
            method_num: params.callback_method,
        })
        .unwrap();
        rt.expect_emitted_event(event);
    }

    fn expect_emitted_pending_event(rt: &MockRuntime, params: &SetReadRequestPendingParams) {
        let event = to_actor_event(ReadRequestPending::new(&params.0)).unwrap();
        rt.expect_emitted_event(event);
    }

    fn expect_emitted_closed_event(rt: &MockRuntime, params: &CloseReadRequestParams) {
        let event = to_actor_event(ReadRequestClosed::new(&params.0)).unwrap();
        rt.expect_emitted_event(event);
    }

    #[test]
    fn test_read_request_operations() {
        let rt = construct_and_verify();

        // Set up test addresses
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        // Create a test blob hash and callback details
        let blob_hash = new_hash(1024).0;
        let offset = 32u32;
        let len = 1024u32;
        let callback_method = 42u64;

        // Test opening a read request
        rt.expect_validate_caller_any();
        let open_params = OpenReadRequestParams {
            hash: blob_hash,
            offset,
            len,
            callback_addr: f4_eth_addr,
            callback_method,
        };
        let expected_id = Hash::from(1);
        expect_emitted_open_event(&rt, &open_params, &expected_id);
        let request_id = rt
            .call::<ReadReqActor>(
                Method::OpenReadRequest as u64,
                IpldBlock::serialize_cbor(&open_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Hash>()
            .unwrap();
        rt.verify();

        // Test checking request status
        rt.expect_validate_caller_any();
        let status_params = GetReadRequestStatusParams(request_id);
        let result = rt
            .call::<ReadReqActor>(
                Method::GetReadRequestStatus as u64,
                IpldBlock::serialize_cbor(&status_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Option<ReadRequestStatus>>()
            .unwrap();
        assert!(matches!(result, Some(ReadRequestStatus::Open)));
        rt.verify();

        // Test getting open requests
        rt.expect_validate_caller_any();
        let get_params = GetOpenReadRequestsParams(1); // Get just one request
        let result = rt
            .call::<ReadReqActor>(
                Method::GetOpenReadRequests as u64,
                IpldBlock::serialize_cbor(&get_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Vec<(Hash, Hash, u32, u32, Address, u64)>>()
            .unwrap();

        assert_eq!(result.len(), 1);
        let (req_id, req_blob_hash, req_offset, req_len, req_callback_addr, req_callback_method) =
            &result[0];
        assert_eq!(req_id, &request_id);
        assert_eq!(req_blob_hash, &blob_hash);
        assert_eq!(req_offset, &offset);
        assert_eq!(req_len, &len);
        assert_eq!(req_callback_addr, &f4_eth_addr);
        assert_eq!(req_callback_method, &callback_method);
        rt.verify();

        // Test setting request to pending
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let pending_params = SetReadRequestPendingParams(request_id);
        expect_emitted_pending_event(&rt, &pending_params);
        let result = rt.call::<ReadReqActor>(
            Method::SetReadRequestPending as u64,
            IpldBlock::serialize_cbor(&pending_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Verify request is now pending
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr); // Reset caller
        rt.expect_validate_caller_any();
        let status_params = GetReadRequestStatusParams(request_id);
        let result = rt
            .call::<ReadReqActor>(
                Method::GetReadRequestStatus as u64,
                IpldBlock::serialize_cbor(&status_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Option<ReadRequestStatus>>()
            .unwrap();
        assert!(matches!(result, Some(ReadRequestStatus::Pending)));
        rt.verify();

        // Test closing a request (requires system actor caller)
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let close_params = CloseReadRequestParams(request_id);
        expect_emitted_closed_event(&rt, &close_params);
        let result = rt.call::<ReadReqActor>(
            Method::CloseReadRequest as u64,
            IpldBlock::serialize_cbor(&close_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Verify request no longer exists
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr); // Reset caller
        rt.expect_validate_caller_any();
        let status_params = GetReadRequestStatusParams(request_id);
        let result = rt
            .call::<ReadReqActor>(
                Method::GetReadRequestStatus as u64,
                IpldBlock::serialize_cbor(&status_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Option<ReadRequestStatus>>()
            .unwrap();
        assert!(result.is_none());
        rt.verify();
    }

    #[test]
    fn test_read_request_error_cases() {
        let rt = construct_and_verify();

        // Set up test addresses
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);

        // Test closing non-existent request
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let non_existent_request_id = Hash([0u8; 32]);
        let close_params = CloseReadRequestParams(non_existent_request_id);
        let result = rt.call::<ReadReqActor>(
            Method::CloseReadRequest as u64,
            IpldBlock::serialize_cbor(&close_params).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();

        // Test closing request with the non-system caller
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let result = rt.call::<ReadReqActor>(
            Method::CloseReadRequest as u64,
            IpldBlock::serialize_cbor(&close_params).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();
    }
}
