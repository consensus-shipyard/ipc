// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use tendermint::block::Height;
use tendermint_rpc::endpoint::abci_query::AbciQuery;

use cid::Cid;
use fvm_shared::ActorID;
use fvm_shared::{address::Address, error::ExitCode};

use fendermint_vm_message::query::{ActorState, FvmQuery};

/// Fendermint client for submitting queries.
#[async_trait]
pub trait QueryClient: Send + Sync {
    /// Query the contents of a CID from the IPLD store.
    async fn ipld(&self, cid: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        let res = self.perform(FvmQuery::Ipld(*cid), None).await?;
        extract(res, |res| Ok(res.value))
    }

    /// Query the the state of an actor.
    async fn actor_state(
        &self,
        address: &Address,
        height: Option<Height>,
    ) -> anyhow::Result<Option<(ActorID, ActorState)>> {
        let res = self.perform(FvmQuery::ActorState(*address), height).await?;

        extract(res, |res| {
            let state: ActorState =
                fvm_ipld_encoding::from_slice(&res.value).context("failed to decode state")?;

            let id: ActorID =
                fvm_ipld_encoding::from_slice(&res.key).context("failed to decode ID")?;

            Ok((id, state))
        })
    }

    /// Run an ABCI query.
    async fn perform(&self, query: FvmQuery, height: Option<Height>) -> anyhow::Result<AbciQuery>;
}

/// Extract some value from the query result, unless it's not found or other error.
fn extract<T, F>(res: AbciQuery, f: F) -> anyhow::Result<Option<T>>
where
    F: FnOnce(AbciQuery) -> anyhow::Result<T>,
{
    if is_not_found(&res) {
        Ok(None)
    } else if res.code.is_err() {
        Err(anyhow!(
            "query returned non-zero exit code: {}",
            res.code.value()
        ))
    } else {
        f(res).map(Some)
    }
}

fn is_not_found(res: &AbciQuery) -> bool {
    res.code.value() == ExitCode::USR_NOT_FOUND.value()
}
