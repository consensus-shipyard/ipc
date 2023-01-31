// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use fvm_shared::{MethodNum, METHOD_SEND};

use crate::runtime::Runtime;
use crate::{actor_error, ActorError};

pub const HAMT_BIT_WIDTH: u32 = 5;

/// ResolveToIDAddr resolves the given address to it's ID address form.
/// If an ID address for the given address dosen't exist yet, it tries to create one by sending
/// a zero balance to the given address.
pub fn resolve_to_id_addr(rt: &mut impl Runtime, address: &Address) -> anyhow::Result<Address> {
    // if we are able to resolve it to an ID address, return the resolved address
    if let Some(addr) = rt.resolve_address(address) {
        return Ok(addr);
    }

    // send 0 balance to the account so an ID address for it is created and then try to resolve
    rt.send(address, METHOD_SEND, Default::default(), Default::default())
        .map_err(|e| e.wrap(format!("failed to send zero balance to address {address}",)))?;

    rt.resolve_address(address).ok_or_else(|| {
        anyhow::anyhow!(
            "failed to resolve address {} to ID address even after sending zero balance",
            address,
        )
    })
}

// The lowest FRC-42 method number.
pub const FIRST_EXPORTED_METHOD_NUMBER: MethodNum = 1 << 24;

// Checks whether the caller is allowed to invoke some method number.
// All method numbers below the FRC-42 range are restricted to built-in actors
// (including the account and multisig actors).
// Methods may subsequently enforce tighter restrictions.
pub fn restrict_internal_api<RT>(rt: &mut RT, method: MethodNum) -> Result<(), ActorError>
where
    RT: Runtime,
{
    if method >= FIRST_EXPORTED_METHOD_NUMBER {
        return Ok(());
    }
    let caller = rt.message().caller();
    let code_cid = rt.get_actor_code_cid(&caller.id().unwrap());
    match code_cid {
        None => {
            return Err(
                actor_error!(forbidden; "no code for caller {} of method {}", caller, method),
            )
        }
        Some(code_cid) => {
            let builtin_type = rt.resolve_builtin_actor_type(&code_cid);
            if builtin_type.is_none() {
                return Err(
                    actor_error!(forbidden; "caller {} of method {} must be built-in", caller, method),
                );
            }
        }
    }
    Ok(())
}
