use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::error::ExitCode;
use fvm_shared::MethodNum;
use num_derive::FromPrimitive;

use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_dispatch, FIRST_EXPORTED_METHOD_NUMBER};
use fil_actors_runtime::{actor_error, ActorError};

use crate::types::RandomXArguments;
use crate::types::RandomXResult;

pub mod types;

pub const FLUENCE_ACTOR_NAME: &str = "fluence";
const SYSCALL_FAILED_EXIT_CODE: u32 = 0x10000000;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(FluenceActor);

/// Account actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    RunRandomX = frc42_dispatch::method_hash!("RunRandomX"),
}

/// Account Actor
pub struct FluenceActor;

impl FluenceActor {
    /// Run RandomX with the provided parameters and returns its result hash.
    pub fn run_randomx(
        rt: &impl Runtime,
        params: RandomXArguments,
    ) -> Result<RandomXResult, ActorError> {
        log::info!("actor::run_randomx: start {params:?}");
        rt.validate_immediate_caller_accept_any()?;

        let result = fluence_actor_sdk::run_randomx(params.global_nonce, params.local_nonce)
            .map_err(|error_num| {
                log::error!("actor::run_randomx: run_randomx failed with {error_num}");

                let err_msg = format!("run_randomx syscall failed with {error_num}");
                ActorError::checked(ExitCode::new(SYSCALL_FAILED_EXIT_CODE), err_msg, None)
            })?;
        log::info!("actor::run_randomx: result is {result:?}");

        let result = RandomXResult { result };
        Ok(result)
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

impl ActorCode for FluenceActor {
    type Methods = Method;

    fn name() -> &'static str {
        FLUENCE_ACTOR_NAME
    }

    actor_dispatch! {
        RunRandomX => run_randomx,
        _ => fallback,
    }
}
