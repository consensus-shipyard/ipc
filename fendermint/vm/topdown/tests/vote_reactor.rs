// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! ```text
//! cargo test --release -p fendermint_vm_topdown --test smt_vote_reactor
//! ```

use async_trait::async_trait;
use libp2p::futures::AsyncReadExt;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;
use fendermint_crypto::SecretKey;
use fendermint_vm_genesis::ValidatorKey;
use fendermint_vm_topdown::vote::error::Error;
use fendermint_vm_topdown::vote::gossip::GossipClient;
use fendermint_vm_topdown::vote::payload::{PowerTable, PowerUpdates, Vote};
use fendermint_vm_topdown::vote::{Config, start_vote_reactor, Weight};
use fendermint_vm_topdown::vote::store::InMemoryVoteStore;

struct Validator {
    sk: SecretKey,
    weight: Weight,
}

impl Validator {
    fn validator_key(&self) -> ValidatorKey {
        ValidatorKey::new(self.sk.public_key())
    }
}

struct ChannelGossipClient {
    tx: broadcast::Sender<Vote>,
    rxs: Vec<broadcast::Receiver<Vote>>,
}

#[async_trait]
impl GossipClient for ChannelGossipClient {
    fn try_poll_vote(&mut self) -> Result<Option<Vote>, Error> {
        for rx in self.rxs.iter_mut() {
            match rx.try_recv() {
                Ok(v) => return Ok(Some(v)),
                Err(broadcast::error::TryRecvError::Empty) => continue,
                _ => panic!("should not happen")
            }
        }

        Ok(None)
    }

    async fn publish_vote(&self, vote: Vote) -> Result<(), Error> {
        self.tx.send(vote).unwrap();
        Ok(())
    }
}

fn default_config() -> Config {
    Config{
        req_channel_buffer_size: 1024,
        req_batch_processing_size: 10,
        gossip_req_processing_size: 10,
        voting_sleep_interval_sec: 1,
    }
}

fn gen_validators(weights: Vec<Weight>) -> (Vec<Validator>, Vec<ChannelGossipClient>) {
    let mut rng = rand::thread_rng();

    let mut gossips: Vec<ChannelGossipClient> = vec![];
    for _ in 0..weights.len() {
        let (tx, rx) = broadcast::channel(100);

        let mut g = ChannelGossipClient{ tx, rxs: vec![] };

        for existing in gossips.iter() {
            g.rxs.push(existing.tx.subscribe())
        }

        for existing in gossips.iter_mut() {
            existing.rxs.push(g.tx.subscribe());
        }

        gossips.push(g);
    }

    let validators = weights
        .into_iter()
        .map(|w| Validator { sk: SecretKey::random(&mut rng), weight: w })
        .collect::<Vec<_>>();

    (validators, gossips)
}

// fn gen_power_table(validators: &[Validator]) -> PowerTable {
//     PowerTable::from_iter(validators.iter().map(|v| (v.validator_key(), v.weight)))
// }

fn gen_power_updates(validators: &[Validator]) -> PowerUpdates {
    validators.iter().map(|v| (v.validator_key(), v.weight)).collect()
}

#[tokio::test]
async fn all_validators_active_mode() {
    let config = default_config();

    // 21 validators equal 100 weight
    let (validators, gossips) = gen_validators(vec![100; 21]);
    let power_updates = gen_power_updates(&validators);
    let initial_finalized_height = 10;

    let (internal_event_tx, _) = broadcast::channel(1024);

    let node_clients = gossips
        .into_iter()
        .map(|gossip| {
            start_vote_reactor(
                config.clone(),
                power_updates.clone(),
                initial_finalized_height,
                gossip,
                InMemoryVoteStore::default(),
                internal_event_tx.subscribe(),
            )
                .unwrap()
        })
        .collect::<Vec<_>>();
}