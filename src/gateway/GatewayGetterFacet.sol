// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {BottomUpCheckpoint, BottomUpMsgBatch, StorableMsg, ParentFinality} from "../structs/CrossNet.sol";
import {QuorumInfo} from "../structs/Quorum.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {Membership} from "../structs/Subnet.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {LibQuorum} from "../lib/LibQuorum.sol";
import {GatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

contract GatewayGetterFacet {
    // slither-disable-next-line uninitialized-state
    GatewayActorStorage internal s;

    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.UintSet;
    using EnumerableSet for EnumerableSet.AddressSet;

    function crossMsgFee() external view returns (uint256) {
        return s.minCrossMsgFee;
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

    function maxMsgsPerBottomUpBatch() external view returns (uint64) {
        return s.maxMsgsPerBottomUpBatch;
    }

    function bottomUpMsgBatchPeriod() external view returns (uint256) {
        return s.bottomUpMsgBatchPeriod;
    }

    function bottomUpCheckPeriod() external view returns (uint256) {
        return s.bottomUpCheckPeriod;
    }

    function getNetworkName() external view returns (SubnetID memory) {
        return s.networkName;
    }

    function bottomUpCheckpoint(uint256 e) external view returns (BottomUpCheckpoint memory) {
        return s.bottomUpCheckpoints[e];
    }

    function bottomUpMsgBatch(uint256 e) external view returns (BottomUpMsgBatch memory) {
        return s.bottomUpMsgBatches[e];
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

    /// @notice Get the latest applied top down nonce
    /// @param subnetId - The subnet id to fetch messages from
    function getAppliedTopDownNonce(SubnetID calldata subnetId) external view returns (bool, uint64) {
        (bool registered, Subnet storage subnet) = LibGateway.getSubnet(subnetId);
        if (!registered) {
            return (false, 0);
        }
        return (true, subnet.topDownNonce);
    }

    function appliedTopDownNonce() external view returns (uint64) {
        return s.appliedTopDownNonce;
    }

    function postbox(bytes32 id) external view returns (StorableMsg memory storableMsg, bool wrapped) {
        return (s.postbox[id].message, s.postbox[id].wrapped);
    }

    function majorityPercentage() external view returns (uint64) {
        return s.majorityPercentage;
    }

    /// @notice returns the list of registered subnets in IPC
    /// @return subnet - the list of subnets
    function listSubnets() external view returns (Subnet[] memory) {
        uint256 size = s.subnetKeys.length;
        Subnet[] memory out = new Subnet[](size);
        for (uint256 i; i < size; ) {
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

    /// @notice get the current membership
    function getCurrentMembership() external view returns (Membership memory) {
        return s.currentMembership;
    }

    /// @notice get the current configuration number
    function getCurrentConfigurationNumber() external view returns (uint64) {
        return s.currentMembership.configurationNumber;
    }

    /// @notice get the checkpoint information corresponding to the block height
    function getCheckpointInfo(uint256 h) external view returns (QuorumInfo memory) {
        return s.checkpointQuorumMap.quorumInfo[h];
    }

    function getBottomUpMsgBatchInfo(uint256 h) external view returns (QuorumInfo memory) {
        return s.bottomUpMsgBatchQuorumMap.quorumInfo[h];
    }

    /// @notice get the checkpoint current weight corresponding to the block height
    function getCheckpointCurrentWeight(uint256 h) external view returns (uint256) {
        return s.checkpointQuorumMap.quorumInfo[h].currentWeight;
    }

    /// @notice get the batch current weight corresponding to the block height
    function getBottomUpMsgBatchCurrentWeight(uint256 h) external view returns (uint256) {
        return s.bottomUpMsgBatchQuorumMap.quorumInfo[h].currentWeight;
    }

    /// @notice get the incomplete checkpoint heights
    function getIncompleteCheckpointHeights() external view returns (uint256[] memory) {
        return s.checkpointQuorumMap.incompleteQuorums.values();
    }

    /// @notice get the incomplete checkpoints
    function getIncompleteCheckpoints() external view returns (BottomUpCheckpoint[] memory) {
        uint256[] memory heights = s.checkpointQuorumMap.incompleteQuorums.values();
        uint256 size = heights.length;

        BottomUpCheckpoint[] memory checkpoints = new BottomUpCheckpoint[](size);
        for (uint64 i; i < size; ) {
            checkpoints[i] = s.bottomUpCheckpoints[uint64(heights[i])];
            unchecked {
                ++i;
            }
        }
        return checkpoints;
    }

    /// @notice get the incomplete batches of messages
    function getIncompleteMsgBatches() external view returns (BottomUpMsgBatch[] memory) {
        uint256[] memory heights = s.bottomUpMsgBatchQuorumMap.incompleteQuorums.values();
        uint256 size = heights.length;

        BottomUpMsgBatch[] memory batches = new BottomUpMsgBatch[](size);
        for (uint64 i; i < size; ) {
            batches[i] = s.bottomUpMsgBatches[uint64(heights[i])];
            unchecked {
                ++i;
            }
        }
        return batches;
    }

    /// @notice get the incomplete msd batches heights
    function getIncompleteMsgBatchHeights() external view returns (uint256[] memory) {
        return s.bottomUpMsgBatchQuorumMap.incompleteQuorums.values();
    }

    /// @notice get the bottom-up checkpoint retention index
    function getCheckpointRetentionHeight() external view returns (uint256) {
        return s.checkpointQuorumMap.retentionHeight;
    }

    /// @notice get the bottom-up batch retention index
    function getBottomUpMsgRetentionHeight() external view returns (uint256) {
        return s.bottomUpMsgBatchQuorumMap.retentionHeight;
    }

    /// @notice Calculate the threshold required for quorum in this subnet
    /// based on the configured majority percentage and the total weight of the validators.
    function getQuorumThreshold(uint256 totalWeight) external view returns (uint256) {
        return LibQuorum.weightNeeded(totalWeight, s.majorityPercentage);
    }

    /// @notice get the checkpoint signature bundle consisting of the checkpoint, its info, signatories and the corresponding signatures.
    function getCheckpointSignatureBundle(
        uint256 h
    )
        external
        view
        returns (
            BottomUpCheckpoint memory ch,
            QuorumInfo memory info,
            address[] memory signatories,
            bytes[] memory signatures
        )
    {
        ch = s.bottomUpCheckpoints[h];
        (info, signatories, signatures) = LibQuorum.getSignatureBundle(s.checkpointQuorumMap, h);

        return (ch, info, signatories, signatures);
    }

    /// @notice get the bottom-up msg batch signature bundle
    function getBottomUpMsgBatchSignatureBundle(
        uint256 h
    )
        external
        view
        returns (
            BottomUpMsgBatch memory batch,
            QuorumInfo memory info,
            address[] memory signatories,
            bytes[] memory signatures
        )
    {
        batch = s.bottomUpMsgBatches[h];
        (info, signatories, signatures) = LibQuorum.getSignatureBundle(s.bottomUpMsgBatchQuorumMap, h);

        return (batch, info, signatories, signatures);
    }

    /// @notice returns the current bottom-up checkpoint
    /// @return exists - whether the checkpoint exists
    /// @return epoch - the epoch of the checkpoint
    /// @return checkpoint - the checkpoint struct
    function getCurrentBottomUpCheckpoint()
        external
        view
        returns (bool exists, uint256 epoch, BottomUpCheckpoint memory checkpoint)
    {
        (exists, epoch, checkpoint) = LibGateway.getCurrentBottomUpCheckpoint();
        return (exists, epoch, checkpoint);
    }
}
