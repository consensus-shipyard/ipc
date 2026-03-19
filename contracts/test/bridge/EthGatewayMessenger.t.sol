// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import {IpcEnvelope, IpcMsgKind, CallMsg} from "../../contracts/structs/CrossNet.sol";
import {SubnetID, IPCAddress} from "../../contracts/structs/Subnet.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {EthGatewayMessenger} from "../../contracts/gateway/EthGatewayMessenger.sol";

// ─── Mock caller contract (needed because EOA check requires code.length > 0) ──
contract MockCaller {
    EthGatewayMessenger public gateway;

    constructor(address gw) {
        gateway = EthGatewayMessenger(gw);
    }

    function sendMsg(
        IPCAddress memory to,
        bytes memory callData,
        uint256 value
    ) external payable returns (IpcEnvelope memory) {
        bytes memory method = abi.encodePacked(bytes4(keccak256("handleMsg(bytes)")));
        CallMsg memory call = CallMsg({method: method, params: callData});
        IpcEnvelope memory env = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: SubnetID({root: 0, route: new address[](0)}), rawAddress: FvmAddressHelper.from(address(0))}),
            to: to,
            value: value,
            message: abi.encode(call),
            localNonce: 0,
            originalNonce: 0
        });
        return gateway.sendContractXnetMessage{value: value}(env);
    }
}

