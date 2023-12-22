// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {GatewayActorStorage, LibGatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {SubnetID, Subnet, SupplySource} from "../structs/Subnet.sol";
import {SubnetActorGetterFacet} from "../subnet/SubnetActorGetterFacet.sol";
import {CrossMsg, StorableMsg, BottomUpMsgBatch, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality} from "../structs/CrossNet.sol";
import {Membership} from "../structs/Subnet.sol";
import {MaxMsgsPerBatchExceeded, BatchWithNoMessages, InvalidCrossMsgNonce, InvalidCrossMsgDstSubnet, OldConfigurationNumber, NotRegisteredSubnet, InvalidActorAddress, ParentFinalityAlreadyCommitted} from "../errors/IPCErrors.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SupplySourceHelper} from "../lib/SupplySourceHelper.sol";
import {StorableMsgHelper} from "../lib/StorableMsgHelper.sol";

library LibGateway {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using SupplySourceHelper for SupplySource;
    using StorableMsgHelper for StorableMsg;

    event MembershipUpdated(Membership);
    /// @dev subnet refers to the next "down" subnet that the `CrossMsg.message.to` should be forwarded to.
    event NewTopDownMessage(address indexed subnet, CrossMsg message);
    /// @dev event emitted when there is a new bottom-up message batch to be signed.
    event NewBottomUpMsgBatch(uint256 indexed epoch, BottomUpMsgBatch batch);

    /// @notice returns the current bottom-up checkpoint
    /// @return exists - whether the checkpoint exists
    /// @return epoch - the epoch of the checkpoint
    /// @return checkpoint - the checkpoint struct
    function getCurrentBottomUpCheckpoint()
        internal
        view
        returns (bool exists, uint256 epoch, BottomUpCheckpoint memory checkpoint)
    {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        epoch = LibGateway.getNextEpoch(block.number, s.bottomUpCheckPeriod);
        checkpoint = s.bottomUpCheckpoints[epoch];
        exists = !checkpoint.subnetID.isEmpty();
    }

    /// @notice returns the bottom-up checkpoint
    function getBottomUpCheckpoint(
        uint256 epoch
    )
        internal
        view
        returns (bool exists, BottomUpCheckpoint storage checkpoint)
    {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        checkpoint = s.bottomUpCheckpoints[epoch];
        exists = checkpoint.blockHeight != 0;
    }

    /// @notice returns the bottom-up batch
    function getBottomUpMsgBatch(
        uint256 epoch
    )
        internal
        view
        returns (bool exists, BottomUpMsgBatch storage batch)
    {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        batch = s.bottomUpMsgBatches[epoch];
        exists = batch.blockHeight != 0;
    }

    /// @notice checks if the bottom-up checkpoint already exists at the target epoch
    function bottomUpCheckpointExists(uint256 epoch) internal view returns (bool) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        return s.bottomUpCheckpoints[epoch].blockHeight != 0;
    }

    /// @notice checks if the bottom-up checkpoint already exists at the target epoch
    function bottomUpBatchMsgsExists(uint256 epoch) internal view returns (bool) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        return s.bottomUpMsgBatches[epoch].blockHeight != 0;
    }

    /// @notice stores checkpoint
    function storeBottomUpCheckpoint(
        BottomUpCheckpoint memory checkpoint
    ) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        s.bottomUpCheckpoints[checkpoint.blockHeight] = checkpoint;
    }

    /// @notice stores bottom-up batch
    function storeBottomUpMsgBatch(
        BottomUpMsgBatch memory batch
    ) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        BottomUpMsgBatch storage b = s.bottomUpMsgBatches[batch.blockHeight];
        b.subnetID = batch.subnetID;
        b.blockHeight = batch.blockHeight;

        uint256 msgLength = batch.msgs.length;
        for (uint256 i; i < msgLength;) {
            // We need to push because initializing an array with a static
            // length will cause a copy from memory to storage, making
            // the compiler unhappy.
            b.msgs.push(batch.msgs[i]);
            unchecked {
                ++i;
            }
        }
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

    /// @notice Commits a new cross-net message to a message batch for execution
    /// @param crossMessage - the cross message to be committed
    function commitBottomUpMsg(CrossMsg memory crossMessage) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        uint256 epoch = getNextEpoch(block.number, s.bottomUpMsgBatchPeriod);

        // assign nonce to the message.
        crossMessage.message.nonce = s.bottomUpNonce;
        s.bottomUpNonce += 1;

        // populate the batch for that epoch
        (bool exists, BottomUpMsgBatch storage batch) = LibGateway.getBottomUpMsgBatch(epoch);
        if (!exists) {
            batch.subnetID = s.networkName;
            batch.blockHeight = epoch;
            // we need to use push here to initialize the array.
            batch.msgs.push(crossMessage);
        } else {
            // if the maximum size was already achieved emit already the event
            // and re-assign the batch to the current epoch.
            if (batch.msgs.length == s.maxMsgsPerBottomUpBatch){
                // copy the batch with max messages into the new cut.
                uint256 epochCut = block.number;
                BottomUpMsgBatch memory newBatch = BottomUpMsgBatch({
                    subnetID: s.networkName,
                    blockHeight: epochCut,
                    msgs: new CrossMsg[](batch.msgs.length)
                });
                uint256 msgLength = batch.msgs.length;
                for (uint256 i; i < msgLength;) {
                    newBatch.msgs[i] = batch.msgs[i];
                    unchecked {
                        ++i;
                    }
                }
                // emit event with the next batch ready to sign quorum over.
                emit NewBottomUpMsgBatch(epochCut,newBatch);

                // Empty the messages of existing batch with epoch and start populating with the new message.
                delete batch.msgs;
                // need to push here to avoid a copy from memory to storage
                batch.msgs.push(crossMessage);

                LibGateway.storeBottomUpMsgBatch(newBatch);
            } else {
                // we append the new message normally, and wait for the batch period
                // to trigger the cutting of the batch.
                batch.msgs.push(crossMessage);
            }
        }
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

    /// @notice method that gives the epoch for a given block number and checkpoint period
    /// @return epoch - the epoch for the given block number and checkpoint period
    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    /// @notice applies a cross-net messages coming from some other subnet.
    /// The forwarder argument determines the previous subnet that submitted the checkpoint triggering the cross-net message execution.
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsgs - the cross-net messages to apply
    function applyMessages(SubnetID memory arrivingFrom, CrossMsg[] memory crossMsgs) internal {
        uint256 crossMsgsLength = crossMsgs.length;
        for (uint256 i; i < crossMsgsLength; ) {
            applyMsg(arrivingFrom, crossMsgs[i]);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsg - the cross message to be executed
    function applyMsg(SubnetID memory arrivingFrom, CrossMsg memory crossMsg) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        if (crossMsg.message.to.subnetId.isEmpty()) {
            revert InvalidCrossMsgDstSubnet();
        }

        // If the crossnet destination is NOT the current network (network where the gateway is running),
        // we add it to the postbox for further propagation.
        if (!crossMsg.message.to.subnetId.equals(s.networkName)) {
            bytes32 cid = crossMsg.toHash();
            s.postbox[cid] = crossMsg;
            return;
        }

        // Now, let's find out the directionality of this message and act accordingly.
        // slither-disable-next-line uninitialized-local
        SupplySource memory supplySource;
        IPCMsgType applyType = crossMsg.message.applyType(s.networkName);
        if (applyType == IPCMsgType.BottomUp) {
            // Load the subnet this message is coming from. Ensure that it exists and that the nonce expectation is met.
            (bool registered, Subnet storage subnet) = LibGateway.getSubnet(arrivingFrom);
            if (!registered) {
                revert NotRegisteredSubnet();
            }
            if (subnet.appliedBottomUpNonce != crossMsg.message.nonce) {
                revert InvalidCrossMsgNonce();
            }
            subnet.appliedBottomUpNonce += 1;

            // The value carried in bottom-up messages needs to be treated according to the supply source
            // configuration of the subnet.
            supplySource = SubnetActorGetterFacet(subnet.id.getActor()).supplySource();
        } else if (applyType == IPCMsgType.TopDown) {
            // Note: there is no need to load the subnet, as a top-down application means that _we_ are the subnet.
            if (s.appliedTopDownNonce != crossMsg.message.nonce) {
                revert InvalidCrossMsgNonce();
            }
            s.appliedTopDownNonce += 1;

            // The value carried in top-down messages locally maps to the native coin, so we pass over the
            // native supply source.
            supplySource = SupplySourceHelper.native();
        }

        // slither-disable-next-line unused-return
        crossMsg.execute(supplySource);
    }

    /// @notice Checks the length of a message batch, ensuring it is in (0, maxMsgsPerBottomUpBatch).
    /// @param batch The batch of messages to check.
    function checkMsgLength(BottomUpMsgBatch memory batch) internal view {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        if (batch.msgs.length > s.maxMsgsPerBottomUpBatch) {
            revert MaxMsgsPerBatchExceeded();
        }
        if (batch.msgs.length == 0) {
            revert BatchWithNoMessages();
        }
    }
}
