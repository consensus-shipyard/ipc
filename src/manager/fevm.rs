use crate::checkpoint::{
    create_proof, BottomUpHandler, Bytes, NativeBottomUpCheckpoint, VoteQuery,
};
use crate::jsonrpc::JsonRpcClientImpl;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::ipc::QueryValidatorSetResponse;
use crate::lotus::message::wallet::WalletKeyType;
use crate::manager::{EthManager, EthSubnetManager, SubnetInfo, SubnetManager};
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::ConstructParams;
use std::collections::HashMap;

pub struct FevmSubnetManager {
    evm_subnet_manager: EthSubnetManager,
    lotus_client: LotusJsonRPCClient<JsonRpcClientImpl>,
}

impl VoteQuery<NativeBottomUpCheckpoint> for FevmSubnetManager {
    async fn last_executed_epoch(&self, _subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .gateway_last_voting_executed_epoch()
            .await
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager.current_epoch().await
    }

    async fn has_voted(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<bool> {
        self.evm_subnet_manager
            .has_voted_in_subnet(subnet_id, epoch, validator)
            .await
    }
}

#[async_trait]
impl BottomUpHandler for FevmSubnetManager {
    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.evm_subnet_manager
            .subnet_bottom_up_checkpoint_period(subnet_id)
            .await
    }

    async fn validators(&self, subnet_id: &SubnetID) -> anyhow::Result<Vec<Address>> {
        self.evm_subnet_manager.validators(subnet_id).await
    }

    async fn checkpoint_template(
        &self,
        epoch: ChainEpoch,
    ) -> anyhow::Result<NativeBottomUpCheckpoint> {
        let _checkpoint = self.evm_subnet_manager.bottom_up_checkpoint(epoch).await?;

        // TODO: impl type conversion to NativeBottomUpCheckpoint
        todo!()
    }

    async fn populate_prev_hash(
        &self,
        template: &mut NativeBottomUpCheckpoint,
    ) -> anyhow::Result<()> {
        let proof = create_proof(&self.lotus_client, template.epoch).await?;
        let proof_bytes = cbor::serialize(&proof, "fevm bottom up checkpoint proof")?.to_vec();
        template.proof = Some(proof_bytes);
        Ok(())
    }

    async fn populate_proof(&self, template: &mut NativeBottomUpCheckpoint) -> anyhow::Result<()> {
        if template.proof.is_none() {
            log::warn!("fevm template should have proof");
        }
        Ok(())
    }

    async fn submit(
        &self,
        _validator: &Address,
        _checkpoint: NativeBottomUpCheckpoint,
    ) -> anyhow::Result<ChainEpoch> {
        // TODO: implement type conversion
        todo!()
    }
}
