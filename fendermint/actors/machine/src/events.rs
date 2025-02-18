// Copyright 2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{actor_error, runtime::Runtime, ActorError};
use fvm_ipld_encoding::IPLD_RAW;
use fvm_shared::event::{ActorEvent, Entry, Flags};
use recall_sol_facade::primitives::IntoLogData;

/// The event key prefix for the Ethereum log topics.
const EVENT_TOPIC_KEY_PREFIX: &str = "t";

/// The event key for the Ethereum log data.
const EVENT_DATA_KEY: &str = "d";

/// Returns an [`ActorEvent`] from an EVM event.
pub fn to_actor_event<T: IntoLogData>(event: T) -> Result<ActorEvent, ActorError> {
    let log = event.to_log_data();
    let num_entries = log.topics().len() + 1; // +1 for log data

    let mut entries: Vec<Entry> = Vec::with_capacity(num_entries);
    for (i, topic) in log.topics().iter().enumerate() {
        let key = format!("{}{}", EVENT_TOPIC_KEY_PREFIX, i + 1);
        entries.push(Entry {
            flags: Flags::FLAG_INDEXED_ALL,
            key,
            codec: IPLD_RAW,
            value: topic.to_vec(),
        });
    }
    entries.push(Entry {
        flags: Flags::FLAG_INDEXED_ALL,
        key: EVENT_DATA_KEY.to_owned(),
        codec: IPLD_RAW,
        value: log.data.to_vec(),
    });

    Ok(entries.into())
}

/// Emits an [`ActorEvent`] from an EVM event.
pub fn emit_evm_event<T: IntoLogData>(
    rt: &impl Runtime,
    event: anyhow::Result<T>,
) -> Result<(), ActorError> {
    let event =
        event.map_err(|e| actor_error!(illegal_argument; "failed to build evm event: {}", e))?;
    let actor_event = to_actor_event(event)?;
    rt.emit_event(&actor_event)
}
