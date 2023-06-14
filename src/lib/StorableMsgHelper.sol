// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "../structs/Checkpoint.sol";
import "../constants/Constants.sol";
import "../lib/SubnetIDHelper.sol";
import "../enums/IPCMsgType.sol";

/// @title Helper library for manipulating StorableMsg struct
/// @author LimeChain team
library StorableMsgHelper {
    using SubnetIDHelper for SubnetID;

    bytes32 public constant EMPTY_STORABLE_MESSAGE_HASH = keccak256(
        abi.encode(
            StorableMsg({
                from: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
                to: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
                value: 0,
                nonce: 0,
                method: METHOD_SEND,
                params: EMPTY_BYTES
            })
        )
    );

    function applyType(StorableMsg calldata message, SubnetID calldata currentSubnet)
        public
        pure
        returns (IPCMsgType)
    {
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
