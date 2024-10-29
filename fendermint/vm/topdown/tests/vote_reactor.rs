// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! ```text
//! cargo test --release -p fendermint_vm_topdown --test vote_reactor
//! ```

use async_trait::async_trait;
use fendermint_crypto::SecretKey;
use fendermint_vm_genesis::ValidatorKey;
use fendermint_vm_topdown::observation::Observation;
use fendermint_vm_topdown::syncer::TopDownSyncEvent;
use fendermint_vm_topdown::vote::error::Error;
use fendermint_vm_topdown::vote::gossip::GossipClient;
use fendermint_vm_topdown::vote::payload::{PowerUpdates, Vote};
use fendermint_vm_topdown::vote::store::InMemoryVoteStore;
use fendermint_vm_topdown::vote::{
    start_vote_reactor, StartVoteReactorParams, VoteConfig, VoteReactorClient, Weight,
};
use fendermint_vm_topdown::BlockHeight;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;

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
                Err(TryRecvError::Empty) => continue,
                _ => panic!("should not happen"),
            }
        }

        Ok(None)
    }

    async fn publish_vote(&self, vote: Vote) -> Result<(), Error> {
        let _ = self.tx.send(vote);
        Ok(())
    }
}

fn default_config() -> VoteConfig {
    VoteConfig {
        req_channel_buffer_size: 1024,
        req_batch_processing_size: 10,
        gossip_req_processing_size: 10,
        voting_sleep_interval_millis: 1,
    }
}

