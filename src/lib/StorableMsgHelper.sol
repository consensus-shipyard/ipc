// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {SubnetID} from "../structs/Subnet.sol";
import {StorableMsg} from "../structs/Checkpoint.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../constants/Constants.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {IPCMsgType} from "../enums/IPCMsgType.sol";

/// @title Helper library for manipulating StorableMsg struct
/// @author LimeChain team
library StorableMsgHelper {
    using SubnetIDHelper for SubnetID;

    function applyType(StorableMsg calldata message, SubnetID calldata currentSubnet) public pure returns (IPCMsgType) {
        SubnetID memory toSubnet = message.to.subnetId;
        SubnetID memory fromSubnet = message.from.subnetId;
        SubnetID memory currentParentSubnet = currentSubnet.commonParent(toSubnet);
        SubnetID memory messageParentSubnet = fromSubnet.commonParent(toSubnet);

        if (currentParentSubnet.equals(messageParentSubnet)) {
            if (fromSubnet.route.length > messageParentSubnet.route.length) {
                return IPCMsgType.BottomUp;
            }
        }

        return IPCMsgType.TopDown;
    }

    function toHash(StorableMsg calldata storableMsg) public pure returns (bytes32) {
        return keccak256(abi.encode(storableMsg));
    }
}
