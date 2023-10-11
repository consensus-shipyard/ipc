// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// The IPC actors have bindings in `ipc_actors_abis`.
// Here we define stable IDs for them, so we can deploy the
// Solidity contracts during genesis.

use anyhow::Context;
use ethers::core::types as et;
use fendermint_vm_genesis::{Power, Validator};
use ipc_actors_abis as ia;
pub use ipc_actors_abis::gateway_manager_facet::SubnetID;
pub use ipc_actors_abis::gateway_router_facet::BottomUpCheckpoint;
use lazy_static::lazy_static;
use merkle_tree_rs::{
    core::{process_proof, Hash},
    format::Raw,
    standard::{standard_leaf_hash, LeafType, StandardMerkleTree},
};

use crate::{
    diamond::{EthContract, EthContractMap, EthFacet},
    eam::EthAddress,
};

define_id!(GATEWAY { id: 64 });
define_id!(SUBNETREGISTRY { id: 65 });

lazy_static! {
    pub static ref IPC_CONTRACTS: EthContractMap = {
        [
            (
                "GatewayDiamond",
                EthContract {
                    actor_id: GATEWAY_ACTOR_ID,
                    abi: ia::gateway_diamond::GATEWAYDIAMOND_ABI.to_owned(),
                    facets: vec![
                        EthFacet {
                            name: "GatewayGetterFacet",
                            abi: ia::gateway_getter_facet::GATEWAYGETTERFACET_ABI.to_owned(),
                        },
                        EthFacet {
                            name: "GatewayManagerFacet",
                            abi: ia::gateway_manager_facet::GATEWAYMANAGERFACET_ABI.to_owned(),
                        },
                        EthFacet {
                            name: "GatewayRouterFacet",
                            abi: ia::gateway_router_facet::GATEWAYROUTERFACET_ABI.to_owned(),
                        },
                        EthFacet {
                            name: "GatewayMessengerFacet",
                            abi: ia::gateway_messenger_facet::GATEWAYMESSENGERFACET_ABI.to_owned(),
                        },
                    ],
                },
            ),
            (
                "SubnetRegistry",
                EthContract {
                    actor_id: SUBNETREGISTRY_ACTOR_ID,
                    abi: ia::subnet_registry::SUBNETREGISTRY_ABI.to_owned(),
                    // The registry incorporates the SubnetActor facets.
                    facets: vec![
                        EthFacet {
                            name: "SubnetActorGetterFacet",
                            abi: ia::subnet_actor_getter_facet::SUBNETACTORGETTERFACET_ABI
                                .to_owned(),
                        },
                        EthFacet {
                            name: "SubnetActorManagerFacet",
                            abi: ia::subnet_actor_manager_facet::SUBNETACTORMANAGERFACET_ABI
                                .to_owned(),
                        },
                    ],
                },
            ),
        ]
        .into_iter()
        .collect()
    };
}

lazy_static! {
    pub static ref VALIDATOR_TREE_FIELDS: Vec<String> =
        vec!["address".to_owned(), "uint256".to_owned()];
}

/// Construct a Merkle tree from the power table in a format which can be validated by
/// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/cryptography/MerkleProof.sol
///
/// The reference implementation is https://github.com/OpenZeppelin/merkle-tree/
pub struct ValidatorMerkleTree {
    tree: StandardMerkleTree<Raw>,
}

impl ValidatorMerkleTree {
    pub fn new(validators: &[Validator<Power>]) -> anyhow::Result<Self> {
        // Using the 20 byte address for keys because that's what the Solidity library returns
        // when recovering a public key from a signature.
        let values = validators
            .iter()
            .map(Self::validator_to_vec)
            .collect::<anyhow::Result<Vec<_>>>()?;

        let tree = StandardMerkleTree::of(&values, &VALIDATOR_TREE_FIELDS)
            .context("failed to construct Merkle tree")?;

        Ok(Self { tree })
    }

    pub fn root_hash(&self) -> Hash {
        self.tree.root()
    }

