// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ethers::prelude::H160;
use ethers::types::U256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Default)]
pub struct NonceManager {
    nonces: Arc<RwLock<HashMap<H160, Arc<Mutex<U256>>>>>,
}

impl NonceManager {
    pub async fn get_and_increment(&self, addr: H160) -> U256 {
        if let Some(nonce_lock) = self.nonces.read().await.get(&addr) {
            let mut nonce = nonce_lock.lock().await;
            let current_nonce = *nonce;
            *nonce += U256::one();
            return current_nonce;
        }

        let mut nonces = self.nonces.write().await;
        let nonce_lock = nonces
            .entry(addr)
            .or_insert_with(|| Arc::new(Mutex::new(U256::zero())));
        let mut nonce = nonce_lock.lock().await;
        let current_nonce = *nonce;
        *nonce += U256::one();
        current_nonce
    }
}
