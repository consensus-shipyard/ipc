// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fendermint_vm_message::query::{ActorState, FvmQuery};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::ActorID;

use crate::QueryInterpreter;

use super::{state::FvmQueryState, FvmMessageInterpreter};

/// Internal return type for queries. It will never be serialized
/// and sent over the wire as it is, only its internal parts are
/// sent in the response. The client has to know what to expect,
/// depending on the kind of query it sent.
pub enum FvmQueryRet {
    /// Bytes from the IPLD store retult, if found.
    Ipld(Option<Vec<u8>>),
    /// The full state of an actor, if found.
    ActorState(Option<Box<(ActorID, ActorState)>>),
}

#[async_trait]
impl<DB> QueryInterpreter for FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync + Clone,
{
    type State = FvmQueryState<DB>;
    type Query = FvmQuery;
    type Output = FvmQueryRet;

    async fn query(
        &self,
        state: Self::State,
        qry: Self::Query,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        let res = match qry {
            FvmQuery::Ipld(cid) => FvmQueryRet::Ipld(state.store_get(&cid)?),
            FvmQuery::ActorState(addr) => {
                FvmQueryRet::ActorState(state.actor_state(&addr)?.map(Box::new))
            }
        };
        Ok((state, res))
    }
}
