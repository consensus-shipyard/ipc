// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {GatewayActorStorage, LibGatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {SubnetID, Subnet} from "../structs/Subnet.sol";
import {CrossMsg, BottomUpCheckpoint, ParentFinality, CheckpointInfo} from "../structs/Checkpoint.sol";
import {Membership} from "../structs/Subnet.sol";
import {OldConfigurationNumber, NotRegisteredSubnet, InvalidActorAddress, ParentFinalityAlreadyCommitted} from "../errors/IPCErrors.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";

library LibGateway {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;

    event MembershipUpdated(Membership);
    /// @dev subnet refers to the next "down" subnet that the `CrossMsg.message.to` should be forwarded to.
    event NewTopDownMessage(address indexed subnet, CrossMsg message);

    /// @notice returns the current bottom-up checkpoint
    /// @return exists - whether the checkpoint exists
    /// @return epoch - the epoch of the checkpoint
    /// @return checkpoint - the checkpoint struct
    function getCurrentBottomUpCheckpoint()
        internal
        view
        returns (bool exists, uint64 epoch, BottomUpCheckpoint memory checkpoint)
    {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        epoch = LibGateway.getNextEpoch(block.number, s.bottomUpCheckPeriod);
        checkpoint = s.bottomUpCheckpoints[epoch];
        exists = !checkpoint.subnetID.isEmpty();
    }

    /// @notice returns the bottom-up checkpoint and its info at the target epoch
    function getBottomUpCheckpointWithInfo(
        uint64 epoch
    )
        internal
        view
        returns (bool exists, BottomUpCheckpoint storage checkpoint, CheckpointInfo storage checkpointInfo)
    {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        checkpoint = s.bottomUpCheckpoints[epoch];
        checkpointInfo = s.bottomUpCheckpointInfo[epoch];

        exists = checkpoint.blockHeight != 0;
    }

    /// @notice checks if the bottom-up checkpoint already exists at the target epoch
    function bottomUpCheckpointExists(uint64 epoch) internal view returns (bool) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        return s.bottomUpCheckpoints[epoch].blockHeight != 0;
    }

    /// @notice stores checkpoint and its info to storage.
    function storeBottomUpCheckpointWithInfo(
        BottomUpCheckpoint memory checkpoint,
        CheckpointInfo memory checkpointInfo
    ) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        s.bottomUpCheckpoints[checkpoint.blockHeight] = checkpoint;
        s.bottomUpCheckpointInfo[checkpoint.blockHeight] = checkpointInfo;
    }

    /// @notice obtain the ipc parent finality at certain block number
    /// @param blockNumber - the block number to obtain the finality
    function getParentFinality(uint256 blockNumber) internal view returns (ParentFinality memory) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        return s.finalitiesMap[blockNumber];
    }

    /// @notice obtain the latest committed ipc parent finality
    function getLatestParentFinality() internal view returns (ParentFinality memory) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        return getParentFinality(s.latestParentHeight);
    }

    /// @notice commit the ipc parent finality into storage
    /// @param finality - the finality to be committed
    function commitParentFinality(
        ParentFinality calldata finality
    ) internal returns (ParentFinality memory lastFinality) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        uint256 lastHeight = s.latestParentHeight;
        if (lastHeight > finality.height) {
            revert ParentFinalityAlreadyCommitted();
        }
        lastFinality = s.finalitiesMap[lastHeight];

        s.finalitiesMap[finality.height] = finality;
        s.latestParentHeight = finality.height;
    }

    /// @notice set the next membership
    /// @param membership - new membership
    function updateMembership(Membership memory membership) internal {
        emit MembershipUpdated(membership);

        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        // perform checks after the genesis membership
        if (s.currentMembership.configurationNumber != 0) {
            if (membership.configurationNumber == s.lastMembership.configurationNumber) {
                return;
            }
            // We reject messages with configuration numbers from the past and revert the call.
            if (membership.configurationNumber < s.lastMembership.configurationNumber) {
                revert OldConfigurationNumber();
            }

            // Check if the membership is equal and return if it is the case
            if (membershipEqual(membership, s.currentMembership)) {
                return;
            }
        }

        s.lastMembership = s.currentMembership;

        uint256 inputLength = membership.validators.length;
        uint256 storeLength = s.currentMembership.validators.length;
        // memory arrays can't be copied directly from memory into storage,
        // we need to explicitly increase the size of the array in storage.
        for (uint256 i; i < inputLength; ) {
            if (i < storeLength) {
                s.currentMembership.validators[i] = membership.validators[i];
            } else {
                s.currentMembership.validators.push(membership.validators[i]);
            }
            unchecked {
                ++i;
            }
        }
        s.currentMembership.configurationNumber = membership.configurationNumber;
        // finally we need to remove any outstanding membership from
        // storage.
        if (storeLength > inputLength) {
            for (uint256 i = inputLength; i < storeLength; ) {
                s.currentMembership.validators.pop();
                unchecked {
                    ++i;
                }
            }
        }
    }

    /// @dev - Computes total weight for a specific membership
    function membershipTotalWeight(Membership memory self) internal pure returns (uint256) {
        uint256 len = self.validators.length;
        uint256 totalValidatorsWeight;
        for (uint256 i; i < len; ) {
            totalValidatorsWeight += self.validators[i].weight;
            unchecked {
                ++i;
            }
        }
        return totalValidatorsWeight;
    }

    /// @dev compares two memberships and returns true if they are equal
    function membershipEqual(Membership memory mb1, Membership memory mb2) internal pure returns (bool) {
        if (mb1.configurationNumber != mb2.configurationNumber) {
            return false;
        }
        if (membershipTotalWeight(mb1) != membershipTotalWeight(mb2)) {
            return false;
        }
        if (mb1.validators.length != mb2.validators.length) {
            return false;
        }
        bytes32 h1 = keccak256(abi.encode(mb1.validators));
        bytes32 h2 = keccak256(abi.encode(mb2.validators));

        return h1 == h2;
    }

    /// @notice commit topdown messages for their execution in the subnet. Adds the message to the subnet struct for future execution
    /// @param crossMessage - the cross message to be committed
    function commitTopDownMsg(CrossMsg memory crossMessage) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        SubnetID memory subnetId = crossMessage.message.to.subnetId.down(s.networkName);

        (bool registered, Subnet storage subnet) = getSubnet(subnetId);

        if (!registered) {
            revert NotRegisteredSubnet();
        }

        uint64 topDownNonce = subnet.topDownNonce;

        crossMessage.message.nonce = topDownNonce;
        subnet.topDownNonce = topDownNonce + 1;
        subnet.circSupply += crossMessage.message.value;

        emit NewTopDownMessage({subnet: subnetId.getAddress(), message: crossMessage});
    }

    /// @notice commit bottom-up messages for their execution in the subnet. Adds the message to the checkpoint for future execution
    /// @param crossMessage - the cross message to be committed
    function commitBottomUpMsg(CrossMsg memory crossMessage) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        uint64 epoch = getNextEpoch(block.number, s.bottomUpCheckPeriod);

        crossMessage.message.nonce = s.bottomUpNonce;

        s.bottomUpMessages[epoch].push(crossMessage);
        s.bottomUpNonce += 1;
    }

    /// @notice returns the subnet created by a validator
    /// @param actor the validator that created the subnet
    /// @return found whether the subnet exists
    /// @return subnet -  the subnet struct
    function getSubnet(address actor) internal view returns (bool found, Subnet storage subnet) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        if (actor == address(0)) {
            revert InvalidActorAddress();
        }
        SubnetID memory subnetId = s.networkName.createSubnetId(actor);

        return getSubnet(subnetId);
    }

    /// @notice returns the subnet with the given id
    /// @param subnetId the id of the subnet
    /// @return found whether the subnet exists
    /// @return subnet -  the subnet struct
    function getSubnet(SubnetID memory subnetId) internal view returns (bool found, Subnet storage subnet) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        subnet = s.subnets[subnetId.toHash()];
        found = !subnet.id.isEmpty();
    }

    /// @notice returns the needed weight value corresponding to the majority percentage
    /// @dev `majorityPercentage` must be a valid number
    function weightNeeded(uint256 weight, uint256 majorityPercentage) internal pure returns (uint256) {
        return (weight * majorityPercentage) / 100;
    }

    /// @notice method that gives the epoch for a given block number and checkpoint period
    /// @return epoch - the epoch for the given block number and checkpoint period
    function getNextEpoch(uint256 blockNumber, uint64 checkPeriod) internal pure returns (uint64) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }
}
