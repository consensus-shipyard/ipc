mod state;

use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::{actor_error, ActorDowncast, ActorError, cbor, INIT_ACTOR_ADDR, runtime};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::error::ExitCode;
use fvm_shared::{MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use crate::state::{State, UserPersistParam};

#[no_mangle]
pub fn invoke(param: u32) -> u32 {
    runtime::fvm::trampoline::<Actor>(param)
}

// #[cfg(feature = "fil-actor")]
// fil_actors_runtime::wasm_trampoline!(Actor);

/// SCA actor methods available
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    /// Constructor for Storage Power Actor
    Constructor = METHOD_CONSTRUCTOR,
    Persist = 2,
}

/// Subnet Coordinator Actor
pub struct Actor;

impl Actor {
    /// Constructor for SCA actor
    fn constructor<BS, RT>(rt: &mut RT) -> Result<(), ActorError>
        where
            BS: Blockstore,
            RT: Runtime<BS>,
    {
        rt.validate_immediate_caller_is(std::iter::once(&*INIT_ACTOR_ADDR))?;
        let st = State::new(rt.store()).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "Failed to create actor state")
        })?;
        rt.create(&st)?;
        Ok(())
    }

    /// Persists some bytes to storage
    fn persist<BS, RT>(rt: &mut RT, param: UserPersistParam) -> Result<(), ActorError>
        where
            BS: Blockstore,
            RT: Runtime<BS>,
    {
        let caller = rt.message().caller();

        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            st.upsert_user(&caller, param.name, rt.store()).map_err(|e|
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "Failed to create SCA actor state")
            )?;
            Ok(())
        })?;

        Ok(())
    }
}

impl ActorCode for Actor {
    fn invoke_method<BS, RT>(
        rt: &mut RT,
        method: MethodNum,
        params: &RawBytes,
    ) -> Result<RawBytes, ActorError>
        where
            BS: Blockstore,
            RT: Runtime<BS>,
    {
        match FromPrimitive::from_u64(method) {
            Some(Method::Constructor) => {
                Self::constructor(rt)?;
                Ok(RawBytes::default())
            }
            Some(Method::Persist) => {
                Self::persist(rt, cbor::deserialize_params(params)?)?;
                Ok(RawBytes::serialize(())?)
            }
            None => Err(actor_error!(unhandled_message; "Invalid method")),
        }
    }
}

#[cfg(test)]
mod test {
    use fvm_ipld_encoding::RawBytes;
    use fvm_shared::address::Address;
    use fvm_shared::MethodNum;
    use fil_actors_runtime::{INIT_ACTOR_ADDR};
    use fil_actors_runtime::test_utils::{MockRuntime};
    use crate::{Actor, Method, State, UserPersistParam};

    #[test]
    fn constructor_works() {
        let mut rt = MockRuntime::new(
            Address::new_id(100),
            *INIT_ACTOR_ADDR,
        );

        rt.expect_validate_caller_addr(vec![*INIT_ACTOR_ADDR]);

        rt.call::<Actor>(
            Method::Constructor as MethodNum,
            &RawBytes::serialize(()).unwrap(),
        ).unwrap();

        rt.verify()
    }

    #[test]
    fn persists_works() {
        let mut rt = MockRuntime::new(
            Address::new_id(1),
            *INIT_ACTOR_ADDR,
        );

        rt.expect_validate_caller_addr(vec![*INIT_ACTOR_ADDR]);

        rt.call::<Actor>(
            Method::Constructor as MethodNum,
            &RawBytes::serialize(()).unwrap(),
        ).unwrap();

        // rt.set_caller(Cid::default(), *SYSTEM_ACTOR_ADDR);
        // let raw = RawBytes::serialize(UserPersistParam { name: String::from("sample") }).unwrap();
        // let vec_bytes = Vec::from(raw);
        // println!("{:?}", base64::encode(vec_bytes));

        rt.expect_validate_caller_any();
        rt.call::<Actor>(
            Method::Persist as MethodNum,
            &RawBytes::serialize(UserPersistParam { name: String::from("sample") }).unwrap(),
        ).unwrap();

        rt.verify();
        let state: State = rt.get_state();
        assert_eq!(state.call_count, 1);
    }
}
