// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/lib/CrossMsgHelper.sol";
import "../src/lib/SubnetIDHelper.sol";

contract CrossMsgHelperTest is Test {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using CrossMsgHelper for CrossMsg[];

    bytes32 EMPTY_CROSS_MSGS_HASH =
        keccak256(abi.encode(createCrossMsgs(0, 0)));
    bytes32 EMPTY_CROSS_MSG_HASH = keccak256(abi.encode(createCrossMsg(0)));

    CrossMsg public crossMsg;
    CrossMsg[] public crossMsgs;

    function test_ToHash_Works_EmptyCrossMsg() public view {
        require(crossMsg.toHash() == EMPTY_CROSS_MSG_HASH);
    }

    function test_ToHash_Works_NonEmptyCrossMsg(uint64 nonce) public {
        crossMsg.message.nonce = nonce;

        CrossMsg memory crossMsgExpected = createCrossMsg(nonce);

        require(crossMsg.toHash() == crossMsgExpected.toHash());
    }

    function test_ToHash_Works_EmptyCrossMsgs() public view {
        require(crossMsgs.toHash() == EMPTY_CROSS_MSGS_HASH);
    }

    function test_ToHash_Works_NonEmptyCrossMsgs(uint64 nonce) public {
        crossMsg.message.nonce = nonce;
        crossMsgs.push(crossMsg);

        CrossMsg[] memory crossMsgsExpected = createCrossMsgs(1, nonce);

        require(crossMsgs.toHash() == crossMsgsExpected.toHash());
    }

    function test_CreateReleaseMsg_Works(
        uint256 releaseAmount,
        uint64 nonce,
        address sender
    ) public {
        address[] memory route = new address[](2);
        route[0] = makeAddr("root");
        route[1] = makeAddr("subnet");
        SubnetID memory subnetId = SubnetID(route);

        vm.prank(sender);

        CrossMsg memory releaseMsg = CrossMsgHelper.createReleaseMsg(
            subnetId,
            sender,
            releaseAmount,
            nonce
        );

        address[] memory parentRoute = new address[](1);
        parentRoute[0] = route[0];
        SubnetID memory parentSubnetId = SubnetID(parentRoute);

        require(releaseMsg.message.from.subnetId.toHash() == subnetId.toHash());
        require(releaseMsg.message.from.rawAddress == BURNT_FUNDS_ACTOR);
        require(
            releaseMsg.message.to.subnetId.toHash() == parentSubnetId.toHash()
        );
        require(releaseMsg.message.to.rawAddress == sender);
        require(releaseMsg.message.value == releaseAmount);
        require(releaseMsg.message.nonce == nonce);
        require(releaseMsg.message.method == 0);
        require(keccak256(releaseMsg.message.params) == keccak256(EMPTY_BYTES));
        require(releaseMsg.wrapped == false);
    }

    function test_CreateReleaseMsg_Fails_SubnetNoParent(
        uint256 releaseAmount,
        uint64 nonce,
        address sender
    ) public {
        address[] memory route = new address[](1);
        route[0] = makeAddr("root");
        SubnetID memory subnetId = SubnetID(route);

        vm.expectRevert("error getting parent for subnet addr");

        CrossMsgHelper.createReleaseMsg(subnetId, sender, releaseAmount, nonce);
    }

    function test_CreateFundMsg_Works(
        uint256 fundAmount,
        address sender
    ) public {
        address[] memory route = new address[](2);
        route[0] = makeAddr("root");
        route[1] = makeAddr("subnet");
        SubnetID memory subnetId = SubnetID(route);

        vm.prank(sender);

        CrossMsg memory fundMsg = CrossMsgHelper.createFundMsg(
            subnetId,
            sender,
            fundAmount
        );

        address[] memory parentRoute = new address[](1);
        parentRoute[0] = route[0];
        SubnetID memory parentSubnetId = SubnetID(parentRoute);

        require(
            fundMsg.message.from.subnetId.toHash() == parentSubnetId.toHash()
        );
        require(fundMsg.message.from.rawAddress == sender);
        require(fundMsg.message.to.subnetId.toHash() == subnetId.toHash());
        require(fundMsg.message.to.rawAddress == sender);
        require(fundMsg.message.value == fundAmount);
        require(fundMsg.message.nonce == 0);
        require(fundMsg.message.method == 0);
        require(keccak256(fundMsg.message.params) == keccak256(EMPTY_BYTES));
        require(fundMsg.wrapped == false);
    }

    function test_CreateFundMsg_Fails_SubnetNoParent(
        uint256 fundAmount,
        address sender
    ) public {
        address[] memory noParentRoute = new address[](1);
        noParentRoute[0] = makeAddr("root");
        SubnetID memory subnetId = SubnetID(noParentRoute);

        vm.expectRevert("error getting parent for subnet addr");

        CrossMsgHelper.createFundMsg(subnetId, sender, fundAmount);
    }

    function createCrossMsg(
        uint64 nonce
    ) internal pure returns (CrossMsg memory) {
        return
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID(new address[](0)),
                        rawAddress: address(0)
                    }),
                    to: IPCAddress({
                        subnetId: SubnetID(new address[](0)),
                        rawAddress: address(0)
                    }),
                    value: 0,
                    nonce: nonce,
                    method: 0,
                    params: EMPTY_BYTES
                }),
                wrapped: false
            });
    }

    function createCrossMsgs(
        uint256 length,
        uint64 nonce
    ) internal pure returns (CrossMsg[] memory _crossMsgs) {
        _crossMsgs = new CrossMsg[](length);

        for (uint i = 0; i < length; i++) {
            _crossMsgs[i] = createCrossMsg(nonce);
        }
    }
}
