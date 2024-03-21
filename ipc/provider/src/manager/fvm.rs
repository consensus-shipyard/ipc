// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::config::serialize::{serialize_address_to_str, serialize_bytes_to_str};
use crate::manager::evm::dry_run::EvmDryRun;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_api::subnet::ConstructParams;
use serde::Serialize;

const INVOKE_CONTRACT: MethodNum = 3844450837;

#[derive(Serialize)]
pub struct MockedTxn {
    #[serde(serialize_with = "serialize_address_to_str")]
    from: Address,
    #[serde(serialize_with = "serialize_address_to_str")]
    to: Address,
    /// The value, display as string to align with evm display
    value: String,
    #[serde(serialize_with = "serialize_bytes_to_str")]
    method_params: Vec<u8>,
    method: MethodNum,
}

pub struct FvmDryRun {
    pub(crate) evm: EvmDryRun,
}

impl FvmDryRun {
    pub fn create_subnet(
        &self,
        from: &Address,
        params: ConstructParams,
    ) -> anyhow::Result<MockedTxn> {
        let to = params.ipc_gateway_addr.unwrap();

        let method_params = to_fvm_calldata(&self.evm.create_subnet(from, params)?.calldata)?;

        Ok(MockedTxn {
            from: *from,
            to,
            value: TokenAmount::from_atto(0).to_string(),
            method: INVOKE_CONTRACT,
            method_params,
        })
    }
}

pub fn to_fvm_calldata(evm_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let params = RawBytes::serialize(BytesSer(evm_data))?;
    Ok(params.to_vec())
}
