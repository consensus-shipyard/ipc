// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{bytes::B256, method::Method};
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::MethodNum;
use recall_actor_sdk::evm::{InputData, InvokeContractParams, InvokeContractReturn};

use crate::{
    sol_facade::{blobs as sol_blobs, credit as sol_credit, AbiCall, AbiCallRuntime},
    State, BLOBS_ACTOR_NAME,
};

mod admin;
mod metrics;
mod system;
mod user;

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(BlobsActor);

/// Singleton actor for managing blob storage.
///
/// The [`Address`]es stored in this actor's state _must_ be ID-based addresses for
/// efficient comparison with message origin and caller addresses, which are always ID-based.
/// [`Address`]es in the method params can be of any type.
/// They will be resolved to ID-based addresses.
///
/// For simplicity, this actor currently manages both blobs and credit.
/// A future version of the protocol will likely separate them in some way.
pub struct BlobsActor;

impl BlobsActor {
    /// Creates a new [`BlobsActor`] state.
    ///
    /// This is only used in tests. This actor is created manually at genesis.
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(rt.store())?;
        rt.create(&state)
    }

    /// Invokes actor methods with EVM calldata.
    fn invoke_contract(
        rt: &impl Runtime,
        params: InvokeContractParams,
    ) -> Result<InvokeContractReturn, ActorError> {
        let input_data: InputData = params.try_into()?;
        if sol_blobs::can_handle(&input_data) {
            let output_data = match sol_blobs::parse_input(&input_data)? {
                sol_blobs::Calls::addBlob(call) => {
                    let params = call.params(rt)?;
                    Self::add_blob(rt, params)?;
                    call.returns(())
                }
                sol_blobs::Calls::deleteBlob(call) => {
                    let params = call.params(rt)?;
                    Self::delete_blob(rt, params)?;
                    call.returns(())
                }
                sol_blobs::Calls::getBlob(call) => {
                    let params = call.params()?;
                    let blob = Self::get_blob(rt, params)?;
                    call.returns(blob)?
                }
                sol_blobs::Calls::getStats(call) => {
                    let stats = Self::get_stats(rt)?;
                    call.returns(stats)
                }
                sol_blobs::Calls::overwriteBlob(call) => {
                    let params = call.params(rt)?;
                    Self::overwrite_blob(rt, params)?;
                    call.returns(())
                }
                sol_blobs::Calls::trimBlobExpiries(call) => {
                    let params = call.params();
                    let cursor = Self::trim_blob_expiries(rt, params)?;
                    call.returns(cursor)
                }
            };
            Ok(InvokeContractReturn { output_data })
        } else if sol_credit::can_handle(&input_data) {
            let output_data = match sol_credit::parse_input(&input_data)? {
                sol_credit::Calls::buyCredit_0(call) => {
                    // function buyCredit() external payable;
                    let params = call.params(rt);
                    Self::buy_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::buyCredit_1(call) => {
                    // function buyCredit(address recipient) external payable;
                    let params = call.params();
                    Self::buy_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::approveCredit_0(call) => {
                    let params = call.params();
                    Self::approve_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::approveCredit_1(call) => {
                    let params = call.params();
                    Self::approve_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::approveCredit_2(call) => {
                    let params = call.params();
                    Self::approve_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::revokeCredit_0(call) => {
                    let params = call.params();
                    Self::revoke_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::revokeCredit_1(call) => {
                    let params = call.params();
                    Self::revoke_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::setAccountSponsor(call) => {
                    let params = call.params();
                    Self::set_account_sponsor(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::getAccount(call) => {
                    let params = call.params();
                    let account_info = Self::get_account(rt, params)?;
                    call.returns(account_info)?
                }
                sol_credit::Calls::getCreditApproval(call) => {
                    let params = call.params();
                    let credit_approval = Self::get_credit_approval(rt, params)?;
                    call.returns(credit_approval)
                }
                sol_credit::Calls::setAccountStatus(call) => {
                    let params = call.params()?;
                    Self::set_account_status(rt, params)?;
                    call.returns(())
                }
            };
            Ok(InvokeContractReturn { output_data })
        } else {
            Err(actor_error!(illegal_argument, "invalid call".to_string()))
        }
    }

    /// Fallback method for unimplemented method numbers.
    fn fallback(
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

impl ActorCode for BlobsActor {
    type Methods = Method;

    fn name() -> &'static str {
        BLOBS_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,

        // EVM interop
        InvokeContract => invoke_contract,

        // User methods
        BuyCredit => buy_credit,
        ApproveCredit => approve_credit,
        RevokeCredit => revoke_credit,
        SetAccountSponsor => set_account_sponsor,
        GetAccount => get_account,
        GetCreditApproval => get_credit_approval,
        AddBlob => add_blob,
        GetBlob => get_blob,
        DeleteBlob => delete_blob,
        OverwriteBlob => overwrite_blob,

        // System methods
        GetGasAllowance => get_gas_allowance,
        UpdateGasAllowance => update_gas_allowance,
        GetBlobStatus => get_blob_status,
        GetAddedBlobs => get_added_blobs,
        GetPendingBlobs => get_pending_blobs,
        SetBlobPending => set_blob_pending,
        FinalizeBlob => finalize_blob,
        DebitAccounts => debit_accounts,

        // Admin methods
        SetAccountStatus => set_account_status,
        TrimBlobExpiries => trim_blob_expiries,

        // Metrics methods
        GetStats => get_stats,

        _ => fallback,
    }
}

/// Makes a syscall that will delete a blob from the underlying Iroh-based data store.
fn delete_from_disc(hash: B256) -> Result<(), ActorError> {
    #[cfg(feature = "fil-actor")]
    {
        recall_actor_sdk::storage::delete_blob(hash.0).map_err(|en| {
            ActorError::unspecified(format!("failed to delete blob from disc: {:?}", en))
        })?;
        log::debug!("deleted blob {} from disc", hash);
        Ok(())
    }
    #[cfg(not(feature = "fil-actor"))]
    {
        log::debug!("mock deletion from disc (hash={})", hash);
        Ok(())
    }
}
