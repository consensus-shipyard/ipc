// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{HashMap, VecDeque};

use arbitrary::Unstructured;
use fendermint_crypto::{PublicKey, SecretKey};
use fendermint_testing::arb::{ArbSubnetAddress, ArbSubnetID, ArbTokenAmount};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_core::Timestamp;
use fendermint_vm_genesis::ipc::{GatewayParams, IpcParams};
use fendermint_vm_genesis::{
    Account, Actor, ActorMeta, Collateral, Genesis, SignerAddr, Validator, ValidatorKey,
};
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::bigint::Integer;
use fvm_shared::{econ::TokenAmount, version::NetworkVersion};
use ipc_sdk::subnet_id::SubnetID;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Debug, Clone)]
pub enum StakingOp {
    Deposit(TokenAmount),
    Withdraw(TokenAmount),
}

/// The staking message that goes towards the subnet to increase or decrease power.
#[derive(Debug, Clone)]
pub struct StakingUpdate {
    pub configuration_number: u64,
    pub addr: Address,
    pub op: StakingOp,
}

#[derive(Debug, Clone)]
pub struct StakingAccount {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub addr: Address,
    /// In this test the accounts should never gain more than their initial balance.
    pub initial_balance: TokenAmount,
    /// Balance after the effects of deposits/withdrawals.
    pub current_balance: TokenAmount,
}

/// Reference implementation for staking.
#[derive(Debug, Clone)]
pub struct StakingState {
    /// Accounts with secret key of accounts in case the contract wants to validate signatures.
    pub accounts: HashMap<Address, StakingAccount>,
    /// List of account addresses to help pick a random one.
    pub addrs: Vec<Address>,
    /// The parent genesis should include a bunch of accounts we can use to join a subnet.
    pub parent_genesis: Genesis,
    /// The child genesis describes the initial validator set to join the subnet
    pub child_genesis: Genesis,
    /// Currently active child validator set.
    pub child_validators: HashMap<Address, Collateral>,
    /// The configuration number to be incremented before each staking operation; 0 belongs to the genesis.
    pub configuration_number: u64,
    /// Unconfirmed staking operations.
    pub pending_updates: VecDeque<StakingUpdate>,
}

impl StakingState {
    pub fn new(
        accounts: Vec<StakingAccount>,
        parent_genesis: Genesis,
        child_genesis: Genesis,
    ) -> Self {
        let child_validators = child_genesis
            .validators
            .iter()
            .map(|v| {
                let addr = Address::new_secp256k1(&v.public_key.0.serialize()).unwrap();
                (addr, v.power.clone())
            })
            .collect();

        let accounts = accounts
            .into_iter()
            .map(|a| (a.addr, a))
            .collect::<HashMap<_, _>>();

        let addrs = accounts.keys().cloned().collect();

        Self {
            accounts,
            addrs,
            parent_genesis,
            child_genesis,
            child_validators,
            configuration_number: 0,
            pending_updates: VecDeque::new(),
        }
    }

    /// Apply the changes up to `the next_configuration_number`.
    pub fn checkpoint(mut self, next_configuration_number: u64) -> Self {
        loop {
            if self.pending_updates.is_empty() {
                break;
            }
            if self.pending_updates[0].configuration_number > next_configuration_number {
                break;
            }
            let update = self.pending_updates.pop_front().expect("checked non-empty");
            match update.op {
                StakingOp::Deposit(v) => {
                    let power = self.child_validators.entry(update.addr).or_default();
                    power.0 += v;
                }
                StakingOp::Withdraw(v) => {
                    match self.child_validators.entry(update.addr) {
                        std::collections::hash_map::Entry::Occupied(mut e) => {
                            let c = e.get().0.clone();
                            let v = v.min(c.clone());

                            if v == c {
                                e.remove();
                            } else {
                                e.insert(Collateral(c - v.clone()));
                            }

                            let a = self
                                .accounts
                                .get_mut(&update.addr)
                                .expect("validators have accounts");

                            a.current_balance += v;
                        }
                        std::collections::hash_map::Entry::Vacant(_) => {
                            // Tried to withdraw more than put in.
                        }
                    }
                }
            }
        }
        self.configuration_number = next_configuration_number;
        self
    }

    /// Enqueue a deposit.
    pub fn stake(mut self, addr: Address, value: TokenAmount) -> Self {
        self.configuration_number += 1;

        let a = self.accounts.get_mut(&addr).expect("accounts exist");

        // Sanity check that we are generating the expected kind of values.
        // Using `debug_assert!` on the reference state to differentiate from assertions on the SUT.
        debug_assert!(
            a.current_balance >= value,
            "stakes are generated within the balance"
        );
        a.current_balance -= value.clone();

        let update = StakingUpdate {
            configuration_number: self.configuration_number,
            addr,
            op: StakingOp::Deposit(value),
        };

        self.pending_updates.push_back(update);
        self
    }

    /// Enqueue a withdrawal.
    pub fn unstake(mut self, addr: Address, value: TokenAmount) -> Self {
        self.configuration_number += 1;
        let update = StakingUpdate {
            configuration_number: self.configuration_number,
            addr,
            op: StakingOp::Withdraw(value),
        };
        self.pending_updates.push_back(update);
        self
    }
}

