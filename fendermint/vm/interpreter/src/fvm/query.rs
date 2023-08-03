// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fendermint_vm_message::query::{ActorState, FvmQuery, GasEstimate, StateParams};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{
    bigint::BigInt, econ::TokenAmount, error::ExitCode, message::Message, ActorID, BLOCK_GAS_LIMIT,
};

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

                let apply_ret = state.call(*msg, true)?;

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
                // Populate gas message parameters.
                let est = match self.estimate_gassed_msg(&state, &mut msg)? {
                    Some(ret) => {
                        // return immediately if there is something is returned,
                        // it means that the message failed to execute so there's
                        // no point on estimating the gas.
                        ret
                    }
                    None => {
                        // perform a gas search for an accurate value
                        self.gas_search(&state, &msg)?
                    }
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

impl<DB> FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync + Clone,
{
    fn estimate_gassed_msg(
        &self,
        state: &FvmQueryState<DB>,
        msg: &mut Message,
    ) -> anyhow::Result<Option<GasEstimate>> {
        // Setting BlockGasLimit as initial limit for gas estimation
        msg.gas_limit = BLOCK_GAS_LIMIT;

        // estimate the gas limit and assign it to the message
        // do not reuse the cache
        let ret = state.call(msg.clone(), false)?;
        if !ret.msg_receipt.exit_code.is_success() {
            // if the message fail we can't estimate the gas.
            return Ok(Some(GasEstimate {
                exit_code: ret.msg_receipt.exit_code,
                info: ret.failure_info.map(|x| x.to_string()).unwrap_or_default(),
                gas_limit: 0,
            }));
        }

        msg.gas_limit = (ret.msg_receipt.gas_used as f64 * self.gas_overestimation_rate) as u64;

        if msg.gas_premium.is_zero() {
            // We need to set the gas_premium to some value other than zero for the
            // gas estimation to work accurately (I really don't know why this is
            // the case but after a lot of testing, setting this value to zero rejects the transaction)
            msg.gas_premium = TokenAmount::from_nano(BigInt::from(1));
        }

        // Same for the gas_fee_cap, not setting the fee cap leads to the message
        // being sent after the estimation to fail.
        if msg.gas_fee_cap.is_zero() {
            // TODO: In Lotus historical values of the base fee and a more accurate overestimation is performed
            // for the fee cap. If we issues with messages going through let's consider the historical analysis.
            // For now we are disregarding the base_fee so I don't think this is needed here.
            // Filecoin clamps the gas premium at GasFeeCap - BaseFee, if lower than the
            // specified premium. Returns 0 if GasFeeCap is less than BaseFee.
            // see https://spec.filecoin.io/#section-systems.filecoin_vm.message.message-semantic-validation
            msg.gas_fee_cap = msg.gas_premium.clone();
        }

        Ok(None)
    }

    // This function performs a simpler implementation of the gas search than the one used in Lotus.
    // Instead of using historical information of the gas limit for other messages, it searches
    // for a valid gas limit for the current message in isolation.
    fn gas_search(&self, state: &FvmQueryState<DB>, msg: &Message) -> anyhow::Result<GasEstimate> {
        let mut curr_limit = msg.gas_limit;

        loop {
            if let Some(ret) = self.estimation_call_with_limit(state, msg.clone(), curr_limit)? {
                return Ok(ret);
            }

            curr_limit = (curr_limit as f64 * self.gas_search_step) as u64;
            if curr_limit > BLOCK_GAS_LIMIT {
                return Ok(GasEstimate {
                    exit_code: ExitCode::OK,
                    info: "".to_string(),
                    gas_limit: BLOCK_GAS_LIMIT,
                });
            }
        }

        // TODO: For a more accurate gas estimation we could track the low and the high
        // of the search and make higher steps (e.g. `GAS_SEARCH_STEP = 2`).
        // Once an interval is found of [low, high] for which the message
        // succeeds, we make a finer-grained within that interval.
        // At this point, I don't think is worth being that accurate as long as it works.
    }

    fn estimation_call_with_limit(
        &self,
        state: &FvmQueryState<DB>,
        mut msg: Message,
        limit: u64,
    ) -> anyhow::Result<Option<GasEstimate>> {
        msg.gas_limit = limit;
        // set message nonce to zero so the right one is picked up
        msg.sequence = 0;

        let apply_ret = state.call(msg, false)?;

        let ret = GasEstimate {
            exit_code: apply_ret.msg_receipt.exit_code,
            info: apply_ret
                .failure_info
                .map(|x| x.to_string())
                .unwrap_or_default(),
            gas_limit: apply_ret.msg_receipt.gas_used,
        };

        // if the message succeeded or failed with a different error than `SYS_OUT_OF_GAS`,
        // immediately return as we either succeeded finding the right gas estimation,
        // or something non-related happened.
        if ret.exit_code == ExitCode::OK || ret.exit_code != ExitCode::SYS_OUT_OF_GAS {
            return Ok(Some(ret));
        }

        Ok(None)
    }
}
