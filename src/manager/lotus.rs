use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};

use super::subnet::{SubnetInfo, SubnetManager};

pub struct LotusSubnetManager {}

#[async_trait]
impl SubnetManager for LotusSubnetManager {
    async fn create_subnet(
        &self,
        _from: Address,
        _params: ConstructParams,
    ) -> Result<Address> {
        panic!("not implemented")
    }

    async fn join_subnet(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
        _params: JoinParams,
    ) -> Result<()> {
        panic!("not implemented")
    }

    async fn leave_subnet(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        panic!("not implemented")
    }

    async fn kill_subnet(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        panic!("not implemented")
    }

    async fn submit_checkpoint(&self, _subnet: SubnetID, _from: Address, _ch: Checkpoint) -> Result<()> {
        panic!("not implemented")
    }

    async fn list_child_subnets(&self, _subnet: SubnetID) -> Result<HashMap<SubnetID, SubnetInfo>> {
        panic!("not implemented")
    }
}
