// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::params::{
    AddBlobParams, BuyCreditParams, DeleteBlobParams, FinalizeBlobParams, GetAccountParams,
    GetBlobParams, GetStatsReturn,
};
use fendermint_actor_blobs_shared::state::{Account, Blob, BlobStatus, Hash, PublicKey};
use fendermint_actor_blobs_shared::Method;
use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::{
    actor_dispatch, actor_error, deserialize_block,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, AsActorError, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::sys::SendFlags;
use fvm_shared::{error::ExitCode, MethodNum};
use num_traits::Zero;
use std::collections::{BTreeMap, HashSet};

use crate::{ext, ConstructorParams, State, BLOBS_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(BlobsActor);

// TODO: cron to debit and handle expiry
pub struct BlobsActor;

impl BlobsActor {
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(params.capacity, params.debit_rate)
            .map_err(to_state_error("failed to construct empty store"))?;
        rt.create(&state)
    }

    fn get_stats(rt: &impl Runtime) -> Result<GetStatsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?
            .get_stats(rt.current_balance())
            .map_err(to_state_error("failed to get stats"))
    }

    fn buy_credit(rt: &impl Runtime, params: BuyCreditParams) -> Result<Account, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.transaction(|st: &mut State, rt| {
            st.buy_credit(params.0, rt.message().value_received(), rt.curr_epoch())
                .map_err(to_state_error("failed to buy credit"))
        })
    }

    fn get_account(
        rt: &impl Runtime,
        params: GetAccountParams,
    ) -> Result<Option<Account>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?
            .get_account(params.0)
            .map_err(to_state_error("failed to get account"))
    }

    fn add_blob(rt: &impl Runtime, params: AddBlobParams) -> Result<Account, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let caller = resolve_caller(rt, params.from)?;
        rt.transaction(|st: &mut State, rt| {
            st.add_blob(
                caller,
                rt.curr_epoch(),
                params.hash,
                params.size,
                params.ttl,
                params.source,
            )
            .map_err(to_state_error("failed to add blob"))
        })
    }

    fn get_blob(rt: &impl Runtime, params: GetBlobParams) -> Result<Option<Blob>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?
            .get_blob(params.0)
            .map_err(to_state_error("failed to get blob"))
    }

    fn get_blob_status(
        rt: &impl Runtime,
        params: GetBlobParams,
    ) -> Result<Option<BlobStatus>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?
            .get_blob(params.0)
            .map_err(to_state_error("failed to get blob"))
            .map(|b| b.map(|b| b.status))
    }

    // TODO: limit return via param
    fn get_pending_blobs(
        rt: &impl Runtime,
    ) -> Result<BTreeMap<Hash, HashSet<PublicKey>>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?
            .get_pending_blobs()
            .map_err(to_state_error("failed to get resolving blobs"))
    }

    fn finalize_blob(rt: &impl Runtime, params: FinalizeBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        rt.transaction(|st: &mut State, _| {
            st.finalize_blob(params.hash, params.status)
                .map_err(to_state_error("failed to finalize blob"))
        })
    }

    fn delete_blob(rt: &impl Runtime, params: DeleteBlobParams) -> Result<Account, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let caller = resolve_caller(rt, params.from)?;
        let (account, delete) = rt.transaction(|st: &mut State, _| {
            st.delete_blob(caller, rt.curr_epoch(), params.hash)
                .map_err(to_state_error("failed to delete blob"))
        })?;
        if delete {
            // TODO: make syscall to delete blob from Iroh
        }
        Ok(account)
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

fn to_state_error(message: &'static str) -> impl FnOnce(anyhow::Error) -> ActorError {
    move |e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, message)
}

impl ActorCode for BlobsActor {
    type Methods = Method;

    fn name() -> &'static str {
        BLOBS_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        GetStats => get_stats,
        BuyCredit => buy_credit,
        GetAccount => get_account,
        AddBlob => add_blob,
        GetBlob => get_blob,
        GetBlobStatus => get_blob_status,
        GetPendingBlobs => get_pending_blobs,
        FinalizeBlob => finalize_blob,
        DeleteBlob => delete_blob,
        _ => fallback,
    }
}

/// Resolves caller address to robust (non-ID) address for safe storage
fn resolve_caller(rt: &impl Runtime, from: Option<Address>) -> Result<Address, ActorError> {
    if let Some(machine) = from {
        match rt.resolve_address(&machine) {
            Some(id) => {
                // Caller is always an ID address
                if id == rt.message().caller().id().unwrap() {
                    Ok(machine)
                } else {
                    Err(ActorError::illegal_argument(
                        "machine address does not match caller".into(),
                    ))
                }
            }
            None => Err(ActorError::not_found("machine address not found".into())),
        }
    } else {
        resolve_caller_external(rt)
    }
}

