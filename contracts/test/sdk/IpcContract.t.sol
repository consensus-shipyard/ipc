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

interface Foo {
    function foo(string calldata) external returns (string memory);
}

contract RecorderIpcExchange is IpcExchange {
    IpcEnvelope private lastEnvelope;
    CallMsg private lastCallMsg;
    ResultMsg private lastResultMsg;
    bool private shouldRevert;

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

contract IpcExchangeTest is Test {
    using CrossMsgHelper for IpcEnvelope;

    function test_IpcExchange() public {
        address gateway = vm.addr(1);

        address[] memory pathA = new address[](1);
        pathA[0] = vm.addr(2000);
        address[] memory pathB = new address[](1);
        pathB[0] = vm.addr(3000);

        // these two subnets are siblings.
        SubnetID memory subnetA = SubnetID({root: 123, route: pathA});
        SubnetID memory subnetB = SubnetID({root: 123, route: pathB});

        CallMsg memory callMsg = CallMsg({method: abi.encodePacked(Foo.foo.selector), params: bytes("1234")});
        IpcEnvelope memory envelope = IpcEnvelope({
            kind: IpcMsgKind.Transfer,
            from: IPCAddress({subnetId: subnetA, rawAddress: FvmAddressHelper.from(address(100))}),
            to: IPCAddress({subnetId: subnetB, rawAddress: FvmAddressHelper.from(address(200))}),
            value: 1000,
            message: abi.encode(callMsg),
            nonce: 0
        });

        RecorderIpcExchange exch = new RecorderIpcExchange(gateway);

        // a transfer; fails because cannot handle.
        vm.expectRevert(IpcHandler.UnsupportedMsgKind.selector);
        vm.prank(gateway);
        exch.handleIpcMessage(envelope);

        // a call; fails when the caller is not the gateway.
        envelope.kind = IpcMsgKind.Call;
        vm.expectRevert(IpcHandler.CallerIsNotGateway.selector);
        exch.handleIpcMessage(envelope);

        vm.startPrank(gateway);
        exch.handleIpcMessage(envelope);

        // succeeds.
        IpcEnvelope memory lastEnvelope = exch.getLastEnvelope();
        CallMsg memory lastCall = exch.getLastCallMsg();
        require(keccak256(abi.encode(envelope)) == keccak256(abi.encode(lastEnvelope)), "unexpected envelope");
        require(keccak256(abi.encode(callMsg)) == keccak256(abi.encode(lastCall)), "unexpected callmsg");

        // a revert bubbles up.
        exch.flipRevert();
        vm.expectRevert("revert requested");
        exch.handleIpcMessage(envelope);

        // an unrecognized result
        envelope.kind = IpcMsgKind.Result;
        envelope.message = abi.encode(ResultMsg({outcome: OutcomeType.Ok, id: keccak256("foo"), ret: bytes("")}));

        IPCAddress memory from = envelope.from;
        envelope.from = envelope.to;
        envelope.to = from;

        vm.expectRevert(IpcHandler.UnrecognizedResult.selector);
        exch.handleIpcMessage(envelope);

        vm.mockCall(gateway, abi.encodeWithSelector(IGateway.sendContractXnetMessage.selector), abi.encode(envelope));
        vm.deal(address(this), 1000);
        exch.performIpcCall_(from, CallMsg({method: bytes("1234"), params: bytes("AABB")}), 1);

        // we store the correct envelope in the correlation map.
        IpcEnvelope memory correlated = exch.getInflight(envelope.toHash());
        require(correlated.toHash() == envelope.toHash());

        // TODO test receipt correlation

        // TODO test dropMessages
    }
}
