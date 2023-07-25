//! Type conversion for IPC Agent struct with solidity contract struct

use crate::manager::evm::manager::{
    subnet_actor_getter_facet, subnet_actor_manager_facet,
};
use ethers::abi::Token;
use ethers::types::U256;
use fvm_shared::address::{Address, Payload};

/// The type conversion for IPC structs to evm solidity contracts. We need this convenient macro because
/// the abigen is creating the same struct but under different modules. This save a lot of
/// code.
macro_rules! type_conversion {
    ($module:ident) => {
        impl From<Address> for $module::FvmAddress {
            fn from(value: Address) -> Self {
                $module::FvmAddress {
                    addr_type: value.protocol() as u8,
                    payload: addr_payload_to_bytes(value.into_payload()),
                }
            }
        }
    };
}

type_conversion!(subnet_actor_getter_facet);
type_conversion!(subnet_actor_manager_facet);

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