impl arbitrary::Arbitrary<'_> for StakingState {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        // Limit the maximum number of *child subnet* validators to what the hypothetical consensus algorithm can scale to.
        let num_max_validators = 1 + usize::arbitrary(u)? % 10;
        // Create a number of accounts; it's okay if not everyone can become validators, and also okay if all of them can.
        let num_accounts = 1 + usize::arbitrary(u)? % 20;
        // Choose the size for the initial *child subnet* validator set.
        let num_validators = 1 + usize::arbitrary(u)? % num_accounts.min(num_max_validators);

        // Limit the amount of balance anyone can have so that the sum total of all of them
        // will still be lower than what we can send within Solidity as a value, which is U128.
        let max_balance = BigInt::from(u128::MAX) / num_accounts;

        // Create the desired number of accounts.
        let mut rng = StdRng::seed_from_u64(u64::arbitrary(u)?);
        let mut accounts = Vec::new();
        for _ in 0..num_accounts {
            let sk = SecretKey::random(&mut rng);
            let pk = sk.public_key();
            // All of them need to be ethereum accounts to interact with IPC.
            let addr = EthAddress::new_secp256k1(&pk.serialize()).unwrap();
            let addr = Address::from(addr);

            // Create with a non-zero balance so we can pick anyone to be a validator and deposit some collateral.
            let initial_balance = ArbTokenAmount::arbitrary(u)?
                .0
                .atto()
                .mod_floor(&max_balance);

            let initial_balance =
                TokenAmount::from_atto(initial_balance).max(TokenAmount::from_whole(1).clone());

            // The current balance is the same as the initial balance even if the account becomes
            // one of the validators on the child subnet, because for that they have to join the
            // subnet and that's when their funds are going to be locked up.
            let current_balance = initial_balance.clone();

            accounts.push(StakingAccount {
                public_key: pk,
                secret_key: sk,
                addr,
                initial_balance,
                current_balance,
            });
        }

        // Accounts on the parent subnet.
        let parent_actors = accounts
            .iter()
            .map(|s| Actor {
                meta: ActorMeta::Account(Account {
                    owner: SignerAddr(s.addr),
                }),
                balance: s.initial_balance.clone(),
            })
            .collect();

        // Select one validator to be the parent validator, it doesn't matter who.
        let parent_validators = vec![Validator {
            public_key: ValidatorKey(accounts[0].public_key),
            // All the power in the parent subnet belongs to this single validator.
            // We are only interested in the staking of the *child subnet*.
            power: Collateral(TokenAmount::from_whole(1)),
        }];

        // Select some of the accounts to be the initial *child subnet* validators.
        let child_validators = accounts
            .iter()
            .take(num_validators)
            .map(|a| {
                // Choose an initial stake committed to the child subnet.
                let initial_balance = a.initial_balance.atto();
                let initial_stake =
                    TokenAmount::from_atto(BigInt::arbitrary(u)?.mod_floor(initial_balance));
                // Make sure it's not zero.
                let initial_stake = initial_stake.max(TokenAmount::from_whole(1));

                Ok(Validator {
                    public_key: ValidatorKey(a.public_key),
                    power: Collateral(initial_stake),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Choose an attainable activation limit.
        let initial_stake: BigInt = child_validators.iter().map(|v| v.power.0.atto()).sum();
        let min_collateral =
            TokenAmount::from_atto(BigInt::arbitrary(u)?.mod_floor(&initial_stake))
                .max(TokenAmount::from_atto(1));

        // IPC of the parent subnet itself - most are not going to be used.
        let parent_ipc = IpcParams {
            gateway: GatewayParams {
                subnet_id: ArbSubnetID::arbitrary(u)?.0,
                bottom_up_check_period: 1 + u.choose_index(100)? as u64,
                msg_fee: ArbTokenAmount::arbitrary(u)?.0,
                majority_percentage: 51 + u8::arbitrary(u)? % 50,
                min_collateral,
                active_validators_limit: 1 + u.choose_index(100)? as u16,
            },
        };

        let child_subnet_id = SubnetID::new_from_parent(
            &parent_ipc.gateway.subnet_id,
            ArbSubnetAddress::arbitrary(u)?.0,
        );

        let parent_genesis = Genesis {
            chain_name: String::arbitrary(u)?,
            timestamp: Timestamp(u64::arbitrary(u)?),
            network_version: NetworkVersion::V20,
            base_fee: ArbTokenAmount::arbitrary(u)?.0,
            power_scale: *u.choose(&[0, 3]).expect("non empty"),
            validators: parent_validators,
            accounts: parent_actors,
            ipc: Some(parent_ipc),
        };

        let child_ipc = IpcParams {
            gateway: GatewayParams {
                subnet_id: child_subnet_id,
                bottom_up_check_period: 1 + u.choose_index(100)? as u64,
                msg_fee: ArbTokenAmount::arbitrary(u)?.0,
                majority_percentage: 51 + u8::arbitrary(u)? % 50,
                min_collateral: ArbTokenAmount::arbitrary(u)?
                    .0
                    .max(TokenAmount::from_atto(1)),
                active_validators_limit: num_max_validators as u16,
            },
        };

        let child_genesis = Genesis {
            chain_name: String::arbitrary(u)?,
            timestamp: Timestamp(u64::arbitrary(u)?),
            network_version: NetworkVersion::V20,
            base_fee: ArbTokenAmount::arbitrary(u)?.0,
            power_scale: *u.choose(&[0, 3]).expect("non empty"),
            validators: child_validators,
            accounts: Vec::new(),
            ipc: Some(child_ipc),
        };

        Ok(StakingState::new(accounts, parent_genesis, child_genesis))
    }
}
