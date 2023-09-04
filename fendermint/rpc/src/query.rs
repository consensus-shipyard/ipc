// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use fvm_ipld_encoding::serde::Serialize;
use fvm_shared::message::Message;
use prost::Message as ProstMessage;
use tendermint::block::Height;
use tendermint::v0_37::abci::response;
use tendermint_rpc::endpoint::abci_query::AbciQuery;

use cid::Cid;
use fvm_shared::ActorID;
use fvm_shared::{address::Address, error::ExitCode};

use fendermint_vm_message::query::{
    ActorState, FvmQuery, FvmQueryHeight, GasEstimate, StateParams,
};

use crate::response::encode_data;

#[derive(Serialize, Debug, Clone)]
/// The parsed value from a query, along with the height at which the query was performed.
pub struct QueryResponse<T> {
    pub height: Height,
    pub value: T,
}

/// Fendermint client for submitting queries.
#[async_trait]
pub trait QueryClient: Sync {
    /// Query the contents of a CID from the IPLD store.
    async fn ipld(&self, cid: &Cid, height: FvmQueryHeight) -> anyhow::Result<Option<Vec<u8>>> {
        let res = self.perform(FvmQuery::Ipld(*cid), height).await?;
        extract_opt(res, |res| Ok(res.value))
    }

    /// Query the the state of an actor.
    async fn actor_state(
        &self,
        address: &Address,
        height: FvmQueryHeight,
    ) -> anyhow::Result<QueryResponse<Option<(ActorID, ActorState)>>> {
        let res = self.perform(FvmQuery::ActorState(*address), height).await?;
        let height = res.height;
        let value = extract_actor_state(res)?;
        Ok(QueryResponse { height, value })
    }

    /// Run a message in a read-only fashion.
    async fn call(
        &self,
        message: Message,
        height: FvmQueryHeight,
    ) -> anyhow::Result<QueryResponse<response::DeliverTx>> {
        let res = self
            .perform(FvmQuery::Call(Box::new(message)), height)
            .await?;
        let height = res.height;
        let value = extract(res, |res| {
            let bz: Vec<u8> = fvm_ipld_encoding::from_slice(&res.value)
                .context("failed to decode IPLD as bytes")?;

            let deliver_tx = tendermint_proto::abci::ResponseDeliverTx::decode(bz.as_ref())
                .context("failed to deserialize ResponseDeliverTx from proto bytes")?;

            let mut deliver_tx = tendermint::abci::response::DeliverTx::try_from(deliver_tx)
                .context("failed to create DeliverTx from proto response")?;

            // Mimic the Base64 encoding of the value that Tendermint does.
            deliver_tx.data = encode_data(&deliver_tx.data);

            Ok(deliver_tx)
        })?;
        Ok(QueryResponse { height, value })
    }

    /// Estimate the gas limit of a message.
    async fn estimate_gas(
        &self,
        message: Message,
        height: FvmQueryHeight,
    ) -> anyhow::Result<QueryResponse<GasEstimate>> {
        let res = self
            .perform(FvmQuery::EstimateGas(Box::new(message)), height)
            .await?;
        let height = res.height;
        let value = extract(res, |res| {
            fvm_ipld_encoding::from_slice(&res.value)
                .context("failed to decode GasEstimate from query")
        })?;
        Ok(QueryResponse { height, value })
    }

    /// Slowly changing state parameters.
    async fn state_params(
        &self,
        height: FvmQueryHeight,
    ) -> anyhow::Result<QueryResponse<StateParams>> {
        let res = self.perform(FvmQuery::StateParams, height).await?;
        let height = res.height;
        let value = extract(res, |res| {
            fvm_ipld_encoding::from_slice(&res.value)
                .context("failed to decode StateParams from query")
        })?;
        Ok(QueryResponse { height, value })
    }

    /// Run an ABCI query.
    async fn perform(&self, query: FvmQuery, height: FvmQueryHeight) -> anyhow::Result<AbciQuery>;
}

/// Extract some value from the query result, unless it's not found or other error.
fn extract_opt<T, F>(res: AbciQuery, f: F) -> anyhow::Result<Option<T>>
where
    F: FnOnce(AbciQuery) -> anyhow::Result<T>,
{
    if is_not_found(&res) {
        Ok(None)
    } else {
        extract(res, f).map(Some)
    }
}

/// Extract some value from the query result, unless there was an error.
fn extract<T, F>(res: AbciQuery, f: F) -> anyhow::Result<T>
where
    F: FnOnce(AbciQuery) -> anyhow::Result<T>,
{
    if res.code.is_err() {
        Err(anyhow!(
            "query returned non-zero exit code: {}",
            res.code.value()
        ))
    } else {
        f(res)
    }
}

fn extract_actor_state(res: AbciQuery) -> anyhow::Result<Option<(ActorID, ActorState)>> {
    extract_opt(res, |res| {
        let state: ActorState =
            fvm_ipld_encoding::from_slice(&res.value).context("failed to decode state")?;

        let id: ActorID = fvm_ipld_encoding::from_slice(&res.key).context("failed to decode ID")?;

        Ok((id, state))
    })
}

fn is_not_found(res: &AbciQuery) -> bool {
    res.code.value() == ExitCode::USR_NOT_FOUND.value()
}
