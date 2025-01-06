use std::collections::HashMap;
use std::sync::Arc;
use ethers::prelude::H160;
use ethers::types::U256;
use tokio::sync::Mutex;

pub struct NonceManager {
    nonces: Arc<Mutex<HashMap<H160, U256>>>
}

impl NonceManager {
    pub fn new() -> Self {
        NonceManager {
            nonces: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn set(&self, addr: H160, amount: U256) {
        let mut nonces = self.nonces.lock().await;
        nonces.insert(addr, amount);
        println!("insert {:?} {:?}", addr, amount);
    }

    pub async fn get_and_increment(&self, addr: H160) -> U256 {
        let mut nonces = self.nonces.lock().await;
        let next_nonce = nonces.entry(addr).or_insert_with(U256::zero);
        let current_nonce = (*next_nonce).clone();
        *next_nonce += U256::one();
        println!("get_and_increment {:?} {:?}", addr, current_nonce);
        current_nonce
    }
}