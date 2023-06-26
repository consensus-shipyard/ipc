// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fendermint_vm_message::query::{ActorState, FvmQuery, GasEstimate, StateParams};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{ActorID, BLOCK_GAS_LIMIT};

use crate::QueryInterpreter;

use super::{state::FvmQueryState, FvmApplyRet, FvmMessageInterpreter};

/// Internal return type for queries. It will never be serialized
/// and sent over the wire as it is, only its internal parts are
/// sent in the response. The client has to know what to expect,
/// depending on the kind of query it sent.
pub enum FvmQueryRet {
    /// Bytes from the IPLD store retult, if found.
    Ipld(Option<Vec<u8>>),
    /// The full state of an actor, if found.
    ActorState(Option<Box<(ActorID, ActorState)>>),
    /// The results of a read-only message application.
    Call(FvmApplyRet),
    /// The estimated gas limit.
    EstimateGas(GasEstimate),
    /// Current state parameters.
    StateParams(StateParams),
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
            FvmQuery::Call(msg) => {
                let from = msg.from;
                let to = msg.to;
                let method_num = msg.method_num;
                let gas_limit = msg.gas_limit;

                let apply_ret = state.call(*msg)?;

                let ret = FvmApplyRet {
                    apply_ret,
                    from,
                    to,
                    method_num,
                    gas_limit,
                };

                FvmQueryRet::Call(ret)
            }
            FvmQuery::EstimateGas(mut msg) => {
                // TODO: Figure out how to relate the token balance to a gas limit.
                //       Do we look at `state.state_params.base_fee`? What about `gas_premium` and `gas_limit`?

                // XXX: This value is for Filecoin, and it's not even used by the FVM, but at least it should not have a problem with it.
                msg.gas_limit = BLOCK_GAS_LIMIT;

                // TODO: This actually fails if the caller doesn't have enough gas.
                //       Should we modify the state tree up front to give it more?
                let apply_ret = state.call(*msg)?;

                let est = GasEstimate {
                    exit_code: apply_ret.msg_receipt.exit_code,
                    info: apply_ret
                        .failure_info
                        .map(|x| x.to_string())
                        .unwrap_or_default(),
                    gas_limit: apply_ret.msg_receipt.gas_used,
                };

                FvmQueryRet::EstimateGas(est)
            }
            FvmQuery::StateParams => {
                let state_params = state.state_params();
                let state_params = StateParams {
                    base_fee: state_params.base_fee.clone(),
                    circ_supply: state_params.circ_supply.clone(),
                    chain_id: state_params.chain_id,
                    network_version: state_params.network_version,
                };
                FvmQueryRet::StateParams(state_params)
            }
        };
        Ok((state, res))
    }
}
