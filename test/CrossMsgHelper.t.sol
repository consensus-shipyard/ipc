// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/lib/CrossMsgHelper.sol";
import "../src/lib/SubnetIDHelper.sol";

contract CrossMsgHelperTest is Test {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using CrossMsgHelper for CrossMsg[];

    bytes32 immutable EMPTY_CROSS_MSGS_HASH = keccak256(abi.encode(createCrossMsgs(0, 0)));
    bytes32 immutable EMPTY_CROSS_MSG_HASH = keccak256(abi.encode(createCrossMsg(0)));
    uint64 private constant ROOTNET_CHAINID = 123;

    CrossMsg public crossMsg;
    CrossMsg[] public crossMsgs;

    error NoParentForSubnet();

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

    function test_IsEmpty_Works_EmptyCrossMsg() public view {
        require(crossMsg.isEmpty() == true);
    }

    function test_IsEmpty_Works_NonEmptyCrossMsg() public {
        crossMsg.message.nonce = 10;

        require(crossMsg.isEmpty() == false);
    }

    function test_CreateReleaseMsg_Works(uint256 releaseAmount, address sender) public {
        address[] memory route = new address[](2);
        route[0] = makeAddr("root");
        route[1] = makeAddr("subnet");
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, route);

        vm.prank(sender);

        CrossMsg memory releaseMsg = CrossMsgHelper.createReleaseMsg(subnetId, sender, releaseAmount);

        address[] memory parentRoute = new address[](1);
        parentRoute[0] = route[0];
        SubnetID memory parentSubnetId = SubnetID(ROOTNET_CHAINID, parentRoute);

        require(releaseMsg.message.from.subnetId.toHash() == subnetId.toHash());
        require(releaseMsg.message.from.rawAddress == BURNT_FUNDS_ACTOR);
        require(releaseMsg.message.to.subnetId.toHash() == parentSubnetId.toHash());
        require(releaseMsg.message.to.rawAddress == sender);
        require(releaseMsg.message.value == releaseAmount);
        require(releaseMsg.message.nonce == 0);
        require(releaseMsg.message.method == METHOD_SEND);
        require(keccak256(releaseMsg.message.params) == keccak256(EMPTY_BYTES));
        require(releaseMsg.wrapped == false);
    }

    function test_CreateReleaseMsg_Fails_SubnetNoParent(uint256 releaseAmount, address sender) public {
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](0));

        vm.expectRevert(NoParentForSubnet.selector);

        CrossMsgHelper.createReleaseMsg(subnetId, sender, releaseAmount);
    }

    function test_CreateFundMsg_Works_Root(uint256 fundAmount, address sender) public {
        address[] memory parentRoute = new address[](1);
        parentRoute[0] = address(101);
        SubnetID memory parentSubnetId = SubnetID(ROOTNET_CHAINID, parentRoute);

        vm.prank(sender);

        CrossMsg memory fundMsg = CrossMsgHelper.createFundMsg(parentSubnetId, sender, fundAmount);

        SubnetID memory rootSubnetId = SubnetID(ROOTNET_CHAINID, new address[](0));

        require(fundMsg.message.from.subnetId.toHash() == rootSubnetId.toHash());
        require(fundMsg.message.from.rawAddress == sender);
        require(fundMsg.message.to.subnetId.toHash() == parentSubnetId.toHash());
        require(fundMsg.message.to.rawAddress == sender);
        require(fundMsg.message.value == fundAmount);
        require(fundMsg.message.nonce == 0);
        require(fundMsg.message.method == METHOD_SEND);
        require(keccak256(fundMsg.message.params) == keccak256(EMPTY_BYTES));
        require(fundMsg.wrapped == false);
    }

    function test_CreateFundMsg_Works(uint256 fundAmount, address sender) public {
        address[] memory route = new address[](2);
        route[0] = makeAddr("root");
        route[1] = makeAddr("subnet");
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, route);

        vm.prank(sender);

        CrossMsg memory fundMsg = CrossMsgHelper.createFundMsg(subnetId, sender, fundAmount);

        address[] memory parentRoute = new address[](1);
        parentRoute[0] = route[0];
        SubnetID memory parentSubnetId = SubnetID(ROOTNET_CHAINID, parentRoute);

        require(fundMsg.message.from.subnetId.toHash() == parentSubnetId.toHash());
        require(fundMsg.message.from.rawAddress == sender);
        require(fundMsg.message.to.subnetId.toHash() == subnetId.toHash());
        require(fundMsg.message.to.rawAddress == sender);
        require(fundMsg.message.value == fundAmount);
        require(fundMsg.message.nonce == 0);
        require(fundMsg.message.method == METHOD_SEND);
        require(keccak256(fundMsg.message.params) == keccak256(EMPTY_BYTES));
        require(fundMsg.wrapped == false);
    }

    function test_CreateFundMsg_Fails_SubnetNoParent(uint256 fundAmount, address sender) public {
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](0));

        vm.expectRevert(NoParentForSubnet.selector);

        CrossMsgHelper.createFundMsg(subnetId, sender, fundAmount);
    }

    function test_Execute_Works_SendValue() public {
        address sender = address(this);
        address recipient = address(100);

        crossMsg.message.to.rawAddress = recipient;
        crossMsg.message.method = METHOD_SEND;
        crossMsg.message.value = 1;

        vm.deal(sender, 1 ether);

        bytes memory result = crossMsg.execute();

        require(keccak256(result) == keccak256(EMPTY_BYTES));
        require(recipient.balance == 1);
        require(sender.balance == 1 ether - 1);
    }

    function test_Execute_Works_FunctionCallWithValue() public {
        address sender = address(this);
        address recipient = address(this);

        crossMsg.message.to.rawAddress = recipient;
        crossMsg.message.method = this.callback.selector;
        crossMsg.message.value = 1;
        crossMsg.message.params = EMPTY_BYTES;

        vm.deal(sender, 1 ether);

        vm.expectCall(recipient, crossMsg.message.value, crossMsg.message.params);

        bytes memory result = crossMsg.execute();
        bytes memory decoded = abi.decode(result, (bytes));

        require(keccak256(decoded) == keccak256(crossMsg.message.params));
    }

    function test_Execute_Works_FunctionCall_Wrapped() public {
        address sender = address(this);
        address recipient = address(this);

        crossMsg.message.to.rawAddress = recipient;
        crossMsg.message.method = this.callback.selector;
        crossMsg.message.value = 0;
        crossMsg.message.params = EMPTY_BYTES;
        crossMsg.wrapped = true;

        vm.deal(sender, 1 ether);

        vm.expectCall(recipient, crossMsg.message.value, crossMsg.message.params);

        bytes memory result = crossMsg.execute();
        bytes memory decoded = abi.decode(result, (bytes));

        CrossMsg memory decodedCrossMsg = abi.decode(decoded, (CrossMsg));

        require(decodedCrossMsg.toHash() == crossMsg.toHash());
    }

    function test_Execute_Fails_InvalidMethod() public {
        vm.expectRevert("Address: low-level call failed");

        crossMsg.message.to.rawAddress = address(this);
        crossMsg.message.method = bytes4("1");

        crossMsg.execute();
    }

    function callback(bytes calldata params) public payable returns (bytes memory) {
        return params;
    }

    function test_IsSorted_Works_SingleMsg() public {
        addCrossMsg(0);

        require(CrossMsgHelper.isSorted(crossMsgs));
    }

    function test_IsSorted_Works_MultipleMsgsSorted() public {
        addCrossMsg(0);
        addCrossMsg(1);

        require(CrossMsgHelper.isSorted(crossMsgs));
    }

    function test_IsSorted_Works_MultipleMsgsNotSorted() public {
        addCrossMsg(0);
        addCrossMsg(2);
        addCrossMsg(1);

        require(CrossMsgHelper.isSorted(crossMsgs) == false);
    }

    function test_IsSorted_Works_MultipleMsgsZeroNonces() public {
        addCrossMsg(0);
        addCrossMsg(0);

        require(CrossMsgHelper.isSorted(crossMsgs) == false);
    }

    function createCrossMsg(uint64 nonce) internal pure returns (CrossMsg memory) {
        return CrossMsg({
            message: StorableMsg({
                from: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
                to: IPCAddress({subnetId: SubnetID(0, new address[](0)), rawAddress: address(0)}),
                value: 0,
                nonce: nonce,
                method: METHOD_SEND,
                params: EMPTY_BYTES
            }),
            wrapped: false
        });
    }

    function createCrossMsgs(uint256 length, uint64 nonce) internal pure returns (CrossMsg[] memory _crossMsgs) {
        _crossMsgs = new CrossMsg[](length);

        for (uint256 i = 0; i < length; i++) {
            _crossMsgs[i] = createCrossMsg(nonce);
        }
    }

    function addCrossMsg(uint64 nonce) internal {
        crossMsg.message.nonce = nonce;

        crossMsgs.push(crossMsg);
    }
}
