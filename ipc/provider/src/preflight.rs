//! The one place where the parameters send to ipc contracts are preprocessed, i.e.
//! setting default/missing parameter, checking if the parameters are valid.

use crate::config::{Config, Subnet};
use anyhow::anyhow;
use ipc_api::subnet::{ConsensusType, ConstructParams};
use ipc_api::subnet_id::SubnetID;
use std::sync::Arc;

const DEFAULT_ACTIVE_VALIDATORS: u16 = 100;
const DEFAULT_POWER_SCALE: i8 = 3;
const DEFAULT_SUBNET_CONSENSUS_TYPE: ConsensusType = ConsensusType::Fendermint;
/// The majority vote percentage for checkpoint submission when creating a subnet.
const SUBNET_MAJORITY_PERCENTAGE: u8 = 67;

/// The one place where the parameters send to ipc contracts are preprocessed, i.e.
/// setting default/missing parameter, checking if the parameters are valid.
#[derive(Clone)]
pub struct Preflight {
    pub config: Arc<Config>,
}

impl Preflight {
    pub fn create_subnet(&self, mut params: ConstructParams) -> anyhow::Result<ConstructParams> {
        let config = self.config(&params.parent)?;

        if params.ipc_gateway_addr.is_none() {
            params.ipc_gateway_addr = Some(config.gateway_addr());
        }

        if params.active_validators_limit.is_none() {
            params.active_validators_limit = Some(DEFAULT_ACTIVE_VALIDATORS);
        }

        if params.power_scale.is_none() {
            params.power_scale = Some(DEFAULT_POWER_SCALE);
        }

        if params.consensus.is_none() {
            params.consensus = Some(DEFAULT_SUBNET_CONSENSUS_TYPE);
        }

        if params.majority_percentage.is_none() {
            params.majority_percentage = Some(SUBNET_MAJORITY_PERCENTAGE);
        }

        Ok(params)
    }

    /// Get the connection instance for the subnet.
    fn config(&self, subnet: &SubnetID) -> anyhow::Result<&Subnet> {
        self.config
            .subnets
            .get(subnet)
            .ok_or_else(|| anyhow!("subnet config does not exist {}", subnet))
    }
}
