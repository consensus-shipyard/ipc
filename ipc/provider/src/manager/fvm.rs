// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::manager::evm::dry_run::EvmDryRun;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_api::subnet::ConstructParams;
use num_traits::Zero;
use serde::Serialize;

const INVOKE_CONTRACT: MethodNum = 3844450837;

#[derive(Serialize)]
pub struct MockedTxn {
    from: Address,
    to: Address,
    value: TokenAmount,
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
            value: TokenAmount::zero(),
            method: INVOKE_CONTRACT,
            method_params,
        })
    }
}

pub fn to_fvm_calldata(evm_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let params = RawBytes::serialize(BytesSer(evm_data))?;
    Ok(params.to_vec())
}
