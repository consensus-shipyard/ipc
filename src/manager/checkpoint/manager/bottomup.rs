use std::fmt::{Display, Formatter};
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::Subnet;
use crate::lotus::client::DefaultLotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::proof::create_proof;
use crate::manager::checkpoint::{chain_head_cid, CheckpointManager};
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::MethodNum;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use primitives::TCid;

pub struct BottomUpCheckpointManager<T> {
    parent: Subnet,
    parent_client: T,
    child_subnet: Subnet,
    child_client: DefaultLotusJsonRPCClient,

    checkpoint_period: ChainEpoch,
}

impl<T: LotusClient + Send + Sync> BottomUpCheckpointManager<T> {
    pub fn new_with_period(
        parent: Subnet,
        parent_client: T,
        child_subnet: Subnet,
        child_client: DefaultLotusJsonRPCClient,
        checkpoint_period: ChainEpoch,
    ) -> Self {
        Self {
            parent,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
        }
    }

    pub async fn new(
        parent_client: T,
        parent: Subnet,
        child_client: DefaultLotusJsonRPCClient,
        child_subnet: Subnet,
    ) -> anyhow::Result<Self> {
        let checkpoint_period = obtain_checkpoint_period(&child_subnet.id, &parent_client).await?;
        Ok(Self::new_with_period(
            parent,
            parent_client,
            child_subnet,
            child_client,
            checkpoint_period,
        ))
    }

    async fn proof(&self, epoch: ChainEpoch) -> anyhow::Result<Vec<u8>> {
        let child_chain_head_tip_sets = self.child_client.chain_head().await?.cids;
        if child_chain_head_tip_sets.is_empty() {
            return Err(anyhow!(
                "chain head has empty cid: {:}",
                self.child_subnet.id
            ));
        }
        let proof = create_proof(&self.child_client, epoch).await?;
        Ok(cbor::serialize(&proof, "bottom up checkpoint proof")?.to_vec())
    }
}

#[async_trait]
impl<T: LotusClient + Send + Sync> CheckpointManager for BottomUpCheckpointManager<T> {
    type LotusClient = T;

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
        let parent_head = self.parent_client.chain_head().await?;

        // A key assumption we make now is that each block has exactly one tip set. We panic
        // if this is not the case as it violates our assumption.
        // TODO: update this logic once the assumption changes (i.e., mainnet)
        assert_eq!(parent_head.cids.len(), 1);

        let cid_map = parent_head.cids.first().unwrap().clone();
        let parent_tip_set = Cid::try_from(cid_map)?;
        // get subnet actor state and last checkpoint executed
        let subnet_actor_state = self
            .parent_client
            .ipc_read_subnet_actor_state(&self.child_subnet.id, parent_tip_set)
            .await?;

        Ok(subnet_actor_state
            .bottom_up_checkpoint_voting
            .last_voting_executed)
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_client.current_epoch().await
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        let mut checkpoint = BottomUpCheckpoint::new(self.child_subnet.id.clone(), epoch);

        // From the template on the gateway actor of the child subnet, we get the children checkpoints
        // and the bottom-up cross-net messages.
        log::debug!(
            "Getting checkpoint bottom-up template for {epoch:} in subnet: {:?}",
            self.child_subnet.id
        );
        let template = self
            .child_client
            .ipc_get_checkpoint_template(&self.child_subnet.gateway_addr, epoch)
            .await
            .map_err(|e| {
                anyhow!(
                    "error getting bottom-up checkpoint template for epoch:{epoch:} in subnet: {:?} due to {e:}",
                    self.child_subnet.id
                )
            })?;
        checkpoint.data.children = template.data.children;
        checkpoint.data.cross_msgs = template.data.cross_msgs;
        checkpoint.data.proof = self.proof(epoch).await?;

        // Get the CID of previous checkpoint of the child subnet from the gateway actor of the parent
        // subnet.
        log::debug!(
            "getting previous checkpoint bottom-up from parent gateway for {epoch:} in subnet: {:?}",
            self.child_subnet.id
        );
        let response = self
            .parent_client
            .ipc_get_prev_checkpoint_for_child(&self.child_subnet.gateway_addr, &self.child_subnet.id)
            .await
            .map_err(|e| {
                anyhow!(
                    "error getting previous bottom-up checkpoint for epoch:{epoch:} in subnet: {:?} due to {e:}",
                    self.child_subnet.id
                )
            })?;

        // if previous checkpoint is set
        if response.is_some() {
            let cid = Cid::try_from(response.unwrap())?;
            checkpoint.data.prev_check = TCid::from(cid);
        }

        log::info!(
            "checkpoint at epoch {:} contains {:} number of cross messages, cid: {:} for manager: {:} and validator: {:}",
            checkpoint.data.epoch,
            checkpoint
                .data
                .cross_msgs
                .cross_msgs
                .as_ref()
                .map(|s| s.len())
                .unwrap_or_default(),
            checkpoint.cid(),
            self,
            validator,
        );

        let to = self.child_subnet.id.subnet_actor();
        let message = MpoolPushMessage::new(
            to,
            *validator,
            ipc_subnet_actor::Method::SubmitCheckpoint as MethodNum,
            cbor::serialize(&checkpoint, "checkpoint")?.to_vec(),
        );
        let message_cid = self.parent_client.mpool_push(message).await.map_err(|e| {
            anyhow!(
                "error submitting checkpoint for epoch {epoch:} in subnet: {:?} with reason {e:}",
                self.child_subnet.id
            )
        })?;
        log::debug!("checkpoint message published with cid: {message_cid:?}");

        self.parent_client.state_wait_msg(message_cid).await?;

        Ok(())
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        log::debug!(
            "attempt to obtain the next submission epoch in bottom up checkpoint for subnet: {:?}",
            self.child_subnet.id
        );

        let has_voted = self
            .parent_client
            .ipc_validator_has_voted_bottomup(&self.child_subnet.id, epoch, validator)
            .await?;

        // we should vote only when the validator has not voted
        Ok(!has_voted)
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}

impl<T> Display for BottomUpCheckpointManager<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bottom-up, parent: {:}, child: {:}",
            self.parent.id, self.child_subnet.id
        )
    }
}

async fn obtain_checkpoint_period<T: LotusClient + Send + Sync>(
    subnet_id: &SubnetID,
    parent_client: &T,
) -> anyhow::Result<ChainEpoch> {
    let parent_tip_set = chain_head_cid(parent_client).await?;
    let state = parent_client
        .ipc_read_subnet_actor_state(subnet_id, parent_tip_set)
        .await
        .map_err(|e| {
            log::error!("error getting subnet actor state for {:?}", subnet_id);
            e
        })?;

    Ok(state.bottom_up_check_period)
}
