pub mod registry;
pub mod verify;

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;

use registry::{ReputationRecord, State};
use verify::recover_address;

pub const EXIT_ILLEGAL_ARGUMENT: u32 = 16;
pub const EXIT_FORBIDDEN: u32 = 18;

#[derive(Debug, Clone)]
pub struct ActorError {
    pub code: u32,
    pub message: String,
}

impl ActorError {
    pub fn illegal_argument(message: impl Into<String>) -> Self {
        Self {
            code: EXIT_ILLEGAL_ARGUMENT,
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: EXIT_FORBIDDEN,
            message: message.into(),
        }
    }
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct SetScoreParams {
    pub developer: [u8; 20],
    pub github_handle: String,
    pub score: u8,
    pub tier: String,
    pub evidence_cid: String,
    pub period: String,
    pub document_hash: [u8; 32],
    pub agent_address: [u8; 20],
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ReputationActor {
    pub state: State,
}

impl ReputationActor {
    pub fn constructor(admin: Address, initial_agent: [u8; 20]) -> Self {
        Self {
            state: State::new(admin, vec![initial_agent]),
        }
    }

    pub fn set_score(
        &mut self,
        caller: [u8; 20],
        params: SetScoreParams,
        block_height: u64,
        timestamp: u64,
    ) -> Result<(), ActorError> {
        let recovered = recover_address(params.document_hash, &params.signature)
            .map_err(|e| ActorError::illegal_argument(format!("signature verification failed: {e}")))?;

        if recovered != params.agent_address {
            return Err(ActorError::illegal_argument("recovered signer does not match agent_address"));
        }

        if !self.state.is_authorised(&params.agent_address) || !self.state.is_authorised(&caller) {
            return Err(ActorError::forbidden("caller or agent not authorised"));
        }

        let record = ReputationRecord {
            score: params.score,
            tier: params.tier,
            evidence_cid: params.evidence_cid,
            period: params.period,
            agent_address: params.agent_address,
            block_height,
            timestamp,
        };
        self.state.records.insert(params.developer, record);
        Ok(())
    }

    pub fn get_score(&self, developer: [u8; 20]) -> Option<ReputationRecord> {
        self.state.records.get(&developer).cloned()
    }

    pub fn add_agent(&mut self, caller_admin: &Address, agent: [u8; 20]) -> Result<(), ActorError> {
        if caller_admin != &self.state.admin {
            return Err(ActorError::forbidden("only admin can add agents"));
        }
        self.state.add_agent(agent);
        Ok(())
    }

    pub fn remove_agent(&mut self, caller_admin: &Address, agent: [u8; 20]) -> Result<(), ActorError> {
        if caller_admin != &self.state.admin {
            return Err(ActorError::forbidden("only admin can remove agents"));
        }
        self.state.remove_agent(&agent);
        Ok(())
    }

    pub fn get_authorised_agents(&self) -> Vec<[u8; 20]> {
        self.state.authorised_agents.clone()
    }
}
