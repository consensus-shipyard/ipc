// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {GatewayActorStorage, LibGatewayActorStorage} from "../lib/LibGatewayActorStorage.sol";
import {BURNT_FUNDS_ACTOR} from "../constants/Constants.sol";
import {SubnetID, Subnet, SupplyKind, SupplySource} from "../structs/Subnet.sol";
import {SubnetActorGetterFacet} from "../subnet/SubnetActorGetterFacet.sol";
import {CallMsg, IpcMsgKind, IpcEnvelope, OutcomeType, BottomUpMsgBatch, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality} from "../structs/CrossNet.sol";
import {Membership} from "../structs/Subnet.sol";
import {CannotSendCrossMsgToItself, NotBottomUpMessage, L3NotSupportedYet, MethodNotAllowed, MaxMsgsPerBatchExceeded, InvalidXnetMessage ,OldConfigurationNumber, NotRegisteredSubnet, InvalidActorAddress, ParentFinalityAlreadyCommitted, InvalidXnetMessageReason} from "../errors/IPCErrors.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SupplySourceHelper} from "../lib/SupplySourceHelper.sol";
import {ISubnet} from "../interfaces/ISubnet.sol";
import {IMsgRouting} from "../interfaces/IMsgRouting.sol";
import {LibSubnetActor, LibSubnetActorQuery} from "../subnet/SubnetActorFacet.sol";

/// @notice The lib for bottom up message execution. Should be called from the Subnet Actor in the 
///         parent network.
library LibBottomUpExecution {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using SupplySourceHelper for address;
    using SubnetIDHelper for SubnetID;
    using FilAddress for address payable;
    using SupplySourceHelper for SupplySource;

    /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
    /// This function assumes that the relevant funds have been already minted or burnt
    /// when the top-down or bottom-up messages have been queued for execution.
    /// This function is not expected to revert. If a controlled failure happens, a new
    /// cross-message receipt is propagated for execution to inform the sending contract.
    /// `Call` cross-messages also trigger receipts if they are successful.
    /// @param crossMsg - the cross message to be executed
    function applyMsg(IpcEnvelope memory crossMsg) internal {
        if (crossMsg.to.subnetId.isEmpty()) {
            sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.DstSubnet));
            return;
        }

        SubnetID memory id = ISubnet(address(this)).id();

        if (crossMsg.applyType(id) != IPCMsgType.BottomUp) {
            revert NotBottomUpMessage();
        }

        // If the crossnet destination is NOT the current network (network where the gateway is running),
        // we add it to the postbox for further propagation.
        // Even if we send for propagation, the execution of every message
        // should increase the appliedNonce to allow the execution of the next message
        // of the batch (this is way we have this after the nonce logic).
        if (!crossMsg.to.subnetId.equals(id)) {
            revert NotBottomUpMessage();
        }

        if (LibSubnetActor.getThenIncrAppliedBottomUpNonce() != crossMsg.nonce) {
            sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Nonce));
            return;
        }

        // execute the message and get the receipt.
        (bool success, bytes memory ret) = executeCrossMsg(crossMsg, LibSubnetActorQuery.supplySource());
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
    function executeCrossMsg(IpcEnvelope memory crossMsg, SupplySource memory supplySource) internal returns (bool success, bytes memory result) {
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

        // the result of a bottom up message is a top down message
        LibSubnetActor.emitTopDownMsg(original.createResultMsg(outcomeType, ret));
    }
}

// library LibTopdownExecution {
//     using SubnetIDHelper for SubnetID;
//     using CrossMsgHelper for IpcEnvelope;
//     using SupplySourceHelper for address;
//     using SubnetIDHelper for SubnetID;
//     using FilAddress for address payable;
//     using SupplySourceHelper for SupplySource;

//     function msgExecutionStorage() internal returns (MsgExecutionStorage storage) {
//         bytes32 position = keccak256("ipc.msgExe.storage");
//         assembly {
//             ds.slot := position
//         }
//     }

//     /// @notice applies a cross-net messages coming from some other subnet.
//     /// The forwarder argument determines the previous subnet that submitted the checkpoint triggering the cross-net message execution.
//     /// @param arrivingFrom - the immediate subnet from which this message is arriving
//     /// @param crossMsgs - the cross-net messages to apply
//     function applyMessages(SubnetID memory arrivingFrom, IpcEnvelope[] memory crossMsgs) internal {
//         uint256 crossMsgsLength = crossMsgs.length;
//         for (uint256 i; i < crossMsgsLength; ) {
//             applyMsg(arrivingFrom, crossMsgs[i]);
//             unchecked {
//                 ++i;
//             }
//         }
//     }

