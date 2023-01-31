mod state;

use crate::state::{State, UserPersistParam};
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{
    actor_dispatch, actor_error, restrict_internal_api, runtime, ActorDowncast, ActorError,
    INIT_ACTOR_ADDR,
};
use fvm_shared::error::ExitCode;
use fvm_shared::{MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[no_mangle]
pub fn invoke(param: u32) -> u32 {
    runtime::fvm::trampoline::<Actor>(param)
}

/// SCA actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    /// Constructor for Storage Power Actor
    Constructor = METHOD_CONSTRUCTOR,
    Persist = frc42_dispatch::method_hash!("Persist"),
}

pub struct Actor;

impl Actor {
    /// Constructor for SCA actor
    fn constructor(rt: &mut impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;
        let st = State::new(rt.store()).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "Failed to create actor state")
        })?;
        rt.create(&st)?;
        Ok(())
    }

    /// Persists some bytes to storage
    fn persist(rt: &mut impl Runtime, param: UserPersistParam) -> Result<(), ActorError> {
        let caller = rt.message().caller();

        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            st.upsert_user(&caller, param.name, rt.store())
                .map_err(|e| {
                    e.downcast_default(
                        ExitCode::USR_ILLEGAL_STATE,
                        "Failed to create SCA actor state",
                    )
                })?;
            Ok(())
        })?;

        Ok(())
    }
}

impl ActorCode for Actor {
    type Methods = Method;
    actor_dispatch! {
        Constructor => constructor,
        Persist => persist,
    }
}

#[cfg(test)]
mod test {
    use crate::{Actor, Method, State, UserPersistParam};
    use fil_actors_runtime::test_utils::{MockRuntime, INIT_ACTOR_CODE_ID};
    use fil_actors_runtime::INIT_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::MethodNum;

    #[test]
    fn constructor_works() {
        let mut rt = new_runtime();

        rt.set_caller(*INIT_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);

        rt.call::<Actor>(Method::Constructor as MethodNum, None)
            .unwrap();

        rt.verify()
    }

    #[test]
    fn persists_works() {
        let mut rt = new_runtime();

        rt.set_caller(*INIT_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);

        rt.call::<Actor>(Method::Constructor as MethodNum, None)
            .unwrap();

        rt.expect_validate_caller_any();
        rt.call::<Actor>(
            Method::Persist as MethodNum,
            IpldBlock::serialize_cbor(&UserPersistParam {
                name: String::from("sample"),
            })
            .unwrap(),
        )
        .unwrap();

        rt.verify();
        let state: State = rt.get_state();
        assert_eq!(state.call_count, 1);
    }

    fn new_runtime() -> MockRuntime {
        MockRuntime {
            receiver: Address::new_id(1),
            caller: INIT_ACTOR_ADDR,
            ..Default::default()
        }
    }
}
