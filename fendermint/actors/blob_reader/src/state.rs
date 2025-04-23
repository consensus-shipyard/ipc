// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::bytes::B256;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use log::info;
use recall_ipld::hamt::{self, map::TrackedFlushResult};

use crate::shared::{ReadRequest, ReadRequestStatus, ReadRequestTuple};

const MAX_READ_REQUEST_LEN: u32 = 1024 * 1024; // 1MB

/// The state represents all read requests.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// ReadRequests Hamt.
    pub read_requests: ReadRequests,
    /// Counter to sequence the requests
    pub request_id_counter: u64,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let read_requests = ReadRequests::new(store)?;
        Ok(State {
            read_requests,
            request_id_counter: 0,
        })
    }

    pub fn open_read_request<BS: Blockstore>(
        &mut self,
        store: &BS,
        blob_hash: B256,
        offset: u32,
        len: u32,
        callback_addr: Address,
        callback_method: u64,
    ) -> Result<B256, ActorError> {
        // Validate length is not greater than the maximum allowed
        if len > MAX_READ_REQUEST_LEN {
            return Err(ActorError::illegal_argument(format!(
                "read request length {} exceeds maximum allowed {}",
                len, MAX_READ_REQUEST_LEN
            )));
        }

        let request_id = self.next_request_id();
        let read_request = ReadRequest {
            blob_hash,
            offset,
            len,
            callback_addr,
            callback_method,
            status: ReadRequestStatus::Open,
        };
        info!("opening a read request onchain: {:?}", request_id);
        // will create a new request even if the request parameters are the same
        let mut read_requests = self.read_requests.hamt(store)?;
        self.read_requests
            .save_tracked(read_requests.set_and_flush_tracked(&request_id, read_request)?);
        Ok(request_id)
    }

    pub fn get_read_request_status<BS: Blockstore>(
        &self,
        store: BS,
        id: B256,
    ) -> Result<Option<ReadRequestStatus>, ActorError> {
        let read_requests = self.read_requests.hamt(store)?;
        Ok(read_requests.get(&id)?.map(|r| r.status.clone()))
    }

    pub fn get_read_requests_by_status<BS: Blockstore>(
        &self,
        store: BS,
        status: ReadRequestStatus,
        size: u32,
    ) -> Result<Vec<ReadRequestTuple>, ActorError> {
        let read_requests = self.read_requests.hamt(store)?;

        let mut requests = Vec::new();
        read_requests.for_each(|id, request| {
            if request.status == status && (requests.len() as u32) < size {
                requests.push((
                    id,
                    request.blob_hash,
                    request.offset,
                    request.len,
                    request.callback_addr,
                    request.callback_method,
                ))
            }

            Ok(())
        })?;
        Ok(requests)
    }

    /// Set a read request status to pending.
    pub fn set_read_request_pending<BS: Blockstore>(
        &mut self,
        store: BS,
        id: B256,
    ) -> Result<(), ActorError> {
        let mut read_requests = self.read_requests.hamt(store)?;
        let mut request = read_requests
            .get(&id)?
            .ok_or_else(|| ActorError::not_found(format!("read request {} not found", id)))?;

        if !matches!(request.status, ReadRequestStatus::Open) {
            return Err(ActorError::illegal_state(format!(
                "read request {} is not in open state",
                id
            )));
        }

        request.status = ReadRequestStatus::Pending;
        self.read_requests
            .save_tracked(read_requests.set_and_flush_tracked(&id, request)?);

        Ok(())
    }

    pub fn close_read_request<BS: Blockstore>(
        &mut self,
        store: &BS,
        request_id: B256,
    ) -> Result<(), ActorError> {
        if self.get_read_request_status(store, request_id)?.is_none() {
            return Err(ActorError::not_found(
                "cannot close read request, it does not exist".to_string(),
            ));
        }

        // remove the closed request
        let mut read_requests = self.read_requests.hamt(store)?;
        self.read_requests
            .save_tracked(read_requests.delete_and_flush_tracked(&request_id)?.0);
        Ok(())
    }

    fn next_request_id(&mut self) -> B256 {
        self.request_id_counter += 1;
        B256::from(self.request_id_counter)
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ReadRequests {
    pub root: hamt::Root<B256, ReadRequest>,
    size: u64,
}

impl ReadRequests {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<B256, ReadRequest>::new(store, "read_requests")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, B256, ReadRequest>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<B256, ReadRequest>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }
}
