// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../src/lib/CrossMsgHelper.sol";
import "../../src/lib/SubnetIDHelper.sol";
import "../../src/lib/FvmAddressHelper.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SupplySource} from "../../src/structs/Subnet.sol";

import "openzeppelin-contracts/utils/Address.sol";

contract CrossMsgHelperTest is Test {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for CrossMsg;
    using CrossMsgHelper for CrossMsg[];
    using FvmAddressHelper for FvmAddress;

    uint64 private constant ROOTNET_CHAINID = 123;
    uint256 CROSS_MESSAGE_FEE = 1 gwei;

    CrossMsg public crossMsg;
    CrossMsg[] public crossMsgs;

    error NoParentForSubnet();

    function test_IsEmpty_Works_EmptyCrossMsg() public view {
        require(crossMsg.isEmpty() == true);
    }

    function test_ToHash_Works() public view {
        CrossMsg[] memory msgs = new CrossMsg[](1);
        msgs[0] = crossMsg;
        require(CrossMsgHelper.toHash(crossMsg) == CrossMsgHelper.toHash(msgs[0]));
        require(CrossMsgHelper.toHash(crossMsg) != CrossMsgHelper.toHash(msgs));
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

        CrossMsg memory releaseMsg = CrossMsgHelper.createReleaseMsg(
            subnetId,
            sender,
            FvmAddressHelper.from(sender),
            releaseAmount,
            CROSS_MESSAGE_FEE
        );

        address[] memory parentRoute = new address[](1);
        parentRoute[0] = route[0];
        SubnetID memory parentSubnetId = SubnetID(ROOTNET_CHAINID, parentRoute);

        require(releaseMsg.message.from.subnetId.toHash() == subnetId.toHash());
        require(releaseMsg.message.from.rawAddress.extractEvmAddress() == sender);
        require(releaseMsg.message.to.subnetId.toHash() == parentSubnetId.toHash());
        require(releaseMsg.message.to.rawAddress.extractEvmAddress() == sender);
        require(releaseMsg.message.value == releaseAmount);
        require(releaseMsg.message.nonce == 0);
        require(releaseMsg.message.method == METHOD_SEND);
        require(keccak256(releaseMsg.message.params) == keccak256(EMPTY_BYTES));
        require(releaseMsg.wrapped == false);
    }

    function test_CreateReleaseMsg_Fails_SubnetNoParent(uint256 releaseAmount, address sender) public {
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](0));

        vm.expectRevert(NoParentForSubnet.selector);

        CrossMsgHelper.createReleaseMsg(
            subnetId,
            sender,
            FvmAddressHelper.from(sender),
            releaseAmount,
            CROSS_MESSAGE_FEE
        );
    }

    function test_CreateFundMsg_Works_Root(uint256 fundAmount, address sender) public {
        address[] memory parentRoute = new address[](1);
        parentRoute[0] = address(101);
        SubnetID memory parentSubnetId = SubnetID(ROOTNET_CHAINID, parentRoute);

        vm.prank(sender);

        CrossMsg memory fundMsg = CrossMsgHelper.createFundMsg(
            parentSubnetId,
            sender,
            FvmAddressHelper.from(sender),
            fundAmount,
            CROSS_MESSAGE_FEE
        );

        SubnetID memory rootSubnetId = SubnetID(ROOTNET_CHAINID, new address[](0));

        require(fundMsg.message.from.subnetId.toHash() == rootSubnetId.toHash());
        require(fundMsg.message.from.rawAddress.extractEvmAddress() == sender);
        require(fundMsg.message.to.subnetId.toHash() == parentSubnetId.toHash());
        require(fundMsg.message.to.rawAddress.extractEvmAddress() == sender);
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

        CrossMsg memory fundMsg = CrossMsgHelper.createFundMsg(
            subnetId,
            sender,
            FvmAddressHelper.from(sender),
            fundAmount,
            CROSS_MESSAGE_FEE
        );

        address[] memory parentRoute = new address[](1);
        parentRoute[0] = route[0];
        SubnetID memory parentSubnetId = SubnetID(ROOTNET_CHAINID, parentRoute);

        require(fundMsg.message.from.subnetId.toHash() == parentSubnetId.toHash());
        require(fundMsg.message.from.rawAddress.extractEvmAddress() == sender);
        require(fundMsg.message.to.subnetId.toHash() == subnetId.toHash());
        require(fundMsg.message.to.rawAddress.extractEvmAddress() == sender);
        require(fundMsg.message.value == fundAmount);
        require(fundMsg.message.nonce == 0);
        require(fundMsg.message.method == METHOD_SEND);
        require(keccak256(fundMsg.message.params) == keccak256(EMPTY_BYTES));
        require(fundMsg.wrapped == false);
    }

    function test_CreateFundMsg_Fails_SubnetNoParent(uint256 fundAmount, address sender) public {
        SubnetID memory subnetId = SubnetID(ROOTNET_CHAINID, new address[](0));

        vm.expectRevert(NoParentForSubnet.selector);

        CrossMsgHelper.createFundMsg(subnetId, sender, FvmAddressHelper.from(sender), fundAmount, CROSS_MESSAGE_FEE);
    }

    function test_Execute_Works_SendValue() public {
        address sender = address(this);
        address recipient = address(100);

        crossMsg.message.to.rawAddress = FvmAddressHelper.from(recipient);
        crossMsg.message.method = METHOD_SEND;
        crossMsg.message.value = 1;

        vm.deal(sender, 1 ether);

        bytes memory result = crossMsg.execute(SupplySourceHelper.native());

        require(keccak256(result) == keccak256(EMPTY_BYTES));
        require(recipient.balance == 1);
        require(sender.balance == 1 ether - 1);
    }

    function test_Execute_Works_FunctionCallWithValue() public {
        address sender = address(this);
        address recipient = address(this);

        crossMsg.message.to.rawAddress = FvmAddressHelper.from(recipient);
        crossMsg.message.method = this.callback.selector;
        crossMsg.message.value = 1;
        crossMsg.message.params = abi.encode(EMPTY_BYTES);
        crossMsg.wrapped = false;

        vm.deal(sender, 1 ether);
        vm.expectCall(recipient, crossMsg.message.value, abi.encodeCall(this.callback, EMPTY_BYTES));

        bytes memory result = crossMsg.execute(SupplySourceHelper.native());
        bytes memory decoded = abi.decode(result, (bytes));

        require(keccak256(decoded) == keccak256(EMPTY_BYTES));
    }

    function test_Execute_Works_FunctionCallWithoutValue() public {
        address sender = address(this);
        address recipient = address(this);

        crossMsg.message.to.rawAddress = FvmAddressHelper.from(recipient);
        crossMsg.message.method = this.callback.selector;
        crossMsg.message.value = 0;
        crossMsg.message.params = abi.encode(EMPTY_BYTES);
        crossMsg.wrapped = false;

        vm.deal(sender, 1 ether);
        vm.expectCall(recipient, crossMsg.message.value, abi.encodeCall(this.callback, EMPTY_BYTES));

        bytes memory result = crossMsg.execute(SupplySourceHelper.native());
        bytes memory decoded = abi.decode(result, (bytes));

        require(keccak256(decoded) == keccak256(EMPTY_BYTES));
    }

    function test_Execute_Works_FunctionCall_Wrapped() public {
        address sender = address(this);
        address recipient = address(this);

        crossMsg.message.to.rawAddress = FvmAddressHelper.from(recipient);
        crossMsg.message.method = this.callbackWrapped.selector;
        crossMsg.message.value = 0;
        crossMsg.message.params = abi.encode(EMPTY_BYTES);
        crossMsg.wrapped = true;

        vm.deal(sender, 1 ether);

        vm.expectCall(recipient, crossMsg.message.value, abi.encodeCall(this.callbackWrapped, crossMsg));
        bytes memory result = crossMsg.execute(SupplySourceHelper.native());

        bytes memory decoded = abi.decode(result, (bytes));
        CrossMsg memory decodedCrossMsg = abi.decode(decoded, (CrossMsg));

        require(decodedCrossMsg.toHash() == crossMsg.toHash(), "decoded.toHash() == crossMsg.toHash()");
    }

    function test_Execute_Fails_InvalidMethod() public {
        SupplySource memory native = SupplySourceHelper.native();

        crossMsg.message.to.rawAddress = FvmAddressHelper.from(address(this));
        crossMsg.message.method = bytes4("1");

        vm.expectRevert(Address.FailedInnerCall.selector);
        crossMsg.execute(native);
    }

    function callback(bytes calldata params) public payable returns (bytes memory) {
        return params;
    }

    function callbackWrapped(CrossMsg memory w) public payable returns (bytes memory) {
        return abi.encode(w);
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

    function createCrossMsg(uint64 nonce, uint256 fee) internal pure returns (CrossMsg memory) {
        return
            CrossMsg({
                message: StorableMsg({
                    from: IPCAddress({
                        subnetId: SubnetID(0, new address[](0)),
                        rawAddress: FvmAddressHelper.from(address(0))
                    }),
                    to: IPCAddress({
                        subnetId: SubnetID(0, new address[](0)),
                        rawAddress: FvmAddressHelper.from(address(0))
                    }),
                    value: 0,
                    nonce: nonce,
                    method: METHOD_SEND,
                    params: EMPTY_BYTES,
                    fee: fee
                }),
                wrapped: false
            });
    }

    function createCrossMsgs(uint256 length, uint64 nonce) internal view returns (CrossMsg[] memory _crossMsgs) {
        _crossMsgs = new CrossMsg[](length);

        for (uint256 i = 0; i < length; i++) {
            _crossMsgs[i] = createCrossMsg(nonce, CROSS_MESSAGE_FEE);
        }
    }

    function addCrossMsg(uint64 nonce) internal {
        crossMsg.message.nonce = nonce;

        crossMsgs.push(crossMsg);
    }
}
