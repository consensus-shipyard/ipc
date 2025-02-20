// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {GatewayActorModifiers} from "../lib/LibGatewayActorStorage.sol";
import {IpcEnvelope, CallMsg, IpcMsgKind} from "../structs/CrossNet.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";
import {Subnet, SubnetID, AssetKind, IPCAddress, Asset} from "../structs/Subnet.sol";
import {InvalidXnetMessage, InvalidXnetMessageReason, MethodNotAllowed} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {LibGateway} from "../lib/LibGateway.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {AssetHelper} from "../lib/AssetHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

string constant ERR_GENERAL_CROSS_MSG_DISABLED = "Support for general-purpose cross-net messages is disabled";
string constant ERR_MULTILEVEL_CROSS_MSG_DISABLED = "Support for multi-level cross-net messages is disabled";

contract GatewayMessengerFacet is GatewayActorModifiers {
    using FilAddress for address payable;
    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.Bytes32Set;
    using CrossMsgHelper for IpcEnvelope;
    using AssetHelper for Asset;

    /**
     * @dev Sends a general-purpose cross-message from the local subnet to the destination subnet.
     * IMPORTANT: Native tokens via msg.value are treated as a contribution toward gas costs associated with message propagation.
     * There is no strict enforcement of the exact gas cost, and any msg.value provided will be accepted.
     *
     * IMPORTANT: Only smart contracts are allowed to trigger these cross-net messages. User wallets can send funds
     * from their address to the destination subnet and then run the transaction in the destination normally.
     *
     * @param envelope - the original envelope, which will be validated, stamped, and committed during the send.
     * @return committed envelope.
     */
    function sendContractXnetMessage(
        IpcEnvelope memory envelope
    ) external payable returns (IpcEnvelope memory committed) {
        if (!s.generalPurposeCrossMsg) {
            revert MethodNotAllowed(ERR_GENERAL_CROSS_MSG_DISABLED);
        }

        // We prevent the sender from being an EoA.
        if (msg.sender.code.length == 0) {
            revert InvalidXnetMessage(InvalidXnetMessageReason.Sender);
        }

        // Will revert if the message won't deserialize into a CallMsg.
        abi.decode(envelope.message, (CallMsg));

        committed = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: s.networkName, rawAddress: FvmAddressHelper.from(msg.sender)}),
            to: envelope.to,
            value: envelope.value,
            message: envelope.message,
            // nonce and originalNonce will be updated by LibGateway.commitValidatedCrossMessage
            originalNonce: 0,
            localNonce: 0
        });

        (bool valid, InvalidXnetMessageReason reason, IPCMsgType applyType) = committed.validateCrossMessage();
        if (!valid) {
            revert InvalidXnetMessage(reason);
        }

        if (applyType == IPCMsgType.TopDown) {
            (, SubnetID memory nextHop) = committed.to.subnetId.down(s.networkName);
            // lock funds on the current subnet gateway for the next hop
            ISubnetActor(nextHop.getActor()).supplySource().lock(envelope.value);
        }

        // Commit xnet message for dispatch.
        bool shouldBurn = LibGateway.commitValidatedCrossMessage(committed);

        // Apply side effects, such as burning funds.
        LibGateway.crossMsgSideEffects({v: committed.value, shouldBurn: shouldBurn});

        // Return a copy of the envelope, which was updated when it was committed.
        // Updates are visible to us because commitCrossMessage takes the envelope with memory scope,
        // which passes the struct by reference.
        return committed;
    }

    /**
     * @dev Propagates all the populated cross-net messages from the postbox.
     */
    function propagateAll() external payable {
        LibGateway.propagateAllPostboxMessages();
    }
}
