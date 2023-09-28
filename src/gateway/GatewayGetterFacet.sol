// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {CrossMsg, BottomUpCheckpoint, BottomUpCheckpointNew, StorableMsg, ParentFinality, CheckpointInfo} from "../structs/Checkpoint.sol";
import {EpochVoteTopDownSubmission} from "../structs/EpochVoteSubmission.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {Membership} from "../structs/Validator.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {GatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {LibVoting} from "../lib/LibVoting.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

contract GatewayGetterFacet {
    // slither-disable-next-line uninitialized-state
    GatewayActorStorage internal s;

    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using EnumerableSet for EnumerableSet.UintSet;

    function crossMsgFee() external view returns (uint256) {
        return s.crossMsgFee;
    }

    function bottomUpNonce() external view returns (uint64) {
        return s.bottomUpNonce;
    }

    function totalSubnets() external view returns (uint64) {
        return s.totalSubnets;
    }

    function minStake() external view returns (uint256) {
        return s.minStake;
    }

    function initialized() external view returns (bool) {
        return s.initialized;
    }

    function bottomUpCheckPeriod() external view returns (uint64) {
        return s.bottomUpCheckPeriod;
    }

    function topDownCheckPeriod() external view returns (uint64) {
        return s.topDownCheckPeriod;
    }

    function getNetworkName() external view returns (SubnetID memory) {
        return s.networkName;
    }

    // TODO: remove or add a new getter
    function bottomUpCheckpoints(uint64 e) external view returns (BottomUpCheckpoint memory) {
        return s.bottomUpCheckpointsLegacy[e];
    }

    function getParentFinality(uint256 blockNumber) external view returns (ParentFinality memory) {
        return LibGateway.getParentFinality(blockNumber);
    }

    function getLatestParentFinality() external view returns (ParentFinality memory) {
        return LibGateway.getLatestParentFinality();
    }

    /// @notice returns the subnet with the given id
    /// @param subnetId the id of the subnet
    /// @return found whether the subnet exists
    /// @return subnet -  the subnet struct
    function getSubnet(SubnetID calldata subnetId) external view returns (bool, Subnet memory) {
        // slither-disable-next-line unused-return
        return LibGateway.getSubnet(subnetId);
    }

    function subnets(bytes32 h) external view returns (Subnet memory subnet) {
        return s.subnets[h];
    }

    /// @notice get number of top-down messages for the given subnet
    function getSubnetTopDownMsgsLength(SubnetID memory subnetId) external view returns (uint256) {
        // slither-disable-next-line unused-return
        (, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        // With every new message, the nonce is added by one, the current nonce should be equal to the top down message length.
        return subnet.topDownNonce;
    }

    /// @notice get the list of top down messages from block number, we may also consider introducing pagination.
    /// @param subnetId - The subnet id to fetch messages from
    /// @param fromBlock - The starting block to get top down messages, inclusive.
    /// @param toBlock - The ending block to get top down messages, inclusive.
    function getTopDownMsgs(
        SubnetID calldata subnetId,
        uint256 fromBlock,
        uint256 toBlock
    ) external view returns (CrossMsg[] memory) {
        return LibGateway.getTopDownMsgs({subnetId: subnetId, fromBlock: fromBlock, toBlock: toBlock});
    }

    /// @notice Get the latest applied top down nonce
    /// @param subnetId - The subnet id to fetch messages from
    function getAppliedTopDownNonce(SubnetID calldata subnetId) external view returns (bool, uint64) {
        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        if (!registered) {
            return (false, 0);
        }
        return (true, subnet.topDownNonce);
    }

    function appliedTopDownNonce() public view returns (uint64) {
        return s.appliedTopDownNonce;
    }

    function postbox(bytes32 id) public view returns (StorableMsg memory storableMsg, bool wrapped) {
        return (s.postbox[id].message, s.postbox[id].wrapped);
    }

    /// @notice whether a validator has voted for a checkpoint submission during an epoch
    /// @param epoch - the epoch to check
    /// @param submitter - the validator to check
    function hasValidatorVotedForSubmission(uint64 epoch, address submitter) external view returns (bool) {
        EpochVoteTopDownSubmission storage voteSubmission = s.epochVoteSubmissions[epoch];
        return voteSubmission.vote.submitters[voteSubmission.vote.nonce][submitter];
    }

    function getGenesisEpoch() public view returns (uint64) {
        // slither-disable-next-line unused-return
        return LibVoting.getGenesisEpoch();
    }

    function executableQueue() public view returns (uint64, uint64, uint64) {
        // slither-disable-next-line unused-return
        return LibVoting.executableQueue();
    }

    function lastVotingExecutedEpoch() public view returns (uint64) {
        return LibVoting.lastVotingExecutedEpoch();
    }

    function majorityPercentage() public view returns (uint64) {
        return LibVoting.majorityPercentage();
    }

    /// @notice returns the list of registered subnets in IPC
    /// @return subnet - the list of subnets
    function listSubnets() external view returns (Subnet[] memory) {
        uint256 size = s.subnetKeys.length;
        Subnet[] memory out = new Subnet[](size);
        for (uint256 i = 0; i < size; ) {
            bytes32 key = s.subnetKeys[i];
            out[i] = s.subnets[key];
            unchecked {
                ++i;
            }
        }
        return out;
    }

    /// @notice get the last membership received from the parent
    function getLastMembership() external view returns (Membership memory) {
        return s.lastMembership;
    }

    /// @notice get the last configuration number received from the parent
    function getLastConfigurationNumber() external view returns (uint64) {
        return s.lastMembership.configurationNumber;
    }

    /// @notice get the last total weight for the membership received from the parent
    function getLastTotalWeight() public view returns (uint256) {
        return s.lastMembership.totalWeight;
    }

    /// @notice get the current membership
    function getCurrentMembership() external view returns (Membership memory) {
        return s.currentMembership;
    }

    /// @notice get the current configuration number
    function getCurrentConfigurationNumber() external view returns (uint64) {
        return s.currentMembership.configurationNumber;
    }

    /// @notice get the current membership validators total weight
    function getCurrentTotalWeight() public view returns (uint256) {
        return s.currentMembership.totalWeight;
    }

    /// @notice get the checkpoint information corresponding to the block height
    function getCheckpointInfo(uint64 h) public view returns (CheckpointInfo memory) {
        return s.bottomUpCheckpointInfo[h];
    }

    /// @notice get the checkpoint current weight corresponding to the block height
    function getCheckpointCurrentWeight(uint64 h) public view returns (uint256) {
        return s.bottomUpCheckpointInfo[h].currentWeight;
    }

    /// @notice get the incomplete checkpoint heights
    function getIncompleteCheckpointHeights() public view returns (uint256[] memory) {
        return s.incompleteCheckpoints.values();
    }

    /// @notice get the incomplete checkpoints
    function getIncompleteCheckpoints() public view returns (BottomUpCheckpointNew[] memory) {
        uint256[] memory heights = s.incompleteCheckpoints.values();
        uint256 size = heights.length;

        BottomUpCheckpointNew[] memory checkpoints = new BottomUpCheckpointNew[](size);
        for (uint64 i = 0; i < size; ) {
            checkpoints[i] = s.bottomUpCheckpoints[uint64(heights[i])];
            unchecked {
                ++i;
            }
        }
        return checkpoints;
    }

    /// @notice get the bottom-up checkpoint retention index
    function getBottomUpRetentionHeight() public view returns (uint64) {
        return s.bottomUpCheckpointRetentionHeight;
    }

    /// @notice Calculate the threshold required for quorum in this subnet
    /// based on the configured majority percentage and the total weight of the validators.
    function getQuorumThreshold(uint256 totalWeight) public view returns (uint256) {
        return LibGateway.weightNeeded(totalWeight, s.majorityPercentage);
    }
}
