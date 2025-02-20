// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {IpcEnvelope, SubnetID} from "../../structs/CrossNet.sol";
import {LibGateway} from "../../lib/LibGateway.sol";
import {IPCMsgType} from "../../enums/IPCMsgType.sol";
import {SubnetActorGetterFacet} from "../../subnet/SubnetActorGetterFacet.sol";
import {Subnet} from "../../structs/Subnet.sol";

import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {CrossMsgHelper} from "../../lib/CrossMsgHelper.sol";
import {AssetHelper} from "../../lib/AssetHelper.sol";
import {Asset} from "../../structs/Subnet.sol";

import {NotRegisteredSubnet} from "../../errors/IPCErrors.sol";

contract XnetMessagingFacet is GatewayActorModifiers {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using AssetHelper for Asset;

    /// @notice Applies top-down cross-net messages locally. This is invoked by IPC nodes when drawing messages from
    ///         their parent subnet for local execution. That's why the sender is restricted to the system sender,
    ///         because this method is implicitly invoked by the node during block production.
    /// @dev It requires the caller to be the system actor.
    /// @param crossMsgs The array of cross-network messages to be applied.
    function applyCrossMessages(IpcEnvelope[] calldata crossMsgs) external systemActorOnly {
        LibGateway.applyTopDownMessages(s.networkName.getParentSubnet(), crossMsgs);
        LibGateway.propagateAllPostboxMessages();
    }
}
