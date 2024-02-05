// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_testing::arb::ArbAddress;
use lazy_static::lazy_static;
use std::{
    cmp::min,
    ops::{Mul, SubAssign},
};

use fendermint_vm_genesis::Collateral;
use fvm_shared::{
    bigint::{BigInt, Integer, Zero},
    econ::TokenAmount,
};
use quickcheck::{Arbitrary, Gen};

use crate::manifest::{
    Account, AccountId, Balance, BalanceMap, CollateralMap, IpcDeployment, Manifest, Node, NodeId,
    NodeMap, NodeMode, Relayer, RelayerId, ResourceId, Rootnet, Subnet, SubnetId, SubnetMap,
};

const RESOURCE_ID_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

lazy_static! {
    /// Assume we have as much tFIL on the root as the faucet would give.
    static ref DEFAULT_BALANCE: Balance = Balance(TokenAmount::from_whole(100));
}

/// Select some items from a slice.
fn choose_at_least<T: Clone>(g: &mut Gen, min_size: usize, xs: &[T]) -> Vec<T> {
    let min_size = min(min_size, xs.len());

    if min_size == xs.len() {
        return Vec::from(xs);
    }

    // Say we have 10 items and we have 3 slots to fill.
    //
    // Imagine a simple algorithm that selects 1 item 3 times without replacement.
    // Initially each item has 1/10 chance to be selected, then 1/9, then 1/8,
    // but we would need to track which item has already been chosen.
    //
    // We want to do a single pass over the data.
    //
    // If we consider the 1st item, the chance that it doesn't get selected for any of the slots is:
    // P_excl(1st) = 9/10 * 8/9 * 7/8 = 7/10
    // P_incl(1st) = 1 - P_excl(1st) = 3/10
    //
    // So we roll the dice and with 30% probability we include the 1st item in the list.
    //
    // Then we have two cases to consider:
    // 1. We included the 1st item, so now we have 2 slots and remaining 9 items to choose from.
    //    P_incl(2nd | incl(1st)) = 1 - 8/9 * 7/8 = 1 - 7/9 = 2/9
    // 2. We excluded the 1st item, so we still have 3 slots to fill and remaining 9 items to choose from.
    //    P_incl(2nd | excl(1st)) = 1 - 8/9 * 7/8 * 6/7 = 1 - 6/9 = 3/9
    //
    // Thus, the probability of including each item is `remaining slots / remaining items`

    let mut remaining_slots = min_size + usize::arbitrary(g) % (xs.len() - min_size);
    let mut remaining_items = xs.len();

    let mut chosen = Vec::new();
    for x in xs {
        if remaining_slots == 0 {
            break;
        }
        if usize::arbitrary(g) % remaining_items < remaining_slots {
            chosen.push(x.clone());
            remaining_slots -= 1;
        }
        remaining_items -= 1;
    }
    chosen
}

fn choose_one<T: Clone>(g: &mut Gen, xs: &[T]) -> T {
    g.choose(xs).expect("empty sequence to choose from").clone()
}

impl Arbitrary for ResourceId {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = 3 + usize::arbitrary(g) % 6;

        let id = (0..len)
            .map(|_| {
                let idx = usize::arbitrary(g) % RESOURCE_ID_CHARSET.len();
                char::from(RESOURCE_ID_CHARSET[idx])
            })
            .collect();

        Self(id)
    }
}

impl Arbitrary for Balance {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(Collateral::arbitrary(g).0)
    }
}

impl Arbitrary for Manifest {
    fn arbitrary(g: &mut Gen) -> Self {
        gen_manifest(g, 4, 3, DEFAULT_BALANCE.clone())
    }
}

fn gen_manifest(
    g: &mut Gen,
    max_children: usize,
    max_level: usize,
    default_balance: Balance,
) -> Manifest {
    let account_ids = (0..3 + usize::arbitrary(g) % 3)
        .map(|_| AccountId::arbitrary(g))
        .collect::<Vec<_>>();

    let accounts = account_ids
        .iter()
        .map(|id| (id.clone(), Account {}))
        .collect();

    let mut balances: BalanceMap = account_ids
        .iter()
        .map(|id| (id.clone(), default_balance.clone()))
        .collect();

    let rootnet = if bool::arbitrary(g) {
        Rootnet::External {
            deployment: if bool::arbitrary(g) {
                IpcDeployment::Existing {
                    gateway: ArbAddress::arbitrary(g).0,
                    registry: ArbAddress::arbitrary(g).0,
                }
            } else {
                IpcDeployment::New {
                    deployer: choose_one(g, &account_ids),
                }
            },
        }
    } else {
        let initial_balances = balances.clone();
        // Reuse the subnet generation logic for picking validators and nodes.
        let subnet = gen_subnets(g, 0, 2, 2, &account_ids, &[], &mut balances);
        let subnet = subnet
            .into_iter()
            .next()
            .expect("should have exactly 1 subnet")
            .1;

        Rootnet::New {
            validators: subnet.validators,
            balances: initial_balances,
            nodes: subnet.nodes,
        }
    };

    // Pick some node IDs targeted by relayers on the rootnet.
    let parent_node_ids = match rootnet {
        Rootnet::External { .. } => Vec::new(), // No specific target, it will be a JSON-RPC endpoint.
        Rootnet::New { ref nodes, .. } => nodes.keys().cloned().collect(),
    };

    // The rootnet is L1, immediate subnets are L2.
    let subnets = gen_subnets(
        g,
        max_children,
        max_level,
        2,
        &account_ids,
        &parent_node_ids,
        &mut balances,
    );

    Manifest {
        accounts,
        rootnet,
        subnets,
    }
}

