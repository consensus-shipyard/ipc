use std::collections::HashMap;

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, PartialEq, Eq)]
pub struct ReputationRecord {
    pub score: u8,
    pub tier: String,
    pub evidence_cid: String,
    pub period: String,
    pub agent_address: [u8; 20],
    pub block_height: u64,
    pub timestamp: u64,
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct State {
    pub records: HashMap<[u8; 20], ReputationRecord>,
    pub authorised_agents: Vec<[u8; 20]>,
    pub admin: Address,
}

impl State {
    pub fn new(admin: Address, initial_agents: Vec<[u8; 20]>) -> Self {
        Self {
            records: HashMap::new(),
            authorised_agents: initial_agents,
            admin,
        }
    }

    pub fn is_authorised(&self, agent: &[u8; 20]) -> bool {
        self.authorised_agents.iter().any(|a| a == agent)
    }

    pub fn add_agent(&mut self, agent: [u8; 20]) {
        if !self.authorised_agents.iter().any(|a| a == &agent) {
            self.authorised_agents.push(agent);
        }
    }

    pub fn remove_agent(&mut self, agent: &[u8; 20]) {
        self.authorised_agents.retain(|a| a != agent);
    }
}
