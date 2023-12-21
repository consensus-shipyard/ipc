// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {CrossMsg, SubnetID, StorableMsg} from "../../structs/CrossNet.sol";
import {LibGateway} from "../../lib/LibGateway.sol";
import {IPCMsgType} from "../../enums/IPCMsgType.sol";
import {SubnetActorGetterFacet} from "../../subnet/SubnetActorGetterFacet.sol";
import {Subnet} from "../../structs/Subnet.sol";

import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {CrossMsgHelper} from "../../lib/CrossMsgHelper.sol";
import {SupplySourceHelper} from "../../lib/SupplySourceHelper.sol";
import {SupplySource} from "../../structs/Subnet.sol";

import {InvalidCrossMsgNonce, InvalidCrossMsgNonce, NotRegisteredSubnet, InvalidCrossMsgDstSubnet} from "../../errors/IPCErrors.sol";

import {StorableMsgHelper} from "../../lib/StorableMsgHelper.sol";

contract XnetMessagingFacet is GatewayActorModifiers {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using SupplySourceHelper for SupplySource;
    using StorableMsgHelper for StorableMsg;

    /// @notice Applies top-down cross-net messages locally. This is invoked by IPC nodes when drawing messages from
    ///         their parent subnet for local execution. That's why the sender is restricted to the system sender,
    ///         because this method is implicitly invoked by the node during block production.
    /// @dev It requires the caller to be the system actor.
    /// @param crossMsgs The array of cross-network messages to be applied.
    function applyCrossMessages(CrossMsg[] calldata crossMsgs) external systemActorOnly {
        LibGateway.applyMessages(s.networkName.getParentSubnet(), crossMsgs);
    }
}
