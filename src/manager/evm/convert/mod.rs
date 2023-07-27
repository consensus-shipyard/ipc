// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Type conversion for IPC Agent struct with solidity contract struct

mod checkpoint;

use crate::manager::evm::manager::agent_subnet_to_evm_addresses;
use crate::manager::evm::manager::{
    gateway_getter_facet, gateway_manager_facet, gateway_router_facet, subnet_actor_getter_facet,
    subnet_actor_manager_facet,
};
use crate::manager::SubnetInfo;
use anyhow::anyhow;
use ethers::abi::{ParamType, Token};
use ethers::types::U256;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::{Address, Payload};
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_gateway::{CrossMsg, Status, StorableMsg};
use ipc_sdk::address::IPCAddress;
use ipc_sdk::subnet_id::SubnetID;
use primitives::EthAddress;
use std::str::FromStr;

/// The type conversion for IPC structs to evm solidity contracts. We need this convenient macro because
/// the abigen is creating the same struct but under different modules. This save a lot of
/// code.
macro_rules! base_type_conversion {
    ($module:ident) => {
        impl From<Address> for $module::FvmAddress {
            fn from(value: Address) -> Self {
                $module::FvmAddress {
                    addr_type: value.protocol() as u8,
                    payload: addr_payload_to_bytes(value.into_payload()),
                }
            }
        }

        impl TryFrom<$module::FvmAddress> for Address {
            type Error = anyhow::Error;

            fn try_from(value: $module::FvmAddress) -> Result<Self, Self::Error> {
                let protocol = value.addr_type;
                let addr = bytes_to_fvm_addr(protocol, &value.payload)?;
                Ok(addr)
            }
        }

        impl TryFrom<&SubnetID> for $module::SubnetID {
            type Error = anyhow::Error;

            fn try_from(subnet: &SubnetID) -> Result<Self, Self::Error> {
                Ok($module::SubnetID {
                    root: subnet.root_id(),
                    route: agent_subnet_to_evm_addresses(subnet)?,
                })
            }
        }

        impl TryFrom<$module::SubnetID> for SubnetID {
            type Error = anyhow::Error;

            fn try_from(value: $module::SubnetID) -> Result<Self, Self::Error> {
                let children = value
                    .route
                    .iter()
                    .map(ethers_address_to_fil_address)
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(SubnetID::new(value.root, children))
            }
        }
    };
}

/// Implement the cross network message types. To use this macro, make sure the $module has already
/// implemented the base types.
macro_rules! cross_msg_types {
    ($module:ident) => {
        impl TryFrom<IPCAddress> for $module::Ipcaddress {
            type Error = anyhow::Error;

            fn try_from(value: IPCAddress) -> Result<Self, Self::Error> {
                Ok($module::Ipcaddress {
                    subnet_id: $module::SubnetID::try_from(&value.subnet()?)?,
                    raw_address: $module::FvmAddress::try_from(value.raw_addr()?)?,
                })
            }
        }

        impl TryFrom<$module::Ipcaddress> for IPCAddress {
            type Error = anyhow::Error;

            fn try_from(value: $module::Ipcaddress) -> Result<Self, Self::Error> {
                let addr = Address::try_from(value.raw_address)?;
                let i = IPCAddress::new(&SubnetID::try_from(value.subnet_id)?, &addr)?;
                Ok(i)
            }
        }

        impl TryFrom<StorableMsg> for $module::StorableMsg {
            type Error = anyhow::Error;

            fn try_from(value: StorableMsg) -> Result<Self, Self::Error> {
                let msg_value = fil_to_eth_amount(&value.value)?;

                log::info!(
                    "storable message token amount: {:}, converted: {:?}",
                    value.value.atto().to_string(),
                    msg_value
                );

                let c = $module::StorableMsg {
                    from: $module::Ipcaddress::try_from(value.from).map_err(|e| {
                        anyhow!("cannot convert `from` ipc address msg due to: {e:}")
                    })?,
                    to: $module::Ipcaddress::try_from(value.to)
                        .map_err(|e| anyhow!("cannot convert `to`` ipc address due to: {e:}"))?,
                    value: msg_value,
                    nonce: value.nonce,
                    // FIXME: we might a better way to handle the encoding of methods and params according to the type of message the cross-net message is targetting.
                    method: (value.method as u32).to_be_bytes(),
                    params: ethers::core::types::Bytes::from(value.params.to_vec()),
                };
                Ok(c)
            }
        }

        impl TryFrom<$module::StorableMsg> for StorableMsg {
            type Error = anyhow::Error;

            fn try_from(value: $module::StorableMsg) -> Result<Self, Self::Error> {
                let s = StorableMsg {
                    from: IPCAddress::try_from(value.from)?,
                    to: IPCAddress::try_from(value.to)?,
                    method: u32::from_be_bytes(value.method) as MethodNum,
                    params: RawBytes::from(value.params.to_vec()),
                    value: eth_to_fil_amount(&value.value)?,
                    nonce: value.nonce,
                };
                Ok(s)
            }
        }

        impl TryFrom<CrossMsg> for $module::CrossMsg {
            type Error = anyhow::Error;

            fn try_from(value: CrossMsg) -> Result<Self, Self::Error> {
                let c = $module::CrossMsg {
                    wrapped: value.wrapped,
                    message: $module::StorableMsg::try_from(value.msg)
                        .map_err(|e| anyhow!("cannot convert storable msg due to: {e:}"))?,
                };
                Ok(c)
            }
        }

        impl TryFrom<$module::CrossMsg> for CrossMsg {
            type Error = anyhow::Error;

            fn try_from(value: $module::CrossMsg) -> Result<Self, Self::Error> {
                let c = CrossMsg {
                    wrapped: value.wrapped,
                    msg: StorableMsg::try_from(value.message)?,
                };
                Ok(c)
            }
        }
    };
}