//     /// @notice executes a cross message if its destination is the current network, otherwise adds it to the postbox to be propagated further
//     /// This function assumes that the relevant funds have been already minted or burnt
//     /// when the top-down or bottom-up messages have been queued for execution.
//     /// This function is not expected to revert. If a controlled failure happens, a new
//     /// cross-message receipt is propagated for execution to inform the sending contract.
//     /// `Call` cross-messages also trigger receipts if they are successful.
//     /// @param arrivingFrom - the immediate subnet from which this message is arriving
//     /// @param crossMsg - the cross message to be executed
//     function applyMsg(SubnetID memory arrivingFrom, IpcEnvelope memory crossMsg) internal {
//         if (crossMsg.to.subnetId.isEmpty()) {
//             sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.DstSubnet));
//             return;
//         }

//         GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

//         // The first thing we do is to find out the directionality of this message and act accordingly,
//         // incrasing the applied nonces conveniently.
//         // slither-disable-next-line uninitialized-local
//         SupplySource memory supplySource;
//         IPCMsgType applyType = crossMsg.applyType(s.networkName);
//         if (applyType == IPCMsgType.BottomUp) {
//             // Load the subnet this message is coming from. Ensure that it exists and that the nonce expectation is met.
//             (bool registered, Subnet storage subnet) = LibGateway.getSubnet(arrivingFrom);
//             if (!registered) {
//                 // this means the subnet that sent the bottom up message is not registered,
//                 // we cannot send the receipt back as top down because the subnet is not registered
//                 // we ignore this message for as it's not valid, and it may be someone trying to forge it.
//                 return;
//             }
//             if (subnet.appliedBottomUpNonce != crossMsg.nonce) {
//                 sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Nonce));
//                 return;
//             }
//             subnet.appliedBottomUpNonce += 1;

//             // The value carried in bottom-up messages needs to be treated according to the supply source
//             // configuration of the subnet.
//             supplySource = SubnetActorGetterFacet(subnet.id.getActor()).supplySource();
//         } else if (applyType == IPCMsgType.TopDown) {
//             // Note: there is no need to load the subnet, as a top-down application means that _we_ are the subnet.
//             if (s.appliedTopDownNonce != crossMsg.nonce) {
//                 sendReceipt(crossMsg, OutcomeType.SystemErr, abi.encodeWithSelector(InvalidXnetMessage.selector, InvalidXnetMessageReason.Nonce));
//                 return;
//             }
//             s.appliedTopDownNonce += 1;

//             // The value carried in top-down messages locally maps to the native coin, so we pass over the
//             // native supply source.
//             supplySource = SupplySourceHelper.native();
//         }

//         // If the crossnet destination is NOT the current network (network where the gateway is running),
//         // we add it to the postbox for further propagation.
//         // Even if we send for propagation, the execution of every message
//         // should increase the appliedNonce to allow the execution of the next message
//         // of the batch (this is way we have this after the nonce logic).
//         if (!crossMsg.to.subnetId.equals(s.networkName)) {
//             bytes32 cid = crossMsg.toHash();
//             s.postbox[cid] = crossMsg;
//             return;
//         }

//         // execute the message and get the receipt.
//         (bool success, bytes memory ret) = executeCrossMsg(crossMsg, supplySource);
//         if (success) {
//             sendReceipt(crossMsg, OutcomeType.Ok, ret);
//         } else {
//             sendReceipt(crossMsg, OutcomeType.ActorErr, ret);
//         }
//     }

//     /// @dev Execute the cross message using low level `call` method. This way ipc will
//     ///      catch contract revert messages as well. We need this because in `CrossMsgHelper.execute`
//     ///      there are `require` and `revert` calls, without reflexive call, the execution will
//     ///      revert and block the checkpoint submission process.
//     function executeCrossMsg(IpcEnvelope memory crossMsg, SupplySource memory supplySource) internal returns (bool success, bytes memory result) {
//         (success, result) = address(CrossMsgHelper).delegatecall(   // solhint-disable-line avoid-low-level-calls
//             abi.encodeWithSelector(CrossMsgHelper.execute.selector, crossMsg, supplySource)
//         );

//         if (success) {
//             return abi.decode(result, (bool, bytes));
//         }

//         return (success, result);
//     }

//     /// @notice Sends a receipt from the execution of a cross-message.
//     /// Only `Call` messages trigger a receipt. Transfer messages should be directly
//     /// handled by the peer client to return the funds to the from address in the
//     /// failing network.
//     /// (we could optionally trigger a receipt from `Transfer`s to, but without
//     /// multi-level execution it would be adding unnecessary overhead).
//     function sendReceipt(IpcEnvelope memory original, OutcomeType outcomeType, bytes memory ret) internal {
//         if (original.isEmpty()) {
//             // This should not happen as previous validation should prevent empty messages arriving here.
//             // If it does, we simply ignore.
//             return;
//         }

