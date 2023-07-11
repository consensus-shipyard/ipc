// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::time::{Duration, Instant};

use anyhow::Context;
use ethers_core::types as et;
use fendermint_vm_actor_interface::eam::EthAddress;
use fvm_shared::address::Address;
use tendermint_rpc::{
    event::{Event, EventData},
    query::{EventType, Query},
};

use crate::conv::from_tm;

/// Check whether to keep a log according to the topic filter.
///
/// A note on specifying topic filters: Topics are order-dependent.
/// A transaction with a log with topics [A, B] will be matched by the following topic filters:
/// * [] "anything"
/// * [A] "A in first position (and anything after)"
/// * [null, B] "anything in first position AND B in second position (and anything after)"
/// * [A, B] "A in first position AND B in second position (and anything after)"
/// * [[A, B], [A, B]] "(A OR B) in first position AND (A OR B) in second position (and anything after)"
pub fn matches_topics(filter: &et::Filter, log: &et::Log) -> bool {
    for i in 0..4 {
        if let Some(topics) = &filter.topics[i] {
            let topic = log.topics.get(i);
            let matches = match topics {
                et::ValueOrArray::Value(Some(t)) => topic == Some(t),
                et::ValueOrArray::Array(ts) => ts.iter().flatten().any(|t| topic == Some(t)),
                _ => true,
            };
            if !matches {
                return false;
            }
        }
    }
    true
}

pub type FilterId = et::U256;

pub enum FilterKind {
    NewBlocks,
    PendingTransactions,
    Logs(Box<et::Filter>),
}

impl FilterKind {
    /// Convert an Ethereum filter to potentially multiple Tendermint queries.
    ///
    /// One limitation with Tendermint is that it only handles AND condition
    /// in filtering, so if the filter contains arrays, we have to make a
    /// cartesian product of all conditions in it and subscribe individually.
    ///
    /// https://docs.tendermint.com/v0.34/rpc/#/Websocket/subscribe
    pub fn to_queries(&self) -> anyhow::Result<Vec<Query>> {
        match self {
            FilterKind::NewBlocks => Ok(vec![Query::from(EventType::NewBlock)]),
            // Testing indicates that `EventType::Tx` might only be raised
            // if there are events emitted by the transaction itself.
            FilterKind::PendingTransactions => Ok(vec![Query::from(EventType::NewBlock)]),
            FilterKind::Logs(filter) => {
                let mut query = Query::from(EventType::Tx);

                if let Some(block_hash) = filter.get_block_hash() {
                    query = query.and_eq("tx.hash", hex::encode(block_hash.0));
                }
                if let Some(from_block) = filter.get_from_block() {
                    query = query.and_gte("tx.height", from_block.as_u64());
                }
                if let Some(to_block) = filter.get_to_block() {
                    query = query.and_lte("tx.height", to_block.as_u64());
                }

                let mut queries = vec![query];

                let addrs = match &filter.address {
                    None => vec![],
                    Some(et::ValueOrArray::Value(addr)) => vec![*addr],
                    Some(et::ValueOrArray::Array(addrs)) => addrs.clone(),
                };

                let addrs = addrs
                    .into_iter()
                    .map(|addr| {
                        Address::from(EthAddress(addr.0))
                            .id()
                            .context("only f0 type addresses are supported")
                    })
                    .collect::<Result<Vec<u64>, _>>()?;

                if !addrs.is_empty() {
                    queries = addrs
                        .iter()
                        .flat_map(|addr| {
                            queries
                                .iter()
                                .map(|q| q.clone().and_eq("message.emitter", addr.to_string()))
                        })
                        .collect();
                };

                for i in 0..4 {
                    if let Some(Some(topics)) = filter.topics.get(i) {
                        let topics = match topics {
                            et::ValueOrArray::Value(Some(t)) => vec![t],
                            et::ValueOrArray::Array(ts) => ts.iter().flatten().collect(),
                            _ => vec![],
                        };
                        if !topics.is_empty() {
                            let key = format!("message.t{}", i + 1);
                            queries = topics
                                .into_iter()
                                .flat_map(|t| {
                                    queries
                                        .iter()
                                        .map(|q| q.clone().and_eq(&key, hex::encode(t.0)))
                                })
                                .collect();
                        }
                    }
                }

                Ok(queries)
            }
        }
    }
}