    /// Create a Merkle proof for a validator.
    pub fn prove(&self, validator: &Validator<Power>) -> anyhow::Result<Vec<Hash>> {
        let v = Self::validator_to_vec(validator)?;
        let proof = self
            .tree
            .get_proof(LeafType::LeafBytes(v))
            .context("failed to produce Merkle proof")?;
        Ok(proof)
    }

    /// Validate a proof against a known root hash.
    pub fn validate(
        validator: &Validator<Power>,
        root: &Hash,
        proof: &[Hash],
    ) -> anyhow::Result<bool> {
        let v = Self::validator_to_vec(validator)?;
        let h = standard_leaf_hash(v, &VALIDATOR_TREE_FIELDS)?;
        let r = process_proof(&h, proof).context("failed to process Merkle proof")?;
        Ok(*root == r)
    }

    /// Convert a validator to what we can pass to the tree.
    fn validator_to_vec(validator: &Validator<Power>) -> anyhow::Result<Vec<String>> {
        let addr = EthAddress::new_secp256k1(&validator.public_key.0.serialize())?;
        let addr = et::Address::from_slice(&addr.0);
        let addr = format!("{addr:?}");

        let power = et::U256::from(validator.power.0);
        let power = power.to_string();
        Ok(vec![addr, power])
    }
}

pub mod gateway {
    use super::SubnetID;
    use ethers::contract::{EthAbiCodec, EthAbiType};
    use ethers::core::types::{Bytes, H160, U256};
    use fendermint_vm_genesis::ipc::GatewayParams;
    use fendermint_vm_genesis::{Collateral, Validator};
    use fvm_shared::address::Error as AddressError;
    use fvm_shared::address::Payload;
    use fvm_shared::econ::TokenAmount;

    pub use ipc_actors_abis::gateway_getter_facet::Validator as GatewayValidator;

    use crate::eam::{self, EthAddress};

    pub const METHOD_INVOKE_CONTRACT: u64 = crate::evm::Method::InvokeContract as u64;

    // Constructor parameters aren't generated as part of the Rust bindings.
    // TODO: Remove these once https://github.com/gakonst/ethers-rs/pull/2631 is merged.

    /// Container type `ConstructorParameters`.
    ///
    /// See [GatewayDiamond.sol](https://github.com/consensus-shipyard/ipc-solidity-actors/blob/255da67fd6ad885f0ab633311be276a4fa936d45/src/GatewayDiamond.sol#L21)
    #[derive(Clone, EthAbiType, EthAbiCodec, Default, Debug, PartialEq, Eq, Hash)]
    pub struct ConstructorParameters {
        pub network_name: SubnetID,
        pub bottom_up_check_period: u64,
        pub min_collateral: U256,
        pub msg_fee: U256,
        pub majority_percentage: u8,
        pub validators: Vec<GatewayValidator>,
        pub active_validators_limit: u16,
    }

    impl ConstructorParameters {
        pub fn new(
            params: GatewayParams,
            validators: Vec<Validator<Collateral>>,
        ) -> Result<Self, AddressError> {
            // Every step along the way in the subnet ID we have an Ethereum address.
            let mut route = Vec::new();
            for addr in params.subnet_id.children() {
                let addr = match addr.payload() {
                    Payload::ID(id) => EthAddress::from_id(*id),
                    Payload::Delegated(da)
                        if da.namespace() == eam::EAM_ACTOR_ID && da.subaddress().len() == 20 =>
                    {
                        EthAddress(da.subaddress().try_into().expect("checked length"))
                    }
                    _ => return Err(AddressError::InvalidPayload),
                };
                route.push(H160::from(addr.0))
            }

            // Every validator has an Ethereum address.
            let validators = validators
                .into_iter()
                .map(|v| {
                    let pk = v.public_key.0.serialize();
                    let addr = EthAddress::new_secp256k1(&pk)?;
                    let collateral = tokens_to_u256(v.power.0);
                    Ok(GatewayValidator {
                        addr: H160::from(addr.0),
                        weight: collateral,
                        metadata: Bytes::from(pk),
                    })
                })
                .collect::<Result<Vec<_>, AddressError>>()?;

            Ok(Self {
                network_name: SubnetID {
                    root: params.subnet_id.root_id(),
                    route,
                },
                bottom_up_check_period: params.bottom_up_check_period,
                min_collateral: tokens_to_u256(params.min_collateral),
                msg_fee: tokens_to_u256(params.msg_fee),
                majority_percentage: params.majority_percentage,
                validators,
                active_validators_limit: params.active_validators_limit,
            })
        }
    }

