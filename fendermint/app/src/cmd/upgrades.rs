// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ethers::abi::Token;
use ethers::types::{Address, U256};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_interpreter::fvm::state::fevm::ContractCaller;
use fendermint_vm_interpreter::fvm::upgrades::{Upgrade, UpgradeScheduler};
use fvm_ipld_blockstore::Blockstore;
use ipc_actors_abis::ownership_facet::{OwnershipFacet, OwnershipFacetErrors};
use ipc_actors_abis::top_down_finality_facet::{StakingChange, StakingChangeRequest};
use ipc_actors_abis::top_down_finality_facet::{TopDownFinalityFacet, TopDownFinalityFacetErrors};
use std::str::FromStr;
use tracing::info;

pub fn create_upgrade_scheduler<DB: Blockstore + 'static + Clone>() -> UpgradeScheduler<DB> {
    let mut upgrade_scheduler = UpgradeScheduler::new();

    // update the to actual chain id
    let chain_id = 901861227013395.into();

    // transfer ownership of the gateway to target address
    upgrade_scheduler
        .add(Upgrade::new_by_id(chain_id, 50u64, None, |state| {
            // update to the actual new owner address
            let new_owner =
                ethers::types::Address::from_str("0x1A79385eAd0e873FE0C441C034636D3Edf7014cC")
                    .expect("invalid new owner address");

            // confirm the existing owner is this address
            let cur_owner =
                ethers::types::Address::from_str("0xfF00000000000000000000000000000000000000")
                    .expect("invalid address");

            let gateway_addr = EthAddress::from(
                ethers::types::Address::from_str("0x77aa40b105843728088c0132e43fc44348881da8")
                    .expect("invalid gateway addr"),
            );

            info!(
                "[Upgrade at height {}] Change gateway ownership",
                state.block_height()
            );

            let ownership = ContractCaller::<_, _, OwnershipFacetErrors>::new(
                gateway_addr,
                OwnershipFacet::new,
            );
            ownership.call_with_return(state, |c| {
                let mut call = c.transfer_ownership(new_owner);
                call = call.from(cur_owner);
                call
            })?;
            let owner = ownership.call(state, |c| c.owner())?;
            info!(owner = owner.to_string(), "updated gateway ownership");

            Ok(())
        }))
        .expect("cannot add gateway ownership upgrade");

    // applied missing validator changes
    upgrade_scheduler
        .add(Upgrade::new_by_id(chain_id, 60u64, None, |state| {
            // The set federated power opt enum
            let op = 3;

            struct SetFederatedPower {
                public_key: Vec<u8>,
                validator: Address,
                power: U256,
                configuration_number: u64,
            }

            let validator_powers = vec![
                SetFederatedPower {
                    public_key: hex::decode("047efe505fb55f56756514db73ff1e3a8d7fc08f7c5bbc3cbf10d646be71c2593766d6a8785f468ed6701c427d9b2a6a8d8a7d7146bc77a7e7a94c49bbcbd39f7f").expect("invalid public key"),
                    validator: Address::from_str("1A79385eAd0e873FE0C441C034636D3Edf7014cC").expect("invalid address"),
                    power: U256::from(1u128),
                    configuration_number: 10,
                },
                SetFederatedPower {
                    public_key: hex::decode("0438f218c19b4237812313c8418d8ae95920b1fd8a70404416949dbc1be235794d84373a6560c9195e3ad0584875fbb3d7798b550c6f6bd989190c7252fd25732b").expect("invalid public key"),
                    validator: Address::from_str("f82e4a50afec9ee1e9148f75bed61e9a13291e50").expect("invalid address"),
                    power: U256::from(1u128),
                    configuration_number: 11,
                },
                SetFederatedPower {
                    public_key: hex::decode("0485ff6016b937c0cec4f77cc452d250901aa8ff601faba7f4153a34c94ef9cdd8305e9432be8f94036d243dc1d8d5d0526d656ad179d252cf4dfdb49c78c47c65").expect("invalid public key"),
                    validator: Address::from_str("c5baa3e9dd11a4cee5ed18233bbb2a2a03fbbd80").expect("invalid address"),
                    power: U256::from(1u128),
                    configuration_number: 12,
                }
            ];

            let validator_changes = validator_powers
                .into_iter()
                .map(|p| {
                    let converted = public_key_to_address(&p.public_key);
                    assert_eq!(converted, p.validator);

                    StakingChangeRequest {
                        configuration_number: p.configuration_number,
                        change: StakingChange {
                            op,
                            payload: ethers::abi::encode(&[
                                Token::Bytes(p.public_key),
                                Token::Uint(p.power),
                            ]).into(),
                            validator: p.validator
                        },
                    }
                })
                .collect();

            let gateway_addr = EthAddress::from(
                ethers::types::Address::from_str("0x77aa40b105843728088c0132e43fc44348881da8")
                    .expect("invalid gateway addr"),
            );

            info!(
                "[Upgrade at height {}] Apply missing validator changes",
                state.block_height()
            );

            let topdown = ContractCaller::<_, _, TopDownFinalityFacetErrors>::new(
                gateway_addr,
                TopDownFinalityFacet::new,
            );

            topdown.call_with_return(state, |c| c.store_validator_changes(validator_changes))?;
            topdown.call_with_return(state, |c| c.apply_finality_changes())?;

            Ok(())
        }))
        .expect("cannot add gateway ownership upgrade");

    upgrade_scheduler
}

fn public_key_to_address(pub_key: &[u8]) -> Address {
    // Hash the serialized public key
    let hash = ethers::utils::keccak256(pub_key);

    // Take the last 20 bytes of the hash to get the address
    let address_bytes: [u8; 20] = hash[hash.len() - 20..].try_into().expect("slice with incorrect length");

    // Convert the bytes into an Ethereum address
    Address::from_slice(&address_bytes)
}