base_type_conversion!(gateway_router_facet);
base_type_conversion!(subnet_actor_getter_facet);
base_type_conversion!(gateway_manager_facet);
base_type_conversion!(subnet_actor_manager_facet);
base_type_conversion!(gateway_getter_facet);

cross_msg_types!(subnet_actor_manager_facet);
cross_msg_types!(subnet_actor_getter_facet);
cross_msg_types!(gateway_getter_facet);
cross_msg_types!(gateway_router_facet);

impl TryFrom<gateway_getter_facet::Subnet> for SubnetInfo {
    type Error = anyhow::Error;

    fn try_from(value: gateway_getter_facet::Subnet) -> Result<Self, Self::Error> {
        Ok(SubnetInfo {
            id: SubnetID::try_from(value.id)?,
            stake: eth_to_fil_amount(&value.stake)?,
            circ_supply: eth_to_fil_amount(&value.circ_supply)?,
            status: match value.status {
                1 => Status::Active,
                2 => Status::Inactive,
                3 => Status::Killed,
                _ => return Err(anyhow!("invalid status: {:}", value.status)),
            },
        })
    }
}

/// Converts a Rust type FVM address into its underlying payload
/// so it can be represented internally in a Solidity contract.
fn addr_payload_to_bytes(payload: Payload) -> ethers::types::Bytes {
    match payload {
        Payload::Secp256k1(v) => ethers::types::Bytes::from(v),
        Payload::Delegated(d) => {
            let addr = d.subaddress();
            let b = ethers::abi::encode(&[Token::Tuple(vec![
                Token::Uint(U256::from(d.namespace())),
                Token::Uint(U256::from(addr.len())),
                Token::Bytes(addr.to_vec()),
            ])]);
            ethers::types::Bytes::from(b)
        }
        _ => unimplemented!(),
    }
}

/// It takes the bytes from an FVMAddress represented in Solidity and
/// converts it into the corresponding FVM address Rust type.
fn bytes_to_fvm_addr(protocol: u8, bytes: &[u8]) -> anyhow::Result<Address> {
    let addr = match protocol {
        1 => Address::from_bytes(&[[1u8].as_slice(), bytes].concat())?,
        4 => {
            let mut data = ethers::abi::decode(
                &[ParamType::Tuple(vec![
                    ParamType::Uint(32),
                    ParamType::Uint(32),
                    ParamType::Bytes,
                ])],
                bytes,
            )?;

            let mut data = data
                .pop()
                .ok_or_else(|| anyhow!("invalid tuple data length"))?
                .into_tuple()
                .ok_or_else(|| anyhow!("not tuple"))?;

            let raw_bytes = data
                .pop()
                .ok_or_else(|| anyhow!("invalid length, should be 3"))?
                .into_bytes()
                .ok_or_else(|| anyhow!("invalid bytes"))?;
            let len = data
                .pop()
                .ok_or_else(|| anyhow!("invalid length, should be 3"))?
                .into_uint()
                .ok_or_else(|| anyhow!("invalid uint"))?
                .as_u128();
            let namespace = data
                .pop()
                .ok_or_else(|| anyhow!("invalid length, should be 3"))?
                .into_uint()
                .ok_or_else(|| anyhow!("invalid uint"))?
                .as_u64();

            if len as usize != raw_bytes.len() {
                return Err(anyhow!("bytes len not match"));
            }
            Address::new_delegated(namespace, &raw_bytes)?
        }
        _ => return Err(anyhow!("address not support now")),
    };
    Ok(addr)
}

/// Converts a Fil TokenAmount into an ethers::U256 amount.
pub fn fil_to_eth_amount(amount: &TokenAmount) -> anyhow::Result<U256> {
    let str = amount.atto().to_string();
    Ok(U256::from_dec_str(&str)?)
}

/// Converts an ethers::U256 TokenAmount into a FIL amount.
pub fn eth_to_fil_amount(amount: &U256) -> anyhow::Result<TokenAmount> {
    let v = BigInt::from_str(&amount.to_string())?;
    Ok(TokenAmount::from_atto(v))
}

pub fn ethers_address_to_fil_address(addr: &ethers::types::Address) -> anyhow::Result<Address> {
    let raw_addr = format!("{addr:?}");
    log::debug!("raw evm subnet addr: {raw_addr:}");

    let eth_addr = EthAddress::from_str(&raw_addr)?;
    Ok(Address::from(eth_addr))
}