    fn tokens_to_u256(value: TokenAmount) -> U256 {
        // XXX: Ignoring any error resulting from larger fee than what fits into U256. This is in genesis after all.
        U256::from_big_endian(&value.atto().to_bytes_be().1)
    }

    #[cfg(test)]
    mod tests {
        use ethers::core::types::{Selector, U256};
        use ethers_core::{
            abi::Tokenize,
            types::{Bytes, H160},
        };
        use fvm_shared::{bigint::BigInt, econ::TokenAmount};
        use ipc_actors_abis::gateway_getter_facet::Validator as GatewayValidator;
        use std::str::FromStr;

        use crate::ipc::tests::{check_param_types, constructor_param_types};

        use super::{tokens_to_u256, ConstructorParameters, SubnetID};

        #[test]
        fn tokenize_constructor_params() {
            let cp = ConstructorParameters {
                network_name: SubnetID {
                    root: 0,
                    route: Vec::new(),
                },
                bottom_up_check_period: 100,
                min_collateral: U256::from(1),
                msg_fee: U256::from(0),
                majority_percentage: 67,
                validators: vec![GatewayValidator {
                    addr: H160::zero(),
                    weight: U256::zero(),
                    metadata: Bytes::new(),
                }],
                active_validators_limit: 100,
            };

            // It looks like if we pass just the record then it will be passed as 5 tokens,
            // but the constructor only expects one parameter, and it has to be a tuple.
            let cp = (Vec::<Selector>::new(), cp);

            let tokens = cp.into_tokens();

            let cons = ipc_actors_abis::gateway_diamond::GATEWAYDIAMOND_ABI
                .constructor()
                .expect("Gateway has a constructor");

            let param_types = constructor_param_types(cons);

            check_param_types(&tokens, &param_types).unwrap();

            cons.encode_input(vec![], &tokens)
                .expect("should encode constructor input");
        }

        #[test]
        #[should_panic]
        fn max_fee_exceeded() {
            let mut value = BigInt::from_str(&U256::MAX.to_string()).unwrap();
            value += 1;
            let value = TokenAmount::from_atto(value);
            let _ = tokens_to_u256(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::bail;
    use ethers_core::abi::{Constructor, ParamType, Token};
    use fendermint_vm_genesis::{Power, Validator};
    use quickcheck_macros::quickcheck;

    use super::ValidatorMerkleTree;

    /// Check all tokens against expected parameters; return any offending one.
    ///
    /// Based on [Tokens::types_check]
    pub fn check_param_types(tokens: &[Token], param_types: &[ParamType]) -> anyhow::Result<()> {
        if param_types.len() != tokens.len() {
            bail!(
                "different number of parameters; expected {}, got {}",
                param_types.len(),
                tokens.len()
            );
        }

        for (i, (pt, t)) in param_types.iter().zip(tokens).enumerate() {
            if !t.type_check(pt) {
                bail!("parameter {i} didn't type check: expected {pt:?}, got {t:?}");
            }
        }

        Ok(())
    }

    /// Returns all input params of given constructor.
    ///
    /// Based on [Constructor::param_types]
    pub fn constructor_param_types(cons: &Constructor) -> Vec<ParamType> {
        cons.inputs.iter().map(|p| p.kind.clone()).collect()
    }

    #[quickcheck]
    fn merkleize_validators(validators: Vec<Validator<Power>>) {
        if validators.is_empty() {
            return;
        }

        let tree = ValidatorMerkleTree::new(&validators).expect("failed to create tree");
        let root = tree.root_hash();

        let validator = validators.first().unwrap();
        let proof = tree.prove(validator).expect("failed to prove");

        assert!(ValidatorMerkleTree::validate(validator, &root, &proof).expect("failed to validate"))
    }
}
