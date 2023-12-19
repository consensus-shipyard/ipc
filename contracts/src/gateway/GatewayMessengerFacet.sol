// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {BURNT_FUNDS_ACTOR} from "../constants/Constants.sol";
import {CrossMsg, StorableMsg} from "../structs/CrossNet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, SupplyKind} from "../structs/Subnet.sol";
import {InvalidCrossMsgFromSubnet, InvalidCrossMsgDstSubnet, CannotSendCrossMsgToItself, InvalidCrossMsgValue, MethodNotAllowed} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {StorableMsgHelper} from "../lib/StorableMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {SupplySourceHelper} from "../lib/SupplySourceHelper.sol";

string constant ERR_GENERAL_CROSS_MSG_DISABLED = "Support for general-purpose cross-net messages is disabled";
string constant ERR_MULTILEVEL_CROSS_MSG_DISABLED = "Support for multi-level cross-net messages is disabled";

contract GatewayMessengerFacet is GatewayActorModifiers {
    using FilAddress for address payable;
    using SubnetIDHelper for SubnetID;
    using StorableMsgHelper for StorableMsg;
    using SupplySourceHelper for address;

    /**
     * @dev sends a general-purpose cross-message from the local subnet to the destination subnet.
     *
     * IMPORTANT: `msg.value` is expected to equal to the value sent in `crossMsg.value` plus the cross-messaging fee.
     *
     * @param crossMsg - a cross-message to send
     */
    function sendUserXnetMessage(CrossMsg calldata crossMsg) external payable {
        if (!s.generalPurposeCrossMsg) {
            revert MethodNotAllowed(ERR_GENERAL_CROSS_MSG_DISABLED);
        }

        if (crossMsg.message.value != msg.value - crossMsg.message.fee) {
            revert InvalidCrossMsgValue();
        }

        // We disregard the "to" of the message that will be verified in the _commitCrossMessage().
        // The caller is the one set as the "from" of the message
        if (!crossMsg.message.from.subnetId.equals(s.networkName)) {
            revert InvalidCrossMsgFromSubnet();
        }

        // commit cross-message for propagation
        bool shouldBurn = _commitCrossMessage(crossMsg);

        _crossMsgSideEffects({v: crossMsg.message.value, shouldBurn: shouldBurn});
    }

    /**
     * @dev propagates the populated cross net message for the given cid
     * @param msgCid - the cid of the cross-net message
     */
    function propagate(bytes32 msgCid) external payable {
        if (!s.multiLevelCrossMsg) {
            revert MethodNotAllowed(ERR_MULTILEVEL_CROSS_MSG_DISABLED);
        }

        CrossMsg storage crossMsg = s.postbox[msgCid];
        validateFee(crossMsg.message.fee);

        bool shouldBurn = _commitCrossMessage(crossMsg);
        // We must delete the message first to prevent potential re-entrancies,
        // and as the message is deleted and we don't have a reference to the object
        // anymore, we need to pull the data from the message to trigger the side-effects.
        uint256 v = crossMsg.message.value;
        delete s.postbox[msgCid];

        _crossMsgSideEffects({v: v, shouldBurn: shouldBurn});

        uint256 feeRemainder = msg.value - s.minCrossMsgFee;

        // gas-opt: original check: feeRemainder > 0
        if (feeRemainder != 0) {
            payable(msg.sender).sendValue(feeRemainder);
        }
    }

    /**
     * @dev Commit the cross message to storage. It outputs a flag signaling
     * if the committed messages was bottom-up and some funds need to be
     * burnt.
     *
     * It also validates that destination subnet ID is not empty
     * and not equal to the current network.
     */
    function _commitCrossMessage(CrossMsg memory crossMessage) internal returns (bool shouldBurn) {
        SubnetID memory to = crossMessage.message.to.subnetId;
        if (to.isEmpty()) {
            revert InvalidCrossMsgDstSubnet();
        }
        // destination is the current network, you are better off with a good old message, no cross needed
        if (to.equals(s.networkName)) {
            revert CannotSendCrossMsgToItself();
        }

        SubnetID memory from = crossMessage.message.from.subnetId;
        IPCMsgType applyType = crossMessage.message.applyType(s.networkName);

        // Are we the LCA? (Lowest Common Ancestor)
        bool isLCA = to.commonParent(from).equals(s.networkName);

        // Even if multi-level messaging is enabled, we reject the xnet message
        // as soon as we learn that one of the networks involved use an ERC20 supply source.
        // This will block propagation on the first step, or the last step.
        //
        // TODO IPC does not implement fault handling yet, so if the message fails
        //  to propagate, the user won't be able to reclaim funds. That's one of the
        //  reasons xnet messages are disabled by default.

        bool reject = false;
        if (applyType == IPCMsgType.BottomUp) {
            // We're traversing up, so if we're the first hop, we reject if the subnet was ERC20.
            // If we're not the first hop, a child propagated this to us, they made a mistake and
            // and we don't have enough info to evaluate.
            reject = from.getParentSubnet().equals(s.networkName) && from.getActor().hasSupplyOfKind(SupplyKind.ERC20);
        } else if (applyType == IPCMsgType.TopDown) {
            // We're traversing down.
            // Check the next subnet (which can may be the destination subnet).
            reject = to.down(s.networkName).getActor().hasSupplyOfKind(SupplyKind.ERC20);
        }
        if (reject) {
            revert MethodNotAllowed("propagation not suppported for subnets with ERC20 supply");
        }

        // If the directionality is top-down, or if we're inverting the direction
        // because we're the LCA, commit a top-down message.
        if (applyType == IPCMsgType.TopDown || isLCA) {
            ++s.appliedTopDownNonce;
            LibGateway.commitTopDownMsg(crossMessage);
            return (shouldBurn = false);
        }

        // Else, commit a bottom up message.
        LibGateway.commitBottomUpMsg(crossMessage);
        // gas-opt: original check: value > 0
        return (shouldBurn = crossMessage.message.value != 0);
    }

    /**
     * @dev Performs transaction side-effects from the commitment of a cross-net message. Like
     * burning funds when bottom-up messages are propagated.
     *
     * @param v - the value of the committed cross-net message
     * @param shouldBurn - flag if the message should burn funds
     */
    function _crossMsgSideEffects(uint256 v, bool shouldBurn) internal {
        if (shouldBurn) {
            payable(BURNT_FUNDS_ACTOR).sendValue(v);
        }
    }
}