fn resolve_caller_external(rt: &impl Runtime) -> Result<Address, ActorError> {
    let caller = rt.message().caller();
    let caller_id = caller.id().unwrap();
    let caller_code_cid = rt
        .get_actor_code_cid(&caller_id)
        .expect("failed to lookup caller code");
    match rt.resolve_builtin_actor_type(&caller_code_cid) {
        Some(Type::Account) => {
            let result = rt
                .send(
                    &caller,
                    ext::account::PUBKEY_ADDRESS_METHOD,
                    None,
                    Zero::zero(),
                    None,
                    SendFlags::READ_ONLY,
                )
                .context_code(
                    ExitCode::USR_ASSERTION_FAILED,
                    "account failed to return its key address",
                )?;

            if !result.exit_code.is_success() {
                return Err(ActorError::checked(
                    result.exit_code,
                    "failed to retrieve account robust address".to_string(),
                    None,
                ));
            }
            let robust_addr: Address = deserialize_block(result.return_data)?;

            Ok(robust_addr)
        }
        Some(Type::EthAccount) => {
            if let Some(delegated_addr) = rt.lookup_delegated_address(caller_id) {
                Ok(delegated_addr)
            } else {
                Err(ActorError::forbidden(format!(
                    "actor {} does not have delegated address",
                    caller_id
                )))
            }
        }
        Some(t) => Err(ActorError::forbidden(format!(
            "disallowed caller type {}",
            t.name()
        ))),
        None => Err(ActorError::forbidden(format!(
            "disallowed caller code {caller_code_cid}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::bigint::BigInt;
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::econ::TokenAmount;
    use rand::RngCore;

    pub fn new_hash(size: usize) -> (Hash, u64) {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; size];
        rng.fill_bytes(&mut data);
        (
            Hash(*iroh_base::hash::Hash::new(&data).as_bytes()),
            size as u64,
        )
    }

    pub fn new_pk() -> PublicKey {
        let mut rng = rand::thread_rng();
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);
        PublicKey(data)
    }

    pub fn construct_and_verify(capacity: u64, debit_rate: u64) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let result = rt
            .call::<BlobsActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    capacity,
                    debit_rate,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();
        rt
    }

    #[test]
    fn test_fund_account() {
        let rt = construct_and_verify(1024 * 1024, 1);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let mut expected_credits = BigInt::from(1000000000000000000u64);
        rt.set_received(TokenAmount::from_whole(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        let result = rt
            .call::<BlobsActor>(
                Method::BuyCredit as u64,
                IpldBlock::serialize_cbor(&fund_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Account>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        rt.verify();

        expected_credits += BigInt::from(1000000000u64);
        rt.set_received(TokenAmount::from_nano(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        let result = rt
            .call::<BlobsActor>(
                Method::BuyCredit as u64,
                IpldBlock::serialize_cbor(&fund_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Account>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        rt.verify();

        expected_credits += BigInt::from(1u64);
        rt.set_received(TokenAmount::from_atto(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        let result = rt
            .call::<BlobsActor>(
                Method::BuyCredit as u64,
                IpldBlock::serialize_cbor(&fund_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Account>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        rt.verify();
    }

    #[test]
    fn test_add_blob() {
        let rt = construct_and_verify(1024 * 1024, 1);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.set_epoch(ChainEpoch::from(0));

        // Try without first funding
        rt.expect_validate_caller_any();
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            from: None,
            source: new_pk(),
            hash: hash.0,
            size: hash.1,
            ttl: 3600,
        };
        let result = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();

        // Fund an account
        rt.set_received(TokenAmount::from_whole(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        let result = rt.call::<BlobsActor>(
            Method::BuyCredit as u64,
            IpldBlock::serialize_cbor(&fund_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Try with sufficient balance
        rt.set_epoch(ChainEpoch::from(5));
        rt.expect_validate_caller_any();
        let account = rt
            .call::<BlobsActor>(
                Method::AddBlob as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Account>()
            .unwrap();
        assert_eq!(
            account,
            Account {
                capacity_used: BigInt::from(1024),
                credit_free: BigInt::from(999999999996313600u64),
                credit_committed: BigInt::from(3686400),
                last_debit_epoch: 5,
            }
        );
        rt.verify();
    }
}
