// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {GatewayActorStorage, LibGatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {BURNT_FUNDS_ACTOR} from "../constants/Constants.sol";
import {SubnetID, Subnet, AssetKind, Asset} from "../structs/Subnet.sol";
import {SubnetActorGetterFacet} from "../subnet/SubnetActorGetterFacet.sol";
import {CallMsg, IpcMsgKind, IpcEnvelope, OutcomeType, BottomUpMsgBatch, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality} from "../structs/CrossNet.sol";
import {Membership} from "../structs/Subnet.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
// solhint-disable-next-line no-global-import
import "../errors/IPCErrors.sol";

library LibGateway {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using AssetHelper for address;
    using SubnetIDHelper for SubnetID;
    using FilAddress for address payable;
    using AssetHelper for Asset;
    using EnumerableSet for EnumerableSet.Bytes32Set;

    event MembershipUpdated(Membership);
    /// @dev subnet refers to the next "down" subnet that the `envelope.message.to` should be forwarded to.
    event NewTopDownMessage(address indexed subnet, IpcEnvelope message, bytes32 indexed id);
    /// @dev event emitted when there is a new bottom-up message added to the batch.
    /// @dev there is no need to emit the message itself, as the message is included in batch.
    event QueuedBottomUpMessage(bytes32 indexed id);
    /// @dev event emitted when there is a new bottom-up message batch to be signed.
    event NewBottomUpMsgBatch(uint256 indexed epoch);
    /// @dev event emmitted when a message is stored in the postbox - to be propagated further.
    event MessageStoredInPostbox(bytes32 indexed id);
    /// @dev event emmitted when a message is propagated further from the postbox.
    event MessagePropagatedFromPostbox(bytes32 id);

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
    ) internal view returns (bool exists, BottomUpCheckpoint storage checkpoint) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        checkpoint = s.bottomUpCheckpoints[epoch];
        exists = checkpoint.blockHeight != 0;
    }

    /// @notice returns the bottom-up batch
    function getBottomUpMsgBatch(uint256 epoch) internal view returns (bool exists, BottomUpMsgBatch storage batch) {
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
    function storeBottomUpCheckpoint(BottomUpCheckpoint memory checkpoint) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        BottomUpCheckpoint storage b = s.bottomUpCheckpoints[checkpoint.blockHeight];
        b.blockHash = checkpoint.blockHash;
        b.subnetID = checkpoint.subnetID;
        b.nextConfigurationNumber = checkpoint.nextConfigurationNumber;
        b.blockHeight = checkpoint.blockHeight;
        b.activity = checkpoint.activity;

        uint256 msgLength = checkpoint.msgs.length;
        for (uint256 i; i < msgLength; ) {
            // We need to push because initializing an array with a static
            // length will cause a copy from memory to storage, making
            // the compiler unhappy.
            b.msgs.push(checkpoint.msgs[i]);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice stores bottom-up batch
    function storeBottomUpMsgBatch(BottomUpMsgBatch memory batch) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        BottomUpMsgBatch storage b = s.bottomUpMsgBatches[batch.blockHeight];
        b.subnetID = batch.subnetID;
        b.blockHeight = batch.blockHeight;

        uint256 msgLength = batch.msgs.length;
        for (uint256 i; i < msgLength; ) {
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
        if (lastHeight >= finality.height) {
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
    function commitTopDownMsg(Subnet storage subnet, IpcEnvelope memory crossMessage) internal {
        uint64 topDownNonce = subnet.topDownNonce;

        crossMessage.localNonce = topDownNonce;
        // only set the original nonce if the message is from this subnet
        if (crossMessage.from.subnetId.equals(subnet.id)) {
            crossMessage.originalNonce = topDownNonce;
        }
        subnet.topDownNonce = topDownNonce + 1;
        subnet.circSupply += crossMessage.value;

        emit NewTopDownMessage({subnet: subnet.id.getAddress(), message: crossMessage, id: crossMessage.toTracingId()});
    }

    /// @notice Commits a new cross-net message to a message batch for execution
    /// @param crossMessage - the cross message to be committed
    function commitBottomUpMsg(IpcEnvelope memory crossMessage) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        uint256 epoch = getNextEpoch(block.number, s.bottomUpCheckPeriod);

        // assign nonce to the message.
        crossMessage.localNonce = s.bottomUpNonce;
        // only set the original nonce if the message is from this subnet
        if (crossMessage.from.subnetId.equals(s.networkName)) {
            crossMessage.originalNonce = s.bottomUpNonce;
        }
        s.bottomUpNonce += 1;

        // populate the batch for that epoch
        (bool exists, BottomUpMsgBatch storage batch) = LibGateway.getBottomUpMsgBatch(epoch);
        if (!exists) {
            batch.subnetID = s.networkName;
            batch.blockHeight = epoch;
            // we need to use push here to initialize the array.
            batch.msgs.push(crossMessage);
            emit QueuedBottomUpMessage({id: crossMessage.toTracingId()});
            return;
        }

        // if the maximum size was already achieved emit already the event
        // and re-assign the batch to the current epoch.
        if (batch.msgs.length == s.maxMsgsPerBottomUpBatch) {
            // copy the batch with max messages into the new cut.
            uint256 epochCut = block.number;
            BottomUpMsgBatch memory newBatch = BottomUpMsgBatch({
                subnetID: s.networkName,
                blockHeight: epochCut,
                msgs: new IpcEnvelope[](batch.msgs.length)
            });

            uint256 msgLength = batch.msgs.length;
            for (uint256 i; i < msgLength; ) {
                newBatch.msgs[i] = batch.msgs[i];
                unchecked {
                    ++i;
                }
            }

            // emit event with the next batch ready to sign quorum over.
            emit NewBottomUpMsgBatch(epochCut);

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

        emit QueuedBottomUpMessage({id: crossMessage.toTracingId()});
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
    function applyMessages(SubnetID memory arrivingFrom, IpcEnvelope[] memory crossMsgs) internal {
        uint256 crossMsgsLength = crossMsgs.length;
        for (uint256 i; i < crossMsgsLength; ) {
            applyMsg(arrivingFrom, crossMsgs[i]);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice applies a top down messages coming from parent. Reverts if a message is not top-down.
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsgs - the cross-net messages to apply
    function applyTopDownMessages(SubnetID memory arrivingFrom, IpcEnvelope[] memory crossMsgs) internal {
        uint256 crossMsgsLength = crossMsgs.length;
        for (uint256 i; i < crossMsgsLength; ) {
            applyMsg(arrivingFrom, crossMsgs[i], true);
            unchecked {
                ++i;
            }
        }
    }

    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// This function assumes that the relevant funds have been already minted or burnt
    /// when the top-down or bottom-up messages have been queued for execution.
    /// This function is not expected to revert. If a controlled failure happens, a new
    /// cross-message receipt is propagated for execution to inform the sending contract.
    /// `Call` cross-messages also trigger receipts if they are successful.
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsg - the cross message to be executed
    function applyMsg(SubnetID memory arrivingFrom, IpcEnvelope memory crossMsg) internal {
        applyMsg(arrivingFrom, crossMsg, false);
    }
    
    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// This function assumes that the relevant funds have been already minted or burnt
    /// when the top-down or bottom-up messages have been queued for execution.
    /// This function is not expected to revert. If a controlled failure happens, a new
    /// cross-message receipt is propagated for execution to inform the sending contract.
    /// `Call` cross-messages also trigger receipts if they are successful.
    /// @param arrivingFrom - the immediate subnet from which this message is arriving
    /// @param crossMsg - the cross message to be executed
    /// @param expectTopDownOnly - whether the message should be top-down only. Reverts if it is not.
    function applyMsg(SubnetID memory arrivingFrom, IpcEnvelope memory crossMsg, bool expectTopDownOnly) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        if (crossMsg.to.subnetId.isEmpty()) {
            sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.DstSubnet));
            return;
        }

        // The first thing we do is to find out the directionality of this message and act accordingly,
        // incrasing the applied nonces conveniently.
        // slither-disable-next-line uninitialized-local
        Asset memory supplySource;
        IPCMsgType applyType = crossMsg.applyType(s.networkName);
        // it's ok to revert here, as this is a programming error or a malicious message from validator.
        if (applyType == IPCMsgType.BottomUp) {
            if (expectTopDownOnly) {
                revert("Expecting top-down messages only");
            }

            // Load the subnet this message is coming from.
            // It will revert in case the subnet is not found - which in this case makes sense
            // This is because non existing child should not send messages.
            (, Subnet storage subnet) = LibGateway.getSubnet(arrivingFrom);

            if (subnet.appliedBottomUpNonce != crossMsg.localNonce) {
                sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Nonce));
                return;
            }
            subnet.appliedBottomUpNonce += 1;

            // The value carried in bottom-up messages needs to be treated according to the supply source
            // configuration of the subnet.
            supplySource = SubnetActorGetterFacet(subnet.id.getActor()).supplySource();
        } else if (applyType == IPCMsgType.TopDown) {
            // Note: there is no need to load the subnet, as a top-down application means that _we_ are the subnet.
            if (s.appliedTopDownNonce != crossMsg.localNonce) {
                sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Nonce));
                return;
            }
            s.appliedTopDownNonce += 1;

            // The value carried in top-down messages locally maps to the native coin, so we pass over the
            // native supply source.
            supplySource = AssetHelper.native();
        }

        // If the crossnet destination is NOT the current network (network where the gateway is running),
        // we add it to the postbox for further propagation.
        // Even if we send for propagation, the execution of every message
        // should increase the appliedNonce to allow the execution of the next message
        // of the batch (this is way we have this after the nonce logic).
        if (!crossMsg.to.subnetId.equals(s.networkName)) {
            (bool valid, InvalidXnetMessageReason reason) = validateCrossMessage(crossMsg);
            if (!valid) {
                sendReceipt(
                    crossMsg,
                    OutcomeType.SystemErr,
                    abi.encodeWithSelector(InvalidXnetMessage.selector, reason)
                );
                return;
            }

            bytes32 cid = crossMsg.toHash();
            s.postboxKeys.add(cid);
            s.postbox[cid] = crossMsg;

            emit MessageStoredInPostbox({id: crossMsg.toTracingId()});
            return;
        }

        // execute the message and get the receipt.
        (bool success, bytes memory ret) = executeCrossMsg(crossMsg, supplySource);
        if (success) {
            sendReceipt(crossMsg, OutcomeType.Ok, ret);
        } else {
            sendReceipt(crossMsg, OutcomeType.ActorErr, ret);
        }
    }

    /// @dev Execute the cross message using low level `call` method. This way ipc will
    ///      catch contract revert messages as well. We need this because in `CrossMsgHelper.execute`
    ///      there are `require` and `revert` calls, without reflexive call, the execution will
    ///      revert and block the checkpoint submission process.
    function executeCrossMsg(IpcEnvelope memory crossMsg, Asset memory supplySource) internal returns (bool success, bytes memory result) {
        (success, result) = address(CrossMsgHelper).delegatecall(   // solhint-disable-line avoid-low-level-calls
            abi.encodeWithSelector(CrossMsgHelper.execute.selector, crossMsg, supplySource)
        );

        if (success) {
            return abi.decode(result, (bool, bytes));
        }

        return (success, result);
    }

    /// @notice Sends a receipt from the execution of a cross-message.
    /// Only `Call` messages trigger a receipt. Transfer messages should be directly
    /// handled by the peer client to return the funds to the from address in the
    /// failing network.
    /// (we could optionally trigger a receipt from `Transfer`s to, but without
    /// multi-level execution it would be adding unnecessary overhead).
    function sendReceipt(IpcEnvelope memory original, OutcomeType outcomeType, bytes memory ret) internal {
        if (original.isEmpty()) {
            // This should not happen as previous validation should prevent empty messages arriving here.
            // If it does, we simply ignore.
            return;
        }

        // if we get a `Receipt` do nothing, no need to send receipts.
        // - And sending a `Receipt` to a `Receipt` could lead to amplification loops.
        if (original.kind == IpcMsgKind.Result) {
            return;
        }

        // commmit the receipt for propagation
        // slither-disable-next-line unused-return
        commitValidatedCrossMessage(original.createResultMsg(outcomeType, ret));
    }
    
    /**
     * @notice Commit the cross message to storage.
     *
     * @dev It does not make any validations. They are assumed to be done before calling this function.
     *  @param crossMessage The cross-network message to commit.
     *  @return shouldBurn A Boolean that indicates if the input amount should be burned.
     */
    function commitValidatedCrossMessage(IpcEnvelope memory crossMessage) internal returns (bool shouldBurn) {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        SubnetID memory to = crossMessage.to.subnetId;
        IPCMsgType applyType = crossMessage.applyType(s.networkName);
        bool isLCA = to.commonParent(crossMessage.from.subnetId).equals(s.networkName);

        // If the directionality is top-down, or if we're inverting the direction
        // because we're the LCA, commit a top-down message.
        if (applyType == IPCMsgType.TopDown || isLCA) {
            (, SubnetID memory subnetId) = to.down(s.networkName);
            (, Subnet storage subnet) = getSubnet(subnetId);
            LibGateway.commitTopDownMsg(subnet, crossMessage);
            return (shouldBurn = false);
        }

        // Else, commit a bottom up message.
        LibGateway.commitBottomUpMsg(crossMessage);
        // gas-opt: original check: value > 0
        return (shouldBurn = crossMessage.value != 0);
    }


    /**
     * @dev Performs transaction side-effects from the commitment of a cross-net message. Like
     * burning funds when bottom-up messages are propagated.
     *
     * @param v - the value of the committed cross-net message
     * @param shouldBurn - flag if the message should burn funds
     */
    function crossMsgSideEffects(uint256 v, bool shouldBurn) internal {
        if (shouldBurn) {
            payable(BURNT_FUNDS_ACTOR).sendValue(v);
        }
    }

    /// @notice Checks the length of a message batch, ensuring it is in (0, maxMsgsPerBottomUpBatch).
    /// @param msgs The batch of messages to check.
    function checkMsgLength(IpcEnvelope[] calldata msgs) internal view {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        if (msgs.length > s.maxMsgsPerBottomUpBatch) {
            revert MaxMsgsPerBatchExceeded();
        }
    }

    /// Checks if the incoming and outgoing subnet supply sources can be mapped.
    /// Caller should make sure the incoming/outgoing subnets and current subnet are immediate parent/child subnets.
    function checkSubnetsSupplyCompatible(
        bool isLCA,
        IPCMsgType applyType,
        SubnetID memory incoming,
        SubnetID memory outgoing,
        SubnetID memory current
    ) internal view returns(bool) {
        if (isLCA) {
            // now, it's pivoting @ LCA (i.e. upwards => downwards)
            // if incoming bottom up subnet and outgoing target subnet have the same 
            // asset, we will allow it. This is because if they are using the 
            // same asset, then the asset can be mapped in both subnets.
            
            (, SubnetID memory incDown) = incoming.down(current);
            (, SubnetID memory outDown) = outgoing.down(current);

            Asset memory incAsset = ISubnetActor(incDown.getActor()).supplySource();
            Asset memory outAsset = ISubnetActor(outDown.getActor()).supplySource();

            return incAsset.equals(outAsset);
        }
        
        if (applyType == IPCMsgType.BottomUp) {
            // The child subnet has supply source native, this is the same as 
            // the current subnet's native source, the mapping makes sense, propagate up.
            (, SubnetID memory incDown) = incoming.down(current);
            return incDown.getActor().hasSupplyOfKind(AssetKind.Native);
        }
        
        // Topdown handling

        // The incoming subnet's supply source will be mapped to native coin in the 
        // next child subnet. If the down subnet has native, then the mapping makes 
        // sense.
        (, SubnetID memory down) = outgoing.down(current);
        return down.getActor().hasSupplyOfKind(AssetKind.Native);
    }

    /// @notice Validates a cross message before committing it.
    function validateCrossMessage(IpcEnvelope memory envelope) internal view returns (bool, InvalidXnetMessageReason) {
        (bool valid, InvalidXnetMessageReason reason, ) = checkCrossMessage(envelope);
        return (valid, reason);
    }

    /// @notice Validates a cross message and returns the applyType if the message is valid
    function checkCrossMessage(IpcEnvelope memory envelope) internal view returns (bool valid, InvalidXnetMessageReason reason, IPCMsgType applyType) {
        SubnetID memory toSubnetId = envelope.to.subnetId;
        if (toSubnetId.isEmpty()) {
            return (false, InvalidXnetMessageReason.DstSubnet, applyType);
        }

        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        SubnetID memory currentNetwork = s.networkName;

        // We cannot send a cross message to the same subnet.
        if (toSubnetId.equals(currentNetwork)) {
            return (false, InvalidXnetMessageReason.ReflexiveSend, applyType);
        }

        // Lowest common ancestor subnet
        bool isLCA = toSubnetId.commonParent(envelope.from.subnetId).equals(currentNetwork);
        applyType = envelope.applyType(currentNetwork);

        // If the directionality is top-down, or if we're inverting the direction
        // else we need to check if the common parent exists.
        if (applyType == IPCMsgType.TopDown || isLCA) {
            (bool foundChildSubnetId, SubnetID memory childSubnetId) = toSubnetId.down(currentNetwork);
            if (!foundChildSubnetId) {
                return (false, InvalidXnetMessageReason.DstSubnet, applyType);
            }

            (bool foundSubnet,) = LibGateway.getSubnet(childSubnetId);
            if (!foundSubnet) {
                return (false, InvalidXnetMessageReason.DstSubnet, applyType);
            }
        } else {
            SubnetID memory commonParent = toSubnetId.commonParent(currentNetwork);
            if (commonParent.isEmpty()) {
                return (false, InvalidXnetMessageReason.NoRoute, applyType);
            }
        }

        // starting/ending subnet, no need check supply sources
        if (envelope.from.subnetId.equals(currentNetwork) || envelope.to.subnetId.equals(currentNetwork)) {
            return (true, reason, applyType);
        }

        bool supplySourcesCompatible = checkSubnetsSupplyCompatible({
            isLCA: isLCA,
            applyType: applyType, 
            incoming: envelope.from.subnetId,
            outgoing: envelope.to.subnetId,
            current: currentNetwork
        });

        if (!supplySourcesCompatible) {
            return (false, InvalidXnetMessageReason.IncompatibleSupplySource, applyType);
        }

        return (true, reason, applyType);
    }
    
     /**
     * @dev Propagates all the populated cross-net messages from the postbox.
     */
    function propagateAllPostboxMessages() internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

        uint256 keysLength = s.postboxKeys.length();

        bytes32[] memory values = s.postboxKeys.values();

        for (uint256 i = 0; i < keysLength; ) {
            bytes32 msgCid = values[i];
            LibGateway.propagatePostboxMessage(msgCid);

            unchecked {
                ++i;
            }
        }
    }

     /**
     * @dev Propagates the populated cross-net message for the given `msgCid`.
     * @param msgCid - the cid of the cross-net message
     */
    function propagatePostboxMessage(bytes32 msgCid) internal {
        GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
        IpcEnvelope storage crossMsg = s.postbox[msgCid];

        if (crossMsg.isEmpty()) {
            revert("Message not found in postbox");
        }

        bool shouldBurn = LibGateway.commitValidatedCrossMessage(crossMsg);

        // Cache value before deletion to avoid re-entrancy
        uint256 v = crossMsg.value;
        bytes32 deterministicId = crossMsg.toTracingId();

        // Remove the message to prevent re-entrancy and clean up state
        delete s.postbox[msgCid];
        s.postboxKeys.remove(msgCid);

        // Execute side effects
        LibGateway.crossMsgSideEffects({v: v, shouldBurn: shouldBurn});

        emit MessagePropagatedFromPostbox({id: deterministicId});
    }

}