/// Recursively generate some subnets.
fn gen_subnets(
    g: &mut Gen,
    max_children: usize,
    max_level: usize,
    level: usize,
    account_ids: &[AccountId],
    parent_node_ids: &[NodeId],
    balances: &mut BalanceMap,
) -> SubnetMap {
    let mut subnets = SubnetMap::default();

    if level > max_level {
        return subnets;
    }

    // Let the root have at least 1 child, otherwise it's not interesting.
    let min_children = if level == 2 { 1 } else { 0 };
    let num_children = if max_children <= min_children {
        min_children
    } else {
        usize::arbitrary(g) % (max_children - min_children)
    };

    for _ in 0..num_children {
        // We can pick any creator; we'll make sure this one also gets some
        // funds to pay for the creation of the subnet.
        let c = choose_one(g, account_ids);

        // Every subnet needs validators, so at least 1 needs to be chosen.
        let vs: CollateralMap = choose_at_least(g, 1, account_ids)
            .into_iter()
            .map(|a| {
                let c = gen_collateral(g, &a, balances);
                (a, c)
            })
            .collect();

        // It's not necessary to have accounts in a subnet; only declaring
        // some that we want to end up with some balance, but others might
        // funded to support similar ones further down the hierarchy.
        let bs: BalanceMap = choose_at_least(g, 0, account_ids)
            .into_iter()
            .map(|a| {
                let b: Balance = gen_balance(g, &a, balances);
                (a, b)
            })
            .collect();

        // Run at least a quroum of validators.
        let tw: TokenAmount = vs.values().map(|c| c.0.clone()).sum();
        let qw = tw.mul(2).div_floor(3);
        let mut ss = Vec::new();
        let mut ns = NodeMap::default();
        let mut sw = TokenAmount::zero();

        for (v, w) in vs.iter() {
            let mode = if sw <= qw || bool::arbitrary(g) {
                NodeMode::Validator(v.clone())
            } else {
                NodeMode::Full
            };
            let seed_nodes = if ss.is_empty() {
                vec![]
            } else {
                choose_at_least(g, 1, &ss)
            };
            let node = Node {
                mode,
                ethapi: bool::arbitrary(g),
                seed_nodes,
                parent_node: g.choose(parent_node_ids).cloned(),
            };
            let id = NodeId::arbitrary(g);
            ss.push(id.clone());
            ns.insert(id, node);
            sw += w.0.clone();
        }

        let rs = (0..1 + usize::arbitrary(g) % 3)
            .map(|_| {
                let r = Relayer {
                    submitter: choose_one(g, account_ids),
                    follow_node: choose_one(g, &ss),
                    submit_node: g.choose(parent_node_ids).cloned(),
                };
                let id = RelayerId::arbitrary(g);
                (id, r)
            })
            .collect();

        let ss = gen_subnets(
            g,
            max_children,
            max_level,
            level + 1,
            account_ids,
            &ss,
            balances,
        );

        let s = Subnet {
            creator: c,
            validators: vs,
            balances: bs,
            nodes: ns,
            relayers: rs,
            subnets: ss,
        };

        let sid = SubnetId::arbitrary(g);

        subnets.insert(sid, s);
    }

    subnets
}

/// Choose some balance, up to 10% of the remaining balance of the account, minimum 1 atto.
///
/// Modify the reamaining balance so we don't run out.
fn gen_balance(g: &mut Gen, account_id: &AccountId, balances: &mut BalanceMap) -> Balance {
    let r = balances
        .get_mut(account_id)
        .expect("account doesn't have balance");
    let m = r.0.atto().div_ceil(&BigInt::from(10));
    let b = BigInt::arbitrary(g).mod_floor(&m).max(BigInt::from(1));
    let b = TokenAmount::from_atto(b);
    r.0.sub_assign(b.clone());
    Balance(b)
}

fn gen_collateral(g: &mut Gen, account_id: &AccountId, balances: &mut BalanceMap) -> Collateral {
    let b = gen_balance(g, account_id, balances);
    Collateral(b.0)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    use super::choose_at_least;

    #[derive(Clone, Debug)]
    struct TestSample {
        items: Vec<u8>,
        min_size: usize,
        sample: Vec<u8>,
    }

    impl Arbitrary for TestSample {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut items = HashSet::<u8>::arbitrary(g);
            items.insert(u8::arbitrary(g));
            let items = items.into_iter().collect::<Vec<_>>();
            let min_size = 1 + usize::arbitrary(g) % items.len();
            let sample = choose_at_least(g, min_size, &items);
            TestSample {
                items,
                min_size,
                sample,
            }
        }
    }

    #[quickcheck]
    fn test_sample_at_least(data: TestSample) {
        let sample_set = BTreeSet::from_iter(&data.sample);
        let item_set = BTreeSet::from_iter(&data.items);

        assert!(
            data.sample.len() >= data.min_size,
            "sampled at least the required amount"
        );
        assert!(
            data.sample.len() <= data.items.len(),
            "didn't sample more than available"
        );
        assert!(
            sample_set.is_subset(&item_set),
            "sample items are taken from the existing ones"
        );
        assert_eq!(data.sample.len(), sample_set.len(), "sample is unique");
    }
}
