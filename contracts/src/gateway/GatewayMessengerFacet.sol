// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {IpcEnvelope, IpcMsg} from "../structs/CrossNet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, SupplyKind} from "../structs/Subnet.sol";
import {InvalidCrossMsgFromSubnet, InvalidCrossMsgSender, InvalidCrossMsgDstSubnet, CannotSendCrossMsgToItself, InvalidCrossMsgValue, MethodNotAllowed} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {SupplySourceHelper} from "../lib/SupplySourceHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";

string constant ERR_GENERAL_CROSS_MSG_DISABLED = "Support for general-purpose cross-net messages is disabled";
string constant ERR_MULTILEVEL_CROSS_MSG_DISABLED = "Support for multi-level cross-net messages is disabled";

contract GatewayMessengerFacet is GatewayActorModifiers {
    using FilAddress for address payable;
    using SubnetIDHelper for SubnetID;

    /**
     * @dev sends a general-purpose cross-message from the local subnet to the destination subnet.
     *
     * IMPORTANT: `msg.value` is expected to equal to the value sent in `crossMsg.value` plus the cross-messaging fee.
     * Only smart contracts are allowed to trigger these cross-net messages, users
     * can always send funds from their address to the destination subnet and then run the transaction in the destination
     * normally.
     *
     * @param crossMsg - a cross-message to send.
     */
    function sendContractXnetMessage(IpcEnvelope calldata crossMsg) external payable {
        if (!s.generalPurposeCrossMsg) {
            revert MethodNotAllowed(ERR_GENERAL_CROSS_MSG_DISABLED);
        }

        // we prevent the sender from being an EoA.
        if (!(msg.sender.code.length > 0)) {
            revert InvalidCrossMsgSender();
        }

        if (crossMsg.value != msg.value) {
            revert InvalidCrossMsgValue();
        }

        // We disregard the "to" of the message that will be verified in the _commitCrossMessage().
        // The caller is the one set as the "from" of the message
        if (!crossMsg.from.subnetId.equals(s.networkName)) {
            revert InvalidCrossMsgFromSubnet();
        }

        // commit cross-message for propagation
        bool shouldBurn = LibGateway.commitCrossMessage(crossMsg);

        LibGateway.crossMsgSideEffects({v: crossMsg.value, shouldBurn: shouldBurn});
    }

    /**
     * @dev propagates the populated cross net message for the given cid
     * @param msgCid - the cid of the cross-net message
     */
    function propagate(bytes32 msgCid) external payable {
        if (!s.multiLevelCrossMsg) {
            revert MethodNotAllowed(ERR_MULTILEVEL_CROSS_MSG_DISABLED);
        }

        IpcEnvelope storage crossMsg = s.postbox[msgCid];

        bool shouldBurn = LibGateway.commitCrossMessage(crossMsg);
        // We must delete the message first to prevent potential re-entrancies,
        // and as the message is deleted and we don't have a reference to the object
        // anymore, we need to pull the data from the message to trigger the side-effects.
        uint256 v = crossMsg.value;
        delete s.postbox[msgCid];

        LibGateway.crossMsgSideEffects({v: v, shouldBurn: shouldBurn});
    }
}
