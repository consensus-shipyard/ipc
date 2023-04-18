// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use cid::Cid;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_gateway::checkpoint::BatchCrossMsgs;
use ipc_gateway::{BottomUpCheckpoint, CrossMsg, Status, StorableMsg};
use ipc_sdk::address::IPCAddress;
use ipc_sdk::subnet_id::SubnetID;
use primitives::TCid;
use serde::{Deserialize, Serialize};

use crate::lotus::message::deserialize::{
    deserialize_ipc_address_from_map, deserialize_subnet_id_from_map,
    deserialize_token_amount_from_str,
};
use crate::lotus::message::serialize::{
    serialize_subnet_id_to_str, serialize_token_amount_to_atto,
};
use crate::lotus::message::CIDMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCGetPrevCheckpointForChildResponse {
    #[serde(rename = "CID")]
    pub cid: Option<CIDMap>,
}

/// The state of a gateway actor. The struct omits all fields that are not relevant for the
/// execution of the IPC agent.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCReadGatewayStateResponse {
    pub bottom_up_check_period: ChainEpoch,
    pub top_down_check_period: ChainEpoch,
    pub applied_topdown_nonce: u64,
    pub top_down_checkpoint_voting: Voting,
    pub initialized: bool,
}

/// The state of a subnet actor. The struct omits all fields that are not relevant for the
/// execution of the IPC agent.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCReadSubnetActorStateResponse {
    pub bottom_up_check_period: ChainEpoch,
    pub validator_set: ValidatorSet,
    pub min_validators: u64,
    pub bottom_up_checkpoint_voting: Voting,
}

/// A subset of the voting structure with information
/// about a checkpoint voting
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Voting {
    pub genesis_epoch: i64,
    pub last_voting_executed: i64,
}

/// SubnetInfo is an auxiliary struct that collects relevant information about the state of a subnet
///
/// Note that the serialization and deserialization casing are different. Reason because for deserialization,
/// it is from the fvm actor, which is `PascalCase`. When serialize, we are using rust's default casing
#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetInfo {
    /// Id of the subnet.
    #[serde(rename(deserialize = "ID"))]
    #[serde(deserialize_with = "deserialize_subnet_id_from_map")]
    #[serde(serialize_with = "serialize_subnet_id_to_str")]
    pub id: SubnetID,
    /// Collateral staked in the subnet.
    #[serde(rename(deserialize = "Stake"))]
    #[serde(deserialize_with = "deserialize_token_amount_from_str")]
    #[serde(serialize_with = "serialize_token_amount_to_atto")]
    pub stake: TokenAmount,
    /// Circulating supply available in the subnet.
    #[serde(rename(deserialize = "CircSupply"))]
    #[serde(deserialize_with = "deserialize_token_amount_from_str")]
    #[serde(serialize_with = "serialize_token_amount_to_atto")]
    pub circ_supply: TokenAmount,
    /// State of the Subnet (Initialized, Active, Killed)
    #[serde(rename(deserialize = "Status"))]
    pub status: Status,
}

/// We need to redefine the struct here due to:
/// In the actor, it is `Deserialize_tuple`, but when returned from json rpc endpoints, it's
/// actually `json` struct. The deserialization is not working because the agent is interpreting
/// the tuple as json.
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSet {
    pub validators: Option<Vec<Validator>>,
    // sequence number that uniquely identifies a validator set
    pub configuration_number: u64,
}

/// The validator struct. See `ValidatorSet` comment on why we need this duplicated definition.
#[derive(Deserialize, Serialize, Debug)]
pub struct Validator {
    pub addr: String,
    pub net_addr: String,
    pub weight: String,
}

