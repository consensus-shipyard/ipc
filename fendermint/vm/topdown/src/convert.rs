// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Handles the type conversion to ethers contract types

use crate::IPCParentFinality;
use anyhow::anyhow;
use ethers::abi::Function;
use ethers::types::U256;
use ipc_actors_abis::{gateway_getter_facet, gateway_router_facet};

const GET_LATEST_PARENT_FINALITY_FUNC_NAME: &str = "getLatestParentFinality";

impl TryFrom<IPCParentFinality> for gateway_router_facet::ParentFinality {
    type Error = anyhow::Error;

    fn try_from(value: IPCParentFinality) -> Result<Self, Self::Error> {
        if value.block_hash.len() != 32 {
            return Err(anyhow!("invalid block hash length, expecting 32"));
        }

        let mut block_hash = [0u8; 32];
        block_hash.copy_from_slice(&value.block_hash[0..32]);

        Ok(Self {
            height: U256::from(value.height),
            block_hash,
        })
    }
}

impl From<gateway_getter_facet::ParentFinality> for IPCParentFinality {
    fn from(value: gateway_getter_facet::ParentFinality) -> Self {
        IPCParentFinality {
            height: value.height.as_u64(),
            block_hash: value.block_hash.to_vec(),
        }
    }
}

pub fn encode_get_latest_parent_finality() -> anyhow::Result<Vec<u8>> {
    let function = get_evm_function(GET_LATEST_PARENT_FINALITY_FUNC_NAME)?;
    let data = ethers::contract::encode_function_data(function, ())?;

    Ok(data.to_vec())
}

pub fn decode_parent_finality_return(bytes: &[u8]) -> anyhow::Result<IPCParentFinality> {
    let function = get_evm_function(GET_LATEST_PARENT_FINALITY_FUNC_NAME)?;
    let finality = ethers::contract::decode_function_data::<gateway_getter_facet::ParentFinality, _>(
        function, bytes, false,
    )?;
    Ok(IPCParentFinality::from(finality))
}

fn get_evm_function(method_name: &str) -> anyhow::Result<&Function> {
    gateway_getter_facet::GATEWAYGETTERFACET_ABI
        .functions
        .get(method_name)
        .ok_or_else(|| anyhow!("report bug, abi function map does not have {}", method_name))?
        .get(0)
        .ok_or_else(|| anyhow!("report bug, abi vec does not have {}", method_name))
}
