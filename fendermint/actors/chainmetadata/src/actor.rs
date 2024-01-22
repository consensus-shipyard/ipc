// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::str::FromStr;

use cid::Cid;
use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorDowncast;
use fil_actors_runtime::ActorError;
use fil_actors_runtime::Array;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::error::ExitCode;

use crate::shared::BLOCKHASHES_AMT_BITWIDTH;
use crate::{ConstructorParams, Method, PushBlockParams, State};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let empty_arr_cid =
            Array::<(), _>::new_with_bit_width(rt.store(), BLOCKHASHES_AMT_BITWIDTH)
                .flush()
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to create empty AMT")
                })?;

        let state = State {
            blockhashes: empty_arr_cid,
            lookback_len: params.lookback_len,
        };

        rt.create(&state)?;

        Ok(())
    }

    fn push_block(rt: &impl Runtime, params: PushBlockParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            // load the blockhashes AMT
            let mut blockhashes = Array::load(&st.blockhashes, rt.store()).map_err(|e| {
                e.downcast_default(
                    ExitCode::USR_ILLEGAL_STATE,
                    "failed to load blockhashes states",
                )
            })?;

            // push the block to the AMT
            blockhashes
                .set(params.epoch as u64, params.block.to_string())
                .unwrap();

            // remove the oldest block if the AMT is full
            if blockhashes.count() > st.lookback_len {
                let mut first_idx = 0;
                blockhashes
                    .for_each_while(|i, _: &String| {
                        first_idx = i;
                        Ok(false)
                    })
                    .unwrap();
                blockhashes.delete(first_idx).unwrap();
            }

            // save the new blockhashes AMT cid root
            st.blockhashes = blockhashes.flush().map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to save blockhashes")
            })?;

            Ok(())
        })?;

        Ok(())
    }

    fn lookback_len(rt: &impl Runtime) -> Result<u64, ActorError> {
        let state: State = rt.state()?;
        Ok(state.lookback_len)
    }

    fn block_cid(rt: &impl Runtime, epoch: ChainEpoch) -> Result<Option<Cid>, ActorError> {
        let st: State = rt.state()?;

        // load the blockhashes AMT
        let blockhashes = Array::load(&st.blockhashes, rt.store()).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to load blockhashes states",
            )
        })?;

        // get the block cid from the AMT, if it does not exist return None
        let blockhash: &String = match blockhashes.get(epoch as u64).unwrap() {
            Some(v) => v,
            None => {
                return Ok(None);
            }
        };

        // return the blockhash as a cid, or an error if the cid is invalid
        match Cid::from_str(blockhash.as_str()) {
            Ok(cid) => Ok(Some(cid)),
            Err(_) => Err(ActorError::unchecked(
                ExitCode::USR_ILLEGAL_STATE,
                format!(
                    "failed to parse cid, blockhash: {}, epoch: {}",
                    blockhash, epoch
                ),
            )),
        }
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        "ChainMetadata"
    }

    actor_dispatch! {
        Constructor => constructor,
        PushBlock => push_block,
        LookbackLen => lookback_len,
        BlockCID => block_cid,
    }
}
