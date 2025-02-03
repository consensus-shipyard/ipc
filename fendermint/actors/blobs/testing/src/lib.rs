// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::{Hash, PublicKey, SubscriptionId};
use fvm_shared::address::Address;
use rand::distributions::Alphanumeric;
use rand::{Rng, RngCore};

pub fn setup_logs() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format().with_line_number(true))
                .with_writer(std::io::stdout),
        )
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();
}

pub fn new_hash(size: usize) -> (Hash, u64) {
    let mut rng = rand::thread_rng();
    let mut data = vec![0u8; size];
    rng.fill_bytes(&mut data);
    (
        Hash(*iroh_base::hash::Hash::new(&data).as_bytes()),
        size as u64,
    )
}

pub fn new_hash_from_vec(buf: Vec<u8>) -> (Hash, u64) {
    (
        Hash(*iroh_base::hash::Hash::new(&buf).as_bytes()),
        buf.len() as u64,
    )
}

pub fn new_metadata_hash() -> Hash {
    let mut rng = rand::thread_rng();
    let mut data = vec![0u8; 8];
    rng.fill_bytes(&mut data);
    Hash(*iroh_base::hash::Hash::new(&data).as_bytes())
}

pub fn new_pk() -> PublicKey {
    let mut rng = rand::thread_rng();
    let mut data = [0u8; 32];
    rng.fill_bytes(&mut data);
    PublicKey(data)
}

pub fn new_address() -> Address {
    let mut rng = rand::thread_rng();
    let mut data = vec![0u8; 32];
    rng.fill_bytes(&mut data);
    Address::new_actor(&data)
}

pub fn new_subscription_id(length: usize) -> SubscriptionId {
    let str: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    SubscriptionId::try_from(str).unwrap()
}