//         // if we get a `Receipt` do nothing, no need to send receipts.
//         // - And sending a `Receipt` to a `Receipt` could lead to amplification loops.
//         if (original.kind == IpcMsgKind.Result) {
//             return;
//         }

//         // commmit the receipt for propagation
//         // slither-disable-next-line unused-return
//         commitCrossMessage(original.createResultMsg(outcomeType, ret));
//     }

//     /**
//      * @notice Commit the cross message to storage.
//      *
//      * @dev It also validates that destination subnet ID is not empty
//      *      and not equal to the current network.
//      *      This function assumes that the funds inside `value` have been
//      *      conveniently minted or burnt already and the message is free to
//      *      use them (see execBottomUpMsgBatch for reference).
//      *  @param crossMessage The cross-network message to commit.
//      *  @return shouldBurn A Boolean that indicates if the input amount should be burned.
//      */
//     function commitCrossMessage(IpcEnvelope memory crossMessage) internal returns (bool shouldBurn) {
//         GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();
//         SubnetID memory to = crossMessage.to.subnetId;
//         if (to.isEmpty()) {
//             revert InvalidXnetMessage(InvalidXnetMessageReason.DstSubnet);
//         }
//         // destination is the current network, you are better off with a good old message, no cross needed
//         if (to.equals(s.networkName)) {
//             revert CannotSendCrossMsgToItself();
//         }

//         SubnetID memory from = crossMessage.from.subnetId;
//         IPCMsgType applyType = crossMessage.applyType(s.networkName);

//         // Are we the LCA? (Lowest Common Ancestor)
//         bool isLCA = to.commonParent(from).equals(s.networkName);

//         // Even if multi-level messaging is enabled, we reject the xnet message
//         // as soon as we learn that one of the networks involved use an ERC20 supply source.
//         // This will block propagation on the first step, or the last step.
//         //
//         // TODO IPC does not implement fault handling yet, so if the message fails
//         //  to propagate, the user won't be able to reclaim funds. That's one of the
//         //  reasons xnet messages are disabled by default.

//         bool reject = false;
//         if (applyType == IPCMsgType.BottomUp) {
//             // We're traversing up, so if we're the first hop, we reject if the subnet was ERC20.
//             // If we're not the first hop, a child propagated this to us, they made a mistake and
//             // and we don't have enough info to evaluate.
//             reject = from.getParentSubnet().equals(s.networkName) && from.getActor().hasSupplyOfKind(SupplyKind.ERC20);
//         } else if (applyType == IPCMsgType.TopDown) {
//             // We're traversing down.
//             // Check the next subnet (which can may be the destination subnet).
//             reject = to.down(s.networkName).getActor().hasSupplyOfKind(SupplyKind.ERC20);
//         }
//         if (reject) {
//             if (crossMessage.kind == IpcMsgKind.Transfer) {
//                 revert MethodNotAllowed("propagation of `Transfer` messages not suppported for subnets with ERC20 supply");
//             }
//         }

//         // If the directionality is top-down, or if we're inverting the direction
//         // because we're the LCA, commit a top-down message.
//         if (applyType == IPCMsgType.TopDown || isLCA) {
//             ++s.appliedTopDownNonce;
//             LibGateway.commitTopDownMsg(crossMessage);
//             return (shouldBurn = false);
//         }

//         // Else, commit a bottom up message.
//         LibGateway.commitBottomUpMsg(crossMessage);
//         // gas-opt: original check: value > 0
//         return (shouldBurn = crossMessage.value != 0);
//     }

//     /**
//      * @dev Performs transaction side-effects from the commitment of a cross-net message. Like
//      * burning funds when bottom-up messages are propagated.
//      *
//      * @param v - the value of the committed cross-net message
//      * @param shouldBurn - flag if the message should burn funds
//      */
//     function crossMsgSideEffects(uint256 v, bool shouldBurn) internal {
//         if (shouldBurn) {
//             payable(BURNT_FUNDS_ACTOR).sendValue(v);
//         }
//     }

//     /// @notice Checks the length of a message batch, ensuring it is in (0, maxMsgsPerBottomUpBatch).
//     /// @param msgs The batch of messages to check.
//     function checkMsgLength(IpcEnvelope[] calldata msgs) internal view {
//         GatewayActorStorage storage s = LibGatewayActorStorage.appStorage();

//         if (msgs.length > s.maxMsgsPerBottomUpBatch) {
//             revert MaxMsgsPerBatchExceeded();
//         }
//     }
// }
