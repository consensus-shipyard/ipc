// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use anyhow::anyhow;
use std::fmt::{Display, Formatter};

use crate::manager::checkpoint::{chain_head_cid, CheckpointManager};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;

use crate::config::Subnet;
use crate::lotus::client::DefaultLotusJsonRPCClient;
use crate::lotus::message::ipc::IPCReadGatewayStateResponse;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::MethodNum;
use ipc_gateway::TopDownCheckpoint;

pub struct TopDownCheckpointManager {
    parent: Subnet,
    parent_client: DefaultLotusJsonRPCClient,
    child_subnet: Subnet,
    child_client: DefaultLotusJsonRPCClient,

    checkpoint_period: ChainEpoch,
}

impl TopDownCheckpointManager {
    pub async fn new(
        parent_client: DefaultLotusJsonRPCClient,
        parent: Subnet,
        child_client: DefaultLotusJsonRPCClient,
        child_subnet: Subnet,
    ) -> anyhow::Result<Self> {
        let child_tip_set = chain_head_cid(&child_client).await?;
        let state = child_client
            .ipc_read_gateway_state(&child_subnet.gateway_addr, child_tip_set)
            .await?;
        let checkpoint_period = state.top_down_check_period;

        Ok(Self {
            parent,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
        })
    }
}

impl Display for TopDownCheckpointManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "top-down, parent: {:}, child: {:}",
            self.parent.id, self.child_subnet.id
        )
    }
}

impl TopDownCheckpointManager {
    async fn child_gateway_state(&self) -> anyhow::Result<IPCReadGatewayStateResponse> {
        let child_head = self.child_client.chain_head().await?;
        let cid_map = child_head.cids.first().unwrap().clone();
        let child_tip_set = Cid::try_from(cid_map)?;

        self.child_client
            .ipc_read_gateway_state(&self.child_subnet.gateway_addr, child_tip_set)
            .await
    }

    async fn parent_head(&self) -> anyhow::Result<Cid> {
        chain_head_cid(&self.parent_client).await
    }

    async fn child_head(&self) -> anyhow::Result<Cid> {
        chain_head_cid(&self.child_client).await
    }

    async fn submission_tipset(&self, epoch: ChainEpoch) -> anyhow::Result<Cid> {
        let submission_tip_set = self
            .parent_client
            .get_tipset_by_height(epoch, self.parent_head().await?)
            .await?;
        let cid_map = submission_tip_set.cids.first().unwrap().clone();
        Cid::try_from(cid_map)
    }
}

#[async_trait]
impl CheckpointManager for TopDownCheckpointManager {
    type LotusClient = DefaultLotusJsonRPCClient;

    fn parent_client(&self) -> &Self::LotusClient {
        &self.parent_client
    }

    fn parent_subnet(&self) -> &Subnet {
        &self.parent
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child_subnet
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let child_gw_state = self.child_gateway_state().await?;
        Ok(child_gw_state
            .top_down_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        let parent_head = self.parent_client.chain_head().await?;
        Ok(ChainEpoch::try_from(parent_head.height)?)
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        let nonce = self
            .child_client
            .ipc_read_gateway_state(&self.child_subnet.gateway_addr, self.child_head().await?)
            .await?
            .applied_topdown_nonce;

        let submission_tip_set = self.submission_tipset(epoch).await?;
        let top_down_msgs = self
            .parent_client
            .ipc_get_topdown_msgs(
                &self.child_subnet.id,
                &self.parent.gateway_addr,
                submission_tip_set,
                nonce,
            )
            .await?;

        log::debug!(
            "nonce: {:} for submission tip set: {:} at epoch {:} of manager: {:}, size of top down messages: {:}",
            nonce, submission_tip_set, epoch, self, top_down_msgs.len()
        );

        // we submit the topdown messages to the CHILD subnet.
        let topdown_checkpoint = TopDownCheckpoint {
            epoch,
            top_down_msgs,
        };
        let message = MpoolPushMessage::new(
            self.parent.gateway_addr,
            *validator,
            ipc_gateway::Method::SubmitTopDownCheckpoint as MethodNum,
            cbor::serialize(&topdown_checkpoint, "topdown_checkpoint")?.to_vec(),
        );
        let message_cid = self.child_client.mpool_push(message).await.map_err(|e| {
            log::error!("error submitting checkpoint at epoch {epoch:} for manager: {self:}");
            e
        })?;
        log::debug!(
            "checkpoint at epoch {:} for manager: {:} published with cid: {:?}, wait for execution",
            epoch,
            self,
            message_cid,
        );
        self.child_client.state_wait_msg(message_cid).await?;
        log::debug!(
            "checkpoint at epoch {:} for manager: {:} published with cid: {:?}, executed",
            epoch,
            self,
            message_cid,
        );

        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        let has_voted = self
            .child_client
            .ipc_validator_has_voted_topdown(&self.child_subnet.gateway_addr, epoch, validator)
            .await
            .map_err(|e| {
                anyhow!("error checking if validator has voted for manager: {self:} due to {e:}")
            })?;

        // we should vote only when the validator has not voted
        Ok(!has_voted)
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        let state = self.child_gateway_state().await?;
        Ok(state.initialized)
    }
}
