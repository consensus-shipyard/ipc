// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "forge-std/Test.sol";

import "../src/lib/StorableMsgHelper.sol";

contract StorableMsgHelperTest is Test {
    using StorableMsgHelper for StorableMsg;

    uint64 private constant ROOTNET_CHAINID = 123;

    StorableMsg EMPTY_STORABLE_MESSAGE = StorableMsg({
        from: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
        to: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
        value: 0,
        nonce: 0,
        method: METHOD_SEND,
        params: EMPTY_BYTES
    });

    function test_ToHash_Works_EmptyMessage() public view {
        require(
            EMPTY_STORABLE_MESSAGE.toHash() == StorableMsgHelper.EMPTY_STORABLE_MESSAGE_HASH, "Hashes should be equal"
        );
    }

    function test_ToHash_Works_NonEmptyMessage() public pure {
        StorableMsg memory storableMsg = StorableMsg({
            from: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
            to: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
            value: 1,
            nonce: 1,
            method: METHOD_SEND,
            params: EMPTY_BYTES
        });
        bytes32 expectedHash = keccak256(abi.encode(storableMsg));
        require(storableMsg.toHash() == expectedHash, "Hashes should be equal");
    }

    function test_applyType_TopDown() public pure {
        address[] memory from = new address[](1);
        from[0] = address(1);
        address[] memory to = new address[](4);
        to[0] = address(1);
        to[1] = address(2);
        to[2] = address(3);
        to[3] = address(4);

        StorableMsg memory storableMsg = StorableMsg({
            from: IPCAddress({subnetId: SubnetID({root: ROOTNET_CHAINID, route: from}), rawAddress: address(3)}),
            to: IPCAddress({subnetId: SubnetID({root: ROOTNET_CHAINID, route: to}), rawAddress: address(3)}),
            value: 1,
            nonce: 1,
            method: METHOD_SEND,
            params: EMPTY_BYTES
        });

        require(
            storableMsg.applyType(SubnetID({root: ROOTNET_CHAINID, route: from})) == IPCMsgType.TopDown,
            "Should be TopDown"
        );

        address[] memory current = new address[](2);
        current[0] = address(1);
        current[1] = address(2);
        SubnetID memory subnetId = SubnetID({root: ROOTNET_CHAINID, route: current});

        require(storableMsg.applyType(subnetId) == IPCMsgType.TopDown, "Should be TopDown");

        address[] memory current2 = new address[](3);
        current2[0] = address(1);
        current2[1] = address(2);
        current2[2] = address(3);

        require(
            storableMsg.applyType(SubnetID({root: ROOTNET_CHAINID, route: current2})) == IPCMsgType.TopDown,
            "Should be TopDown"
        );
    }

    function test_applyType_BottomUp() public pure {
        address[] memory from = new address[](2);
        from[0] = address(1);
        from[1] = address(2);
        address[] memory to = new address[](1);
        to[0] = address(1);

        StorableMsg memory storableMsg = StorableMsg({
            from: IPCAddress({subnetId: SubnetID({root: ROOTNET_CHAINID, route: from}), rawAddress: address(3)}),
            to: IPCAddress({subnetId: SubnetID({root: ROOTNET_CHAINID, route: to}), rawAddress: address(3)}),
            value: 1,
            nonce: 1,
            method: METHOD_SEND,
            params: EMPTY_BYTES
        });

        require(
            storableMsg.applyType(SubnetID({root: ROOTNET_CHAINID, route: from})) == IPCMsgType.BottomUp,
            "Should be BottomUp"
        );
        require(
            storableMsg.applyType(SubnetID({root: ROOTNET_CHAINID, route: to})) == IPCMsgType.BottomUp,
            "Should be BottomUp"
        );
    }
}