fn gen_validators(weights: Vec<Weight>) -> (Vec<Validator>, Vec<ChannelGossipClient>) {
    let mut rng = rand::thread_rng();

    let mut gossips: Vec<ChannelGossipClient> = vec![];
    for _ in 0..weights.len() {
        let (tx, _) = broadcast::channel(100);

        let mut g = ChannelGossipClient { tx, rxs: vec![] };

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
        .map(|w| Validator {
            sk: SecretKey::random(&mut rng),
            weight: w,
        })
        .collect::<Vec<_>>();

    (validators, gossips)
}

// fn gen_power_table(validators: &[Validator]) -> PowerTable {
//     PowerTable::from_iter(validators.iter().map(|v| (v.validator_key(), v.weight)))
// }

fn gen_power_updates(validators: &[Validator]) -> PowerUpdates {
    validators
        .iter()
        .map(|v| (v.validator_key(), v.weight))
        .collect()
}

async fn ensure_votes_received(
    clients: &[VoteReactorClient],
    height_votes: Vec<(BlockHeight, usize)>,
) {
    for client in clients {
        for (height, votes) in &height_votes {
            while client.query_votes(*height).await.unwrap().unwrap().len() != *votes {}
        }
    }
}

#[tokio::test]
async fn simple_lifecycle() {
    let config = default_config();

    // 21 validators equal 100 weight
    let (validators, mut gossips) = gen_validators(vec![100; 1]);
    let power_updates = gen_power_updates(&validators);
    let initial_finalized_height = 10;

    let (internal_event_tx, _) = broadcast::channel(validators.len() + 1);

    let client = start_vote_reactor(StartVoteReactorParams {
        config: config.clone(),
        validator_key: validators[0].sk.clone(),
        power_table: power_updates.clone(),
        last_finalized_height: initial_finalized_height,
        latest_child_block: 100,
        gossip: gossips.pop().unwrap(),
        vote_store: InMemoryVoteStore::default(),
        internal_event_listener: internal_event_tx.subscribe(),
    })
    .unwrap();

    assert_eq!(client.find_quorum().await.unwrap(), None);

    // now topdown sync published a new observation on parent height 100
    let parent_height = 100;
    let obs = Observation::new(parent_height, vec![1, 2, 3], vec![2, 3, 4]);
    internal_event_tx
        .send(TopDownSyncEvent::NewProposal(Box::new(obs)))
        .unwrap();

    // wait for vote to be casted
    while client.find_quorum().await.unwrap().is_none() {}

    let r = client.find_quorum().await.unwrap().unwrap();
    assert_eq!(r.payload().parent_height(), parent_height);

    let r = client.query_votes(parent_height).await.unwrap().unwrap();
    assert_eq!(r.len(), 1);

    // now push another observation
    let parent_height2 = 101;
    let obs2 = Observation::new(parent_height2, vec![1, 2, 3], vec![2, 3, 4]);
    internal_event_tx
        .send(TopDownSyncEvent::NewProposal(Box::new(obs2)))
        .unwrap();

    client
        .set_quorum_finalized(parent_height)
        .await
        .unwrap()
        .unwrap();

    let state = client.query_vote_tally_state().await.unwrap();
    assert_eq!(state.last_finalized_height, parent_height);

    let votes = client.query_votes(parent_height2).await.unwrap().unwrap();
    assert_eq!(votes.len(), 1);
    let r = client.find_quorum().await.unwrap().unwrap();
    assert_eq!(r.payload().parent_height(), parent_height2);

    client
        .set_quorum_finalized(parent_height2)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(client.find_quorum().await.unwrap(), None);
    assert!(
        client.dump_votes().await.unwrap().unwrap().is_empty(),
        "should have no votes left"
    );
}

/// This tests votes coming in the wrong block height order and it still works
#[tokio::test]
async fn waiting_for_quorum() {
    let config = default_config();

    let (validators, mut gossips) = gen_validators(vec![100; 5]);
    let power_updates = gen_power_updates(&validators);
    let initial_finalized_height = 10;

    let mut clients = vec![];
    let mut internal_txs = vec![];
    for i in 0..validators.len() {
        let (internal_event_tx, _) = broadcast::channel(validators.len() + 1);

        let client = start_vote_reactor(StartVoteReactorParams {
            config: config.clone(),
            validator_key: validators[i].sk.clone(),
            power_table: power_updates.clone(),
            last_finalized_height: initial_finalized_height,
            latest_child_block: 100,
            gossip: gossips.pop().unwrap(),
            vote_store: InMemoryVoteStore::default(),
            internal_event_listener: internal_event_tx.subscribe(),
        })
        .unwrap();

        clients.push(client);
        internal_txs.push(internal_event_tx);
    }

    // now topdown sync published a new observation on parent height 100
    let parent_height1 = 100;
    let obs1 = Observation::new(parent_height1, vec![1, 2, 3], vec![2, 3, 4]);
    let parent_height2 = 110;
    let obs2 = Observation::new(parent_height2, vec![1, 2, 3], vec![2, 3, 4]);
    let parent_height3 = 120;
    let obs3 = Observation::new(parent_height3, vec![1, 2, 3], vec![2, 3, 4]);

    internal_txs[0]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs1.clone())))
        .unwrap();
    internal_txs[1]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs1.clone())))
        .unwrap();

    internal_txs[2]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs2.clone())))
        .unwrap();
    internal_txs[3]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs2.clone())))
        .unwrap();

    internal_txs[4]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs3.clone())))
        .unwrap();

    ensure_votes_received(
        &clients,
        vec![
            (parent_height1, 2),
            (parent_height2, 2),
            (parent_height3, 1),
        ],
    )
    .await;

    // at this moment, no quorum should have ever formed
    for client in &clients {
        assert!(
            client.find_quorum().await.unwrap().is_none(),
            "should have no quorum"
        );
    }

    // new observations made
    internal_txs[3]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs3.clone())))
        .unwrap();
    internal_txs[0]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs3.clone())))
        .unwrap();
    internal_txs[1]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs3.clone())))
        .unwrap();

    ensure_votes_received(&clients, vec![(parent_height3, 4)]).await;

    for client in &clients {
        let r = client.find_quorum().await.unwrap().unwrap();
        assert_eq!(
            r.payload().parent_height(),
            parent_height3,
            "should have quorum"
        );
    }

    // make observation on previous heights
    internal_txs[3]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs1.clone())))
        .unwrap();
    internal_txs[2]
        .send(TopDownSyncEvent::NewProposal(Box::new(obs1.clone())))
        .unwrap();

    // ensure every client receives the votes
    ensure_votes_received(&clients, vec![(parent_height1, 4)]).await;

    // but larger parent height wins
    for client in &clients {
        let r = client.find_quorum().await.unwrap().unwrap();
        assert_eq!(
            r.payload().parent_height(),
            parent_height3,
            "should have formed quorum on larger height"
        );
    }

    // finalize parent height 3
    for client in &clients {
        client
            .set_quorum_finalized(parent_height3)
            .await
            .unwrap()
            .unwrap();
        assert!(
            client.dump_votes().await.unwrap().unwrap().is_empty(),
            "should have empty votes"
        );
    }
}

#[tokio::test]
async fn all_validator_in_sync() {
    let config = default_config();

    // 21 validators equal 100 weight
    let (validators, mut gossips) = gen_validators(vec![100; 10]);
    let power_updates = gen_power_updates(&validators);
    let initial_finalized_height = 10;

    let (internal_event_tx, _) = broadcast::channel(validators.len() + 1);

    let mut node_clients = vec![];
    for validator in &validators {
        let r = start_vote_reactor(StartVoteReactorParams {
            config: config.clone(),
            validator_key: validator.sk.clone(),
            power_table: power_updates.clone(),
            last_finalized_height: initial_finalized_height,
            latest_child_block: 100,
            gossip: gossips.pop().unwrap(),
            vote_store: InMemoryVoteStore::default(),
            internal_event_listener: internal_event_tx.subscribe(),
        })
        .unwrap();

        node_clients.push(r);
    }

    let parent_height = 100;
    let obs = Observation::new(parent_height, vec![1, 2, 3], vec![2, 3, 4]);
    internal_event_tx
        .send(TopDownSyncEvent::NewProposal(Box::new(obs)))
        .unwrap();

    for n in node_clients {
        while n.find_quorum().await.unwrap().is_none() {}

        let r = n.find_quorum().await.unwrap().unwrap();
        assert_eq!(r.payload().parent_height(), parent_height)
    }
}
