// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../../src/errors/IPCErrors.sol";
import {IpcEnvelope, CallMsg, ResultMsg, IpcMsgKind, OutcomeType} from "../../src/structs/CrossNet.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../../src/structs/Subnet.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../src/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../src/lib/CrossMsgHelper.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {IpcHandler, IpcExchange} from "../../sdk/IpcContract.sol";
import {IGateway} from "../../src/interfaces/IGateway.sol";
import {CrossMsgHelper} from "../../src/lib/CrossMsgHelper.sol";

import {IntegrationTestBase, TestRegistry} from "../IntegrationTestBase.sol";

interface Foo {
    function foo(string calldata) external returns (string memory);
}

contract RecorderIpcExchange is IpcExchange {
    IpcEnvelope private lastEnvelope;
    CallMsg private lastCallMsg;
    ResultMsg private lastResultMsg;
    bool private shouldRevert;
    bool public handleIpcResultCalled = false;

    constructor(address gatewayAddr_) IpcExchange(gatewayAddr_) {}

    function _handleIpcCall(
        IpcEnvelope memory envelope,
        CallMsg memory callMsg
    ) internal override returns (bytes memory) {
        require(!shouldRevert, "revert requested");
        console.log("handling ipc call");
        lastEnvelope = envelope;
        lastCallMsg = callMsg;
        return bytes("");
    }

    function _handleIpcResult(
        IpcEnvelope storage original,
        IpcEnvelope memory result,
        ResultMsg memory resultMsg
    ) internal override {
        // indicate this method was called
        handleIpcResultCalled = true;

        require(!shouldRevert, "revert requested");
        console.log("handling ipc result");
        require(keccak256(abi.encode(original)) == keccak256(abi.encode(lastEnvelope)));
        lastEnvelope = result;
        lastResultMsg = resultMsg;
    }

    function flipRevert() public {
        shouldRevert = !shouldRevert;
    }

    // Expose this method so we can test it.
    function performIpcCall_(IPCAddress calldata to, CallMsg calldata callMsg, uint256 value) public {
        performIpcCall(to, callMsg, value);
    }

    // We need these manual getters because Solidity-generated ones on public fields decompose the struct
    // into its constituents.
    function getLastEnvelope() public view returns (IpcEnvelope memory) {
        return lastEnvelope;
    }

    // We need these manual getters because Solidity-generated ones on public fields decompose the struct
    // into its constituents.
    function getLastCallMsg() public view returns (CallMsg memory) {
        return lastCallMsg;
    }

    // We need these manual getters because Solidity-generated ones on public fields decompose the struct
    // into its constituents.
    function getLastResultMsg() public view returns (ResultMsg memory) {
        return lastResultMsg;
    }

    // We need these manual getters because Solidity-generated ones on public fields decompose the struct
    // into its constituents.
    function getInflight(bytes32 id) public view returns (IpcEnvelope memory) {
        return inflightMsgs[id];
    }
}