/// Accumulator for filter data.
///
/// The type expected can be seen in [ethers::providers::Provider::watch_blocks].
pub enum FilterRecords {
    NewBlocks(Vec<et::H256>),
    PendingTransactions(Vec<et::H256>),
    Logs(Vec<et::Log>),
}

impl FilterRecords {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::NewBlocks(xs) => xs.is_empty(),
            Self::PendingTransactions(xs) => xs.is_empty(),
            Self::Logs(xs) => xs.is_empty(),
        }
    }
}

impl From<&FilterKind> for FilterRecords {
    fn from(value: &FilterKind) -> Self {
        match value {
            FilterKind::NewBlocks => Self::NewBlocks(vec![]),
            FilterKind::PendingTransactions => Self::PendingTransactions(vec![]),
            FilterKind::Logs(_) => Self::Logs(vec![]),
        }
    }
}

impl From<&FilterRecords> for FilterRecords {
    fn from(value: &FilterRecords) -> Self {
        match value {
            Self::NewBlocks(_) => Self::NewBlocks(vec![]),
            Self::PendingTransactions(_) => Self::PendingTransactions(vec![]),
            Self::Logs(_) => Self::Logs(vec![]),
        }
    }
}

/// Accumulate changes between polls.
pub struct FilterState {
    _id: FilterId,
    timeout: Duration,
    last_poll: Instant,
    finished: Option<Option<anyhow::Error>>,
    records: FilterRecords,
}

impl FilterState {
    pub fn new(id: FilterId, timeout: Duration, kind: &FilterKind) -> Self {
        Self {
            _id: id,
            timeout,
            last_poll: Instant::now(),
            finished: None,
            records: FilterRecords::from(kind),
        }
    }

    /// Accumulate the events.
    pub fn update(&mut self, event: Event) -> anyhow::Result<()> {
        match (&mut self.records, &event.data) {
            (
                FilterRecords::NewBlocks(ref mut hashes),
                EventData::NewBlock {
                    block: Some(block), ..
                },
            ) => {
                let h = block.header().hash();
                let h = et::H256::from_slice(h.as_bytes());
                hashes.push(h);
            }
            (
                FilterRecords::PendingTransactions(ref mut hashes),
                EventData::NewBlock {
                    block: Some(block), ..
                },
            ) => {
                for tx in &block.data {
                    let h = from_tm::message_hash(tx)?;
                    let h = et::H256::from_slice(h.as_bytes());
                    hashes.push(h);
                }
            }
            (FilterRecords::Logs(ref mut logs), EventData::Tx { tx_result }) => {
                // An example of an `Event`:
                // Event {
                //     query: "tm.event = 'Tx'",
                //     data: Tx {
                //         tx_result: TxInfo {
                //             height: 1088,
                //             index: None,
                //             tx: [161, 102, ..., 0],
                //             result: TxResult {
                //                 log: None,
                //                 gas_wanted: Some("5156433"),
                //                 gas_used: Some("5151233"),
                //                 events: [
                //                     Event {
                //                         kind: "message",
                //                         attributes: [
                //                             EventAttribute { key: "emitter", value: "108", index: true },
                //                             EventAttribute { key: "t1", value: "dd...b3ef", index: true },
                //                             EventAttribute { key: "t2", value: "00...362f", index: true },
                //                             EventAttribute { key: "t3", value: "00...44eb", index: true },
                //                             EventAttribute { key: "d",  value: "00...0064", index: true }
                //                         ]
                //                     }
                //                 ]
                //             }
                //         }
                //     },
                //     events: Some(
                //     {
                //         "message.d": ["00...0064"],
                //         "message.emitter": ["108"],
                //         "message.t1": ["dd...b3ef"],
                //         "message.t2": ["00...362f"],
                //         "message.t3": ["00...44eb"],
                //         "tm.event": ["Tx"],
                //         "tx.hash": ["FA7339B4D9F6AF80AEDB03FC4BFBC1FDD9A62F97632EF8B79C98AAD7044C5BDB"],
                //         "tx.height": ["1088"]
                //     })
                // }

                // TODO: There is no easy way here to tell the block hash. Maybe it has been given in a preceding event,
                // but other than that our only option is to query the Tendermint API. If we do that we should have caching,
                // otherwise all the transactions in a block hammering the node will act like a DoS attack.
                let block_hash = et::H256::default();
                let block_number = et::U64::from(tx_result.height);

                let transaction_hash = from_tm::message_hash(&tx_result.tx)?;
                let transaction_hash = et::H256::from_slice(transaction_hash.as_bytes());

                // TODO: The transaction index comes as None.
                let transaction_index = et::U64::from(tx_result.index.unwrap_or_default());

                // TODO: We have no way to tell where the logs start within the block.
                let log_index_start = Default::default();

                let tx_logs = from_tm::to_logs(
                    &tx_result.result.events,
                    block_hash,
                    block_number,
                    transaction_hash,
                    transaction_index,
                    log_index_start,
                )?;

                logs.extend(tx_logs)
            }
            _ => {}
        }
        Ok(())
    }