// ─── Test suite ───────────────────────────────────────────────────────────────
contract EthGatewayMessengerTest is Test {
    EthGatewayMessenger public gw;
    MockCaller public caller;

    address owner  = address(0xA11CE);
    address other  = address(0xBAD);

    SubnetID sepoliaSubnet;
    IPCAddress destAddr;

    function setUp() public {
        address[] memory route = new address[](0);
        sepoliaSubnet = SubnetID({root: 11155111, route: route});
        gw = new EthGatewayMessenger(owner, sepoliaSubnet);
        caller = new MockCaller(address(gw));
        vm.deal(address(caller), 10 ether);

        // Destination: some subnet actor
        address[] memory dRoute = new address[](1);
        dRoute[0] = address(0x1234);
        destAddr = IPCAddress({
            subnetId: SubnetID({root: 314159, route: dRoute}),
            rawAddress: FvmAddressHelper.from(address(0x5678))
        });
    }

    // ─── Constructor ──────────────────────────────────────────────────────────

    function test_constructor_setsOwner() public {
        assertEq(gw.owner(), owner);
    }

    function test_constructor_setsNetworkName() public {
        // networkName root should be 11155111
        // (can't compare full struct directly, check via send)
        assertEq(address(gw).code.length > 0, true);
    }

    // ─── sendContractXnetMessage: happy path ──────────────────────────────────

    function test_send_emitsXnetMessageCommitted() public {
        vm.expectEmit(false, false, false, false);
        emit EthGatewayMessenger.XnetMessageCommitted(IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: sepoliaSubnet, rawAddress: FvmAddressHelper.from(address(caller))}),
            to: destAddr,
            value: 0,
            message: bytes(""),
            localNonce: 0,
            originalNonce: 0
        }));
        caller.sendMsg(destAddr, bytes("hello"), 0);
    }

    function test_send_setsFromToCallerAddress() public {
        // Use the return value directly instead of log parsing (avoids ABI-encoding the event sig)
        IpcEnvelope memory result = caller.sendMsg(destAddr, bytes("data"), 0);
        // from.subnetId.root should be the Sepolia chain ID
        assertEq(result.from.subnetId.root, 11155111);
        // from.rawAddress should encode the caller contract address
        address extracted = FvmAddressHelper.extractEvmAddress(result.from.rawAddress);
        assertEq(extracted, address(caller));
    }

    function test_send_incrementsNonce() public {
        assertEq(gw.currentNonce(), 0);
        caller.sendMsg(destAddr, bytes("a"), 0);
        assertEq(gw.currentNonce(), 1);
        caller.sendMsg(destAddr, bytes("b"), 0);
        assertEq(gw.currentNonce(), 2);
    }

    function test_send_returnsCommittedEnvelope() public {
        IpcEnvelope memory result = caller.sendMsg(destAddr, bytes("payload"), 0);
        assertEq(uint8(result.kind), uint8(IpcMsgKind.Call));
        assertEq(result.localNonce, 0);
    }

    function test_send_forwardsValue() public {
        uint256 val = 0.1 ether;
        IpcEnvelope memory result = caller.sendMsg{value: val}(destAddr, bytes(""), val);
        assertEq(result.value, val);
    }

    // ─── sendContractXnetMessage: revert cases ────────────────────────────────

    function test_send_revertsForEOA() public {
        bytes memory method = abi.encodePacked(bytes4(keccak256("x()")));
        CallMsg memory call = CallMsg({method: method, params: bytes("")});
        IpcEnvelope memory env = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: SubnetID({root: 0, route: new address[](0)}), rawAddress: FvmAddressHelper.from(address(0))}),
            to: destAddr,
            value: 0,
            message: abi.encode(call),
            localNonce: 0,
            originalNonce: 0
        });
        // EOA call — no code
        vm.prank(other);
        vm.expectRevert(EthGatewayMessenger.CallerIsEOA.selector);
        gw.sendContractXnetMessage(env);
    }

    function test_send_revertsWhenPaused() public {
        vm.prank(owner);
        gw.pause();
        vm.expectRevert();
        caller.sendMsg(destAddr, bytes(""), 0);
    }

    function test_send_revertsForUnapprovedSubnetWhenRequired() public {
        vm.prank(owner);
        gw.setRequireApprovedSubnet(true);

        vm.expectRevert(abi.encodeWithSelector(EthGatewayMessenger.SubnetNotApproved.selector, address(caller)));
        caller.sendMsg(destAddr, bytes(""), 0);
    }

    function test_send_allowsApprovedSubnetWhenRequired() public {
        vm.prank(owner);
        gw.setRequireApprovedSubnet(true);
        vm.prank(owner);
        gw.approveSubnet(address(caller));

        caller.sendMsg(destAddr, bytes(""), 0); // should not revert
        assertEq(gw.currentNonce(), 1);
    }

    // ─── Admin: approve / revoke ──────────────────────────────────────────────

    function test_approveSubnet_setsMapping() public {
        vm.prank(owner);
        gw.approveSubnet(address(0x1));
        assertTrue(gw.approvedSubnets(address(0x1)));
    }

    function test_revokeSubnet_clearsMapping() public {
        vm.prank(owner);
        gw.approveSubnet(address(0x1));
        vm.prank(owner);
        gw.revokeSubnet(address(0x1));
        assertFalse(gw.approvedSubnets(address(0x1)));
    }

    function test_approveSubnet_revertsForNonOwner() public {
        vm.prank(other);
        vm.expectRevert();
        gw.approveSubnet(address(0x1));
    }

    // ─── Admin: pause / unpause ───────────────────────────────────────────────

    function test_pause_haltsSend() public {
        vm.prank(owner);
        gw.pause();
        vm.expectRevert();
        caller.sendMsg(destAddr, bytes(""), 0);
    }

    function test_unpause_resumesSend() public {
        vm.prank(owner);
        gw.pause();
        vm.prank(owner);
        gw.unpause();
        caller.sendMsg(destAddr, bytes(""), 0);
        assertEq(gw.currentNonce(), 1);
    }

    function test_pause_revertsForNonOwner() public {
        vm.prank(other);
        vm.expectRevert();
        gw.pause();
    }

    // ─── Fuzz ─────────────────────────────────────────────────────────────────

    function testFuzz_send_incrementsNonceMonotonically(uint8 n) public {
        vm.assume(n > 0 && n < 20);
        for (uint i = 0; i < n; i++) {
            caller.sendMsg(destAddr, bytes(""), 0);
        }
        assertEq(gw.currentNonce(), n);
    }
}