contract IpcExchangeTest is Test, IntegrationTestBase {
    using CrossMsgHelper for IpcEnvelope;
    address gateway = vm.addr(1);
    SubnetID subnetA;
    SubnetID subnetB;
    CallMsg callMsg;
    ResultMsg resultMsg;
    IpcEnvelope callEnvelope;
    IpcEnvelope resultEnvelope;
    RecorderIpcExchange exch;

    IPCAddress ipcAddressA;
    IPCAddress ipcAddressB;

    function setUp() public override {
        address[] memory pathA = new address[](1);
        pathA[0] = vm.addr(2000);
        address[] memory pathB = new address[](1);
        pathB[0] = vm.addr(3000);

        // these two subnets are siblings.
        subnetA = SubnetID({root: 123, route: pathA});
        subnetB = SubnetID({root: 123, route: pathB});
        ipcAddressA = IPCAddress({subnetId: subnetA, rawAddress: FvmAddressHelper.from(address(100))});
        ipcAddressB = IPCAddress({subnetId: subnetB, rawAddress: FvmAddressHelper.from(address(200))});

        callMsg = CallMsg({method: abi.encodePacked(Foo.foo.selector), params: bytes("1234")});
        callEnvelope = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: ipcAddressA,
            to: ipcAddressB,
            value: 1000,
            message: abi.encode(callMsg),
            nonce: 0
        });

        resultMsg = ResultMsg({outcome: OutcomeType.Ok, id: callEnvelope.toHash(), ret: bytes("")});

        resultEnvelope = IpcEnvelope({
            kind: IpcMsgKind.Result,
            from: ipcAddressB,
            to: ipcAddressA,
            value: 1000,
            message: abi.encode(resultMsg),
            nonce: 0
        });

        exch = new RecorderIpcExchange(gateway);
    }

    function test_IpcExchangeTestTransferFails() public {
        callEnvelope.kind = IpcMsgKind.Transfer;

        // a transfer; fails because cannot handle.
        vm.expectRevert(IpcHandler.UnsupportedMsgKind.selector);
        vm.prank(gateway);
        exch.handleIpcMessage(callEnvelope);
    }

    function test_IpcExchangeTestGatewayOnlyFails() public {
        // a call; fails when the caller is not the gateway.
        vm.expectRevert(IpcHandler.CallerIsNotGateway.selector);
        exch.handleIpcMessage(callEnvelope);
    }

    function test_IpcExchange() public {
        vm.startPrank(gateway);
        exch.handleIpcMessage(callEnvelope);

        // succeeds.
        IpcEnvelope memory lastEnvelope = exch.getLastEnvelope();
        CallMsg memory lastCall = exch.getLastCallMsg();
        require(keccak256(abi.encode(callEnvelope)) == keccak256(abi.encode(lastEnvelope)), "unexpected callEnvelope");
        require(keccak256(abi.encode(callMsg)) == keccak256(abi.encode(lastCall)), "unexpected callmsg");
    }

    function test_IpcExchangeFlipRevert() public {
        vm.startPrank(gateway);
        // a revert bubbles up.
        exch.flipRevert();
        vm.expectRevert("revert requested");
        exch.handleIpcMessage(callEnvelope);
    }

    function test_IpcExchangeUnexpectedResult() public {
        vm.startPrank(gateway);
        //
        // an unrecognized result
        callEnvelope.kind = IpcMsgKind.Result;
        callEnvelope.message = abi.encode(ResultMsg({outcome: OutcomeType.Ok, id: keccak256("foo"), ret: bytes("")}));

        IPCAddress memory from = callEnvelope.from;
        callEnvelope.from = callEnvelope.to;
        callEnvelope.to = from;

        vm.expectRevert(IpcHandler.UnrecognizedResult.selector);
        exch.handleIpcMessage(callEnvelope);
    }

    function test_IpcExchangeTestReceiptCorrelation() public {
        vm.startPrank(gateway);
        vm.mockCall(
            gateway,
            abi.encodeWithSelector(IGateway.sendContractXnetMessage.selector),
            abi.encode(callEnvelope)
        );
        vm.deal(address(this), 1000);
        exch.performIpcCall_(callEnvelope.from, CallMsg({method: bytes("1234"), params: bytes("AABB")}), 1);

        // we store the correct callEnvelope in the correlation map.
        IpcEnvelope memory correlated = exch.getInflight(callEnvelope.toHash());
        require(correlated.toHash() == callEnvelope.toHash());

        // TODO test receipt correlation

        // TODO test dropMessages
    }

    function test_IpcExchangeCallResult() public {
        vm.mockCall(
            gateway,
            abi.encodeWithSelector(IGateway.sendContractXnetMessage.selector),
            abi.encode(callEnvelope)
        );
        vm.deal(address(this), 1000);
        exch.performIpcCall_(ipcAddressA, callMsg, 1);

        //possibly move start prank here
        vm.startPrank(gateway);
        exch.handleIpcMessage(callEnvelope);

        // succeeds.
        IpcEnvelope memory lastEnvelope = exch.getLastEnvelope();
        CallMsg memory lastCall = exch.getLastCallMsg();
        require(keccak256(abi.encode(callEnvelope)) == keccak256(abi.encode(lastEnvelope)), "unexpected callEnvelope");
        require(keccak256(abi.encode(callMsg)) == keccak256(abi.encode(lastCall)), "unexpected callmsg");

        // gateway calls callback with result
        exch.handleIpcMessage(resultEnvelope);
        require(exch.handleIpcResultCalled(), "_handleIpcResult was not called");
    }
}