    /// Take all the accumulated changes.
    ///
    /// If there are no changes but there was an error, return that.
    /// If the producers have stopped, return `None`.
    pub fn try_take(&mut self) -> anyhow::Result<Option<FilterRecords>> {
        self.last_poll = Instant::now();

        let mut records = FilterRecords::from(&self.records);
        std::mem::swap(&mut self.records, &mut records);

        if records.is_empty() {
            if let Some(ref mut finished) = self.finished {
                // Return error on first poll, because it can't be cloned.
                return match finished.take() {
                    Some(e) => Err(e),
                    None => Ok(None),
                };
            }
        }

        Ok(Some(records))
    }

    /// Signal that the producers are finished, or that the reader is no longer intersted.
    ///
    /// Propagate the error to the reader next time it comes to check on the filter.
    pub fn finish(&mut self, error: Option<anyhow::Error>) {
        // Keep any already existing error.
        let error = self.finished.take().flatten().or(error);

        self.finished = Some(error);
    }

    /// Indicate whether the reader has been too slow at polling the filter
    /// and that it should be removed.
    pub fn is_timed_out(&self) -> bool {
        Instant::now().duration_since(self.last_poll) > self.timeout
    }

    /// Indicate that that the filter takes no more data.
    pub fn is_finished(&self) -> bool {
        self.finished.is_some()
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::types as et;

    use super::FilterKind;

    #[test]
    fn filter_to_query() {
        fn hash(s: &str) -> et::H256 {
            et::H256::from(ethers_core::utils::keccak256(s))
        }

        fn hash_hex(s: &str) -> String {
            hex::encode(hash(s))
        }

        let filter = et::Filter::new()
            .select(1234..)
            .address(
                "0xff00000000000000000000000000000000000064"
                    .parse::<et::Address>()
                    .unwrap(),
            )
            .events(vec!["Foo", "Bar"])
            .topic1(hash("Alice"))
            .topic2(
                vec!["Bob", "Charlie"]
                    .into_iter()
                    .map(hash)
                    .collect::<Vec<_>>(),
            );

        eprintln!("filter = {filter:?}");

        assert_eq!(
            filter.topics[0],
            Some(et::ValueOrArray::Array(vec![
                Some(hash("Foo")),
                Some(hash("Bar"))
            ]))
        );

        let queries = FilterKind::Logs(Box::new(filter))
            .to_queries()
            .expect("failed to convert");

        assert_eq!(queries.len(), 4);

        for (i, (t1, t3)) in [
            ("Foo", "Bob"),
            ("Bar", "Bob"),
            ("Foo", "Charlie"),
            ("Bar", "Charlie"),
        ]
        .iter()
        .enumerate()
        {
            let q = queries[i].to_string();
            let e = format!("tm.event = 'Tx' AND tx.height >= 1234 AND message.emitter = '100' AND message.t1 = '{}' AND message.t2 = '{}' AND message.t3 = '{}'", hash_hex(t1), hash_hex("Alice"), hash_hex(t3));
            assert_eq!(q, e, "combination {i}");
        }
    }
}
