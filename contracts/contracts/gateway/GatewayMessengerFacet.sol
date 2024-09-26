// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {IpcEnvelope, CallMsg, IpcMsgKind} from "../structs/CrossNet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {SubnetID, AssetKind, IPCAddress} from "../structs/Subnet.sol";
import {InvalidXnetMessage, InvalidXnetMessageReason, CannotSendCrossMsgToItself, MethodNotAllowed} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

string constant ERR_GENERAL_CROSS_MSG_DISABLED = "Support for general-purpose cross-net messages is disabled";
string constant ERR_MULTILEVEL_CROSS_MSG_DISABLED = "Support for multi-level cross-net messages is disabled";

contract GatewayMessengerFacet is GatewayActorModifiers {
    using FilAddress for address payable;
    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.Bytes32Set;

    /**
     * @dev Sends a general-purpose cross-message from the local subnet to the destination subnet.
     * Any value in msg.value will be forwarded in the call.
     *
     * IMPORTANT: Only smart contracts are allowed to trigger these cross-net messages. User wallets can send funds
     * from their address to the destination subnet and then run the transaction in the destination normally.
     *
     * @param envelope - the original envelope, which will be validated, stamped and committed during the send.
     * @return committed envelope.
     */
    function sendContractXnetMessage(
        IpcEnvelope calldata envelope
    ) external payable returns (IpcEnvelope memory committed) {
        if (!s.generalPurposeCrossMsg) {
            revert MethodNotAllowed(ERR_GENERAL_CROSS_MSG_DISABLED);
        }

        // We prevent the sender from being an EoA.
        if (!(msg.sender.code.length > 0)) {
            revert InvalidXnetMessage(InvalidXnetMessageReason.Sender);
        }

        if (envelope.value != msg.value) {
            revert InvalidXnetMessage(InvalidXnetMessageReason.Value);
        }

        if (envelope.kind != IpcMsgKind.Call) {
            revert InvalidXnetMessage(InvalidXnetMessageReason.Kind);
        }

        // Will revert if the message won't deserialize into a CallMsg.
        abi.decode(envelope.message, (CallMsg));

        committed = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: s.networkName, rawAddress: FvmAddressHelper.from(msg.sender)}),
            to: envelope.to,
            value: msg.value,
            message: envelope.message,
            nonce: 0 // nonce will be updated by LibGateway.commitCrossMessage
        });

        // Commit xnet message for dispatch.
        bool shouldBurn = LibGateway.commitCrossMessage(committed);

        // Apply side effects, such as burning funds.
        LibGateway.crossMsgSideEffects({v: committed.value, shouldBurn: shouldBurn});

        // Return a copy of the envelope, which was updated when it was committed.
        // Updates are visible to us because commitCrossMessage takes the envelope with memory scope,
        // which passes the struct by reference.
        return committed;
    }

    /**
     * @dev Propagates all the populated cross-net messages for the given `cid`.
     */
    function propagateAll() external payable {
        uint256 keysLength = s.postboxKeys.length(); // Cache length for gas efficiency
        for (uint256 i = 0; i < keysLength; ) {
            bytes32 msgCid = s.postboxKeys.at(i);
            _propagate(msgCid);

            unchecked {
                ++i;
            }
        }
    }

    /**
     * @dev Propagates the populated cross-net message for the given `msgCid`.
     * @param msgCid - the cid of the cross-net message
     */
    function propagate(bytes32 msgCid) external payable {
        _propagate(msgCid);
    }

    /**
     * @dev Internal function to propagate the cross-net message.
     * @param msgCid - the cid of the cross-net message
     */
    function _propagate(bytes32 msgCid) internal {
        IpcEnvelope storage crossMsg = s.postbox[msgCid];

        require(crossMsg.value > 0, "Invalid message CID or message");

        bool shouldBurn = LibGateway.commitCrossMessage(crossMsg);

        // Cache value before deletion to avoid re-entrancy
        uint256 v = crossMsg.value;

        // Remove the message to prevent re-entrancy and clean up state
        delete s.postbox[msgCid];
        s.postboxKeys.remove(msgCid);

        // Execute side effects
        LibGateway.crossMsgSideEffects({v: v, shouldBurn: shouldBurn});

        // Emit an event for off-chain monitoring
        // emit MessagePropagated(msgCid, v, shouldBurn);
    }
}