/// This deserializes from the `gateway::BottomUpCheckpoint`, we need to redefine
/// here because the Lotus API json serializes and the cbor tuple deserializer is not
/// able to pick it up automatically
#[derive(Deserialize, Serialize, Debug)]
pub struct BottomUpCheckpointWrapper {
    #[serde(rename(deserialize = "Data"))]
    pub data: CheckDataWrapper,
    #[serde(rename(deserialize = "Sig"))]
    #[serde(with = "serde_bytes")]
    pub sig: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CheckDataWrapper {
    #[serde(rename(deserialize = "Source"))]
    #[serde(deserialize_with = "deserialize_subnet_id_from_map")]
    pub source: SubnetID,
    #[serde(rename(deserialize = "Proof"))]
    #[serde(with = "serde_bytes")]
    pub proof: Option<Vec<u8>>,
    #[serde(rename(deserialize = "Epoch"))]
    pub epoch: i64,
    #[serde(rename(deserialize = "PrevCheck"))]
    pub prev_check: Option<CIDMap>,
    #[serde(rename(deserialize = "Children"))]
    pub children: Option<Vec<CheckData>>,
    #[serde(rename(deserialize = "CrossMsgs"))]
    pub cross_msgs: Option<BatchCrossMsgsWrapper>,
}

#[derive(PartialEq, Eq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct BatchCrossMsgsWrapper {
    #[serde(rename(deserialize = "CrossMsgs"))]
    pub cross_msgs: Option<Vec<CrossMsgsWrapper>>,
    #[serde(rename(deserialize = "Fee"))]
    #[serde(deserialize_with = "deserialize_token_amount_from_str")]
    pub fee: TokenAmount,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CrossMsgsWrapper {
    pub msg: StorableMsgsWrapper,
    pub wrapped: bool,
}

#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorableMsgsWrapper {
    #[serde(deserialize_with = "deserialize_ipc_address_from_map")]
    pub from: IPCAddress,
    #[serde(deserialize_with = "deserialize_ipc_address_from_map")]
    pub to: IPCAddress,
    pub method: MethodNum,
    pub params: RawBytes,
    pub value: TokenAmount,
    pub nonce: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CheckData {
    #[serde(rename(deserialize = "Source"))]
    #[serde(deserialize_with = "deserialize_subnet_id_from_map")]
    pub source: SubnetID,
    #[serde(rename(deserialize = "Checks"))]
    pub checks: Vec<CIDMap>,
}

impl From<BatchCrossMsgsWrapper> for BatchCrossMsgs {
    fn from(wrapper: BatchCrossMsgsWrapper) -> Self {
        let cross_msgs = wrapper.cross_msgs.map(|cross_msgs| {
            cross_msgs
                .into_iter()
                .map(|cross_wrapper| CrossMsg {
                    msg: StorableMsg {
                        from: cross_wrapper.msg.from,
                        to: cross_wrapper.msg.to,
                        method: cross_wrapper.msg.method,
                        params: cross_wrapper.msg.params,
                        value: cross_wrapper.msg.value,
                        nonce: cross_wrapper.msg.nonce,
                    },
                    wrapped: cross_wrapper.wrapped,
                })
                .collect::<Vec<_>>()
        });

        BatchCrossMsgs {
            cross_msgs,
            fee: wrapper.fee,
        }
    }
}

impl TryFrom<BottomUpCheckpointWrapper> for BottomUpCheckpoint {
    type Error = anyhow::Error;

    fn try_from(checkpoint_response: BottomUpCheckpointWrapper) -> Result<Self, Self::Error> {
        let prev_check = if let Some(prev_check) = checkpoint_response.data.prev_check {
            TCid::from(Cid::try_from(prev_check)?)
        } else {
            TCid::default()
        };
        log::debug!("previous checkpoint: {prev_check:?}");

        let children = if let Some(children) = checkpoint_response.data.children {
            children
                .into_iter()
                .map::<Result<_, Self::Error>, _>(|c| {
                    let checks: Result<Vec<_>, _> = c
                        .checks
                        .into_iter()
                        .map(|cid_map| Cid::try_from(cid_map).map(TCid::from))
                        .collect();

                    Ok(ipc_gateway::checkpoint::ChildCheck {
                        source: c.source,
                        checks: checks?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };
        log::debug!("children: {children:?}");

        let cross_msgs = if let Some(cross_msgs) = checkpoint_response.data.cross_msgs {
            BatchCrossMsgs::from(cross_msgs)
        } else {
            BatchCrossMsgs::default()
        };
        log::debug!("cross_msgs: {cross_msgs:?}");

        let data = ipc_gateway::checkpoint::CheckData {
            source: checkpoint_response.data.source,
            proof: checkpoint_response.data.proof.unwrap_or_default(),
            epoch: checkpoint_response.data.epoch,
            prev_check,
            children,
            cross_msgs,
        };
        Ok(BottomUpCheckpoint {
            data,
            sig: checkpoint_response.sig.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::lotus::message::ipc::IPCReadSubnetActorStateResponse;

    #[test]
    fn deserialize_ipc_subnet_state() {
        let raw = r#"
        {"Name":"test2","ParentID":{"Parent":"/root","Actor":"t00"},"IPCGatewayAddr":"t064","Consensus":3,"MinValidatorStake":"1000000000000000000","TotalStake":"10000000000000000000","Stake":{"/":"bafy2bzacebentzoqaapingrxwknlxqcusl23rqaa7cwb42u76fgvb25nxpmhq"},"Status":1,"Genesis":null,"BottomUpCheckPeriod":10,"TopDownCheckPeriod":10,"GenesisEpoch":0,"CommittedCheckpoints":{"/":"bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay"},"ValidatorSet":{"validators":[{"addr":"t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq","net_addr":"test","weight":"10000000000000000000"}],"configuration_number":1},"MinValidators":1,"PreviousExecutedCheckpoint":{"/":"bafy2bzacedkoa623kvi5gfis2yks7xxjl73vg7xwbojz4tpq63dd5jpfz757i"},"BottomUpCheckpointVoting":{"GenesisEpoch":0,"SubmissionPeriod":10,"LastVotingExecuted":0,"ExecutableEpochQueue":null,"EpochVoteSubmission":{"/":"bafy2bzaceamp42wmmgr2g2ymg46euououzfyck7szknvfacqscohrvaikwfay"},"Ratio":{"Num":2,"Denom":3}}}
        "#;

        let r = serde_json::from_str::<IPCReadSubnetActorStateResponse>(raw);
        assert!(r.is_ok());
    }
}
