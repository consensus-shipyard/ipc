//! Type conversion for IPC Agent struct with solidity contract struct

use crate::manager::evm::manager::agent_subnet_to_evm_addresses;
use crate::manager::evm::manager::{
    gateway_getter_facet, gateway_manager_facet, subnet_actor_getter_facet,
    subnet_actor_manager_facet,
};
use anyhow::anyhow;
use ethers::abi::{ParamType, Token};
use ethers::types::U256;
use fvm_shared::address::{Address, Payload};
use ipc_sdk::subnet_id::SubnetID;

/// The type conversion for IPC structs to evm solidity contracts. We need this convenient macro because
/// the abigen is creating the same struct but under different modules. This save a lot of
/// code.
macro_rules! common_type_conversion {
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
    };
}

common_type_conversion!(subnet_actor_getter_facet);
common_type_conversion!(subnet_actor_manager_facet);
common_type_conversion!(gateway_manager_facet);
common_type_conversion!(gateway_getter_facet);

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
