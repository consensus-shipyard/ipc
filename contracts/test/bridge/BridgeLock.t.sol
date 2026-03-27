// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {IpcEnvelope, CallMsg, ResultMsg, IpcMsgKind, OutcomeType} from "../../contracts/structs/CrossNet.sol";
import {SubnetID, IPCAddress} from "../../contracts/structs/Subnet.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {IGateway} from "../../contracts/interfaces/IGateway.sol";

import {BridgeLock} from "../../contracts/bridge/BridgeLock.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {ERC20Mock} from "../mocks/ERC20Mock.sol";

// ─────────────────────────────────────────────────────────────────────────────
// Mock gateway — records the last xnet message sent
// ─────────────────────────────────────────────────────────────────────────────
contract MockGateway {
    // Store internally (not as public auto-getter — IpcEnvelope has nested structs
    // with dynamic arrays that Solidity can't auto-generate getters for).
    IpcEnvelope private _lastEnvelope;
    bool public shouldRevert;

    function sendContractXnetMessage(
        IpcEnvelope calldata envelope
    ) external payable returns (IpcEnvelope memory committed) {
        require(!shouldRevert, "MockGateway: forced revert");
        _lastEnvelope = envelope;
        committed = envelope;
        committed.localNonce = 1;
    }

    /// @notice Manual getter so tests can retrieve the full struct.
    function lastEnvelope() external view returns (IpcEnvelope memory) {
        return _lastEnvelope;
    }

    function setShouldRevert(bool v) external { shouldRevert = v; }

    // Stub unused IGateway functions
    function register(uint256, uint256) external payable {}
    function addStake(uint256) external payable {}
    function releaseStake(uint256) external {}
    function kill() external {}
}

// ─────────────────────────────────────────────────────────────────────────────
// Minimal ERC20 mock (if not already in test/mocks)
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// BridgeLock test suite
// ─────────────────────────────────────────────────────────────────────────────
contract BridgeLockTest is Test {
    using FvmAddressHelper for address;

    BridgeLock public impl;
    BridgeLock public bridge;       // proxy, cast to BridgeLock
    MockGateway public gateway;
    ERC20Mock public token;

    address admin    = address(0xA11CE);
    address user     = address(0xB0B);
    address relayer  = address(0xC0DE);
    address receiver = address(0xDEAD); // BridgeMint on dest chain

    SubnetID destSubnet;
    uint256 constant IPC_FEE = 0.01 ether;

    // ─── Setup ───────────────────────────────────────────────────────────────

    function setUp() public {
        gateway = new MockGateway();
        token   = new ERC20Mock("TestToken", "TT");

        // Destination subnet: Ethereum Sepolia (chainid=11155111)
        address[] memory route = new address[](0);
        destSubnet = SubnetID({root: 11155111, route: route});

        // Deploy implementation + UUPS proxy
        impl = new BridgeLock(address(gateway));

        bytes memory initData = abi.encodeWithSelector(
            BridgeLock.initialize.selector,
            admin,
            destSubnet,
            receiver,
            IPC_FEE
        );
        ERC1967Proxy proxy = new ERC1967Proxy(address(impl), initData);
        bridge = BridgeLock(payable(address(proxy)));

        // Fund user with tokens and ETH
        token.mint(user, 1_000 ether);
        vm.deal(user, 10 ether);
        vm.deal(admin, 10 ether);
    }

    // ─── Initialization ──────────────────────────────────────────────────────

    function test_initialize_setsAdmin() public {
        assertTrue(bridge.hasRole(bridge.DEFAULT_ADMIN_ROLE(), admin));
    }

    function test_initialize_setsPauserRole() public {
        assertTrue(bridge.hasRole(bridge.PAUSER_ROLE(), admin));
    }

    function test_initialize_setsDestination() public {
        assertEq(bridge.destReceiver(), receiver);
        assertEq(bridge.ipcFee(), IPC_FEE);
    }

    function test_initialize_revertsIfAdminZero() public {
        BridgeLock impl2 = new BridgeLock(address(gateway));
        address[] memory route = new address[](0);
        SubnetID memory sid = SubnetID({root: 1, route: route});
        bytes memory data = abi.encodeWithSelector(
            BridgeLock.initialize.selector,
            address(0), sid, receiver, IPC_FEE
        );
        vm.expectRevert(BridgeLock.ZeroAddress.selector);
        new ERC1967Proxy(address(impl2), data);
    }

    function test_initialize_revertsIfReceiverZero() public {
        BridgeLock impl2 = new BridgeLock(address(gateway));
        address[] memory route = new address[](0);
        SubnetID memory sid = SubnetID({root: 1, route: route});
        bytes memory data = abi.encodeWithSelector(
            BridgeLock.initialize.selector,
            admin, sid, address(0), IPC_FEE
        );
        vm.expectRevert(BridgeLock.ZeroAddress.selector);
        new ERC1967Proxy(address(impl2), data);
    }

    // ─── lock() happy path ───────────────────────────────────────────────────

    function test_lock_emitsTokensLocked() public {
        uint256 amount = 100 ether;
        address recipient = address(0xFACE);

        vm.startPrank(user);
        token.approve(address(bridge), amount);

        vm.expectEmit(true, true, true, false);
        emit BridgeLock.TokensLocked(address(token), user, recipient, amount, bytes32(0));

        bridge.lock{value: IPC_FEE}(address(token), amount, recipient);
        vm.stopPrank();
    }

    function test_lock_transfersTokensToBridge() public {
        uint256 amount = 100 ether;
        vm.startPrank(user);
        token.approve(address(bridge), amount);
        bridge.lock{value: IPC_FEE}(address(token), amount, address(0xFACE));
        vm.stopPrank();

        assertEq(token.balanceOf(address(bridge)), amount);
        assertEq(token.balanceOf(user), 900 ether);
    }

    function test_lock_recordsTransferId() public {
        uint256 amount = 50 ether;
        vm.startPrank(user);
        token.approve(address(bridge), amount);
        // Capture the emitted transferId via event recording
        vm.recordLogs();
        bridge.lock{value: IPC_FEE}(address(token), amount, address(0xFACE));
        vm.stopPrank();

        Vm.Log[] memory logs = vm.getRecordedLogs();
        // TokensLocked is the first event; topic[4] is transferId (non-indexed param in struct)
        // Find the TokensLocked event
        bytes32 sig = keccak256("TokensLocked(address,address,address,uint256,bytes32)");
        bytes32 transferId;
        for (uint i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == sig) {
                (uint256 amt, bytes32 tid) = abi.decode(logs[i].data, (uint256, bytes32));
                transferId = tid;
                break;
            }
        }
        assertTrue(bridge.isProcessed(transferId), "transferId should be recorded");
    }

    function test_lock_sendsIpcMessage() public {
        uint256 amount = 100 ether;
        address recipient = address(0xFACE);
        vm.startPrank(user);
        token.approve(address(bridge), amount);
        bridge.lock{value: IPC_FEE}(address(token), amount, recipient);
        vm.stopPrank();

        IpcEnvelope memory env = gateway.lastEnvelope();
        assertEq(uint8(env.kind), uint8(IpcMsgKind.Call));
        assertEq(env.value, IPC_FEE);

        CallMsg memory call = abi.decode(env.message, (CallMsg));
        (address t, address r, uint256 a, bytes32 _tid) = abi.decode(call.params, (address, address, uint256, bytes32));
        assertEq(t, address(token));
        assertEq(r, recipient);
        assertEq(a, amount);
    }

    function test_lock_incrementsNonce() public {
        address recipient = address(0xFACE);
        vm.startPrank(user);
        token.approve(address(bridge), 200 ether);

        vm.recordLogs();
        bridge.lock{value: IPC_FEE}(address(token), 50 ether, recipient);
        bridge.lock{value: IPC_FEE}(address(token), 50 ether, recipient);
        vm.stopPrank();

        bytes32 sig = keccak256("TokensLocked(address,address,address,uint256,bytes32)");
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bytes32[] memory ids = new bytes32[](2);
        uint idx = 0;
        for (uint i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == sig) {
                (, bytes32 tid) = abi.decode(logs[i].data, (uint256, bytes32));
                ids[idx++] = tid;
                if (idx == 2) break;
            }
        }
        assertTrue(ids[0] != ids[1], "transferIds must be unique");
    }

    // ─── lock() revert cases ─────────────────────────────────────────────────

    function test_lock_revertsOnZeroAmount() public {
        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        vm.expectRevert(BridgeLock.ZeroAmount.selector);
        bridge.lock{value: IPC_FEE}(address(token), 0, address(0xFACE));
        vm.stopPrank();
    }

    function test_lock_revertsOnZeroToken() public {
        vm.startPrank(user);
        vm.expectRevert(BridgeLock.ZeroAddress.selector);
        bridge.lock{value: IPC_FEE}(address(0), 100 ether, address(0xFACE));
        vm.stopPrank();
    }

    function test_lock_revertsOnZeroRecipient() public {
        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        vm.expectRevert(BridgeLock.ZeroAddress.selector);
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0));
        vm.stopPrank();
    }

    function test_lock_revertsOnInsufficientFee() public {
        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        vm.expectRevert(
            abi.encodeWithSelector(BridgeLock.InsufficientMsgValue.selector, IPC_FEE, IPC_FEE - 1)
        );
        bridge.lock{value: IPC_FEE - 1}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();
    }

    function test_lock_revertsWhenPaused() public {
        vm.prank(admin);
        bridge.pause();

        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        vm.expectRevert();
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();
    }

    function test_lock_revertsOnDisallowedToken() public {
        vm.prank(admin);
        bridge.setTokenAllowlistEnabled(true);
        // token NOT in allowlist

        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        vm.expectRevert(abi.encodeWithSelector(BridgeLock.TokenNotAllowed.selector, address(token)));
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();
    }

    function test_lock_allowsListedToken() public {
        vm.startPrank(admin);
        bridge.setTokenAllowlistEnabled(true);
        bridge.setTokenAllowed(address(token), true);
        vm.stopPrank();

        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();

        assertEq(token.balanceOf(address(bridge)), 100 ether);
    }

    // ─── Pause / Unpause ─────────────────────────────────────────────────────

    function test_pause_onlyPauserRole() public {
        vm.expectRevert();
        vm.prank(user);
        bridge.pause();
    }

    function test_unpause_resumesLock() public {
        vm.prank(admin);
        bridge.pause();
        vm.prank(admin);
        bridge.unpause();

        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();
        assertEq(token.balanceOf(address(bridge)), 100 ether);
    }

    // ─── Admin: setDestination ────────────────────────────────────────────────

    function test_setDestination_updatesState() public {
        address newReceiver = address(0x9999);
        address[] memory route = new address[](1);
        route[0] = address(0x1);
        SubnetID memory newSubnet = SubnetID({root: 1, route: route});

        vm.prank(admin);
        bridge.setDestination(newSubnet, newReceiver);

        assertEq(bridge.destReceiver(), newReceiver);
    }

    function test_setDestination_revertsForNonAdmin() public {
        address[] memory route = new address[](0);
        SubnetID memory sid = SubnetID({root: 1, route: route});
        vm.expectRevert();
        vm.prank(user);
        bridge.setDestination(sid, address(0x9999));
    }

    function test_setDestination_revertsOnZeroReceiver() public {
        address[] memory route = new address[](0);
        SubnetID memory sid = SubnetID({root: 1, route: route});
        vm.expectRevert(BridgeLock.ZeroAddress.selector);
        vm.prank(admin);
        bridge.setDestination(sid, address(0));
    }

    // ─── Admin: setIpcFee ─────────────────────────────────────────────────────

    function test_setIpcFee_updatesValue() public {
        vm.prank(admin);
        bridge.setIpcFee(0.05 ether);
        assertEq(bridge.ipcFee(), 0.05 ether);
    }

    function test_setIpcFee_revertsForNonAdmin() public {
        vm.expectRevert();
        vm.prank(user);
        bridge.setIpcFee(0.05 ether);
    }

    // ─── rescueTokens ─────────────────────────────────────────────────────────

    function test_rescueTokens_transfersOut() public {
        // First lock some tokens
        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();

        assertEq(token.balanceOf(address(bridge)), 100 ether);

        vm.prank(admin);
        bridge.rescueTokens(address(token), admin, 100 ether);
        assertEq(token.balanceOf(admin), 100 ether);
        assertEq(token.balanceOf(address(bridge)), 0);
    }

    function test_rescueTokens_revertsForNonAdmin() public {
        vm.expectRevert();
        vm.prank(user);
        bridge.rescueTokens(address(token), user, 1 ether);
    }

    function test_rescueTokens_revertsOnZeroTo() public {
        vm.expectRevert(BridgeLock.ZeroAddress.selector);
        vm.prank(admin);
        bridge.rescueTokens(address(token), address(0), 1 ether);
    }

    // ─── UUPS upgrade ─────────────────────────────────────────────────────────

    function test_upgrade_onlyAdmin() public {
        BridgeLock impl2 = new BridgeLock(address(gateway));

        // Non-admin cannot upgrade
        vm.expectRevert();
        vm.prank(user);
        bridge.upgradeToAndCall(address(impl2), bytes(""));

        // Admin can upgrade
        vm.prank(admin);
        bridge.upgradeToAndCall(address(impl2), bytes(""));
    }

    // ─── IpcExchange: handleIpcMessage (result receipt) ──────────────────────

    function test_handleResult_emitsAcknowledged() public {
        // First perform a lock to create an inflight message
        vm.startPrank(user);
        token.approve(address(bridge), 100 ether);
        bridge.lock{value: IPC_FEE}(address(token), 100 ether, address(0xFACE));
        vm.stopPrank();

        // The gateway recorded the envelope; simulate a result coming back via gateway
        IpcEnvelope memory sent = gateway.lastEnvelope();
        // Give it the tracing id that IpcExchange would have stored
        bytes32 id = CrossMsgHelper.toTracingId(sent);

        ResultMsg memory result = ResultMsg({
            id: id,
            outcome: OutcomeType.Ok,
            ret: bytes("")
        });

        IpcEnvelope memory resultEnvelope = IpcEnvelope({
            kind: IpcMsgKind.Result,
            localNonce: 2,
            originalNonce: sent.originalNonce,
            value: 0,
            to: sent.from,
            from: sent.to,
            message: abi.encode(result)
        });

        // Gateway delivers result to bridge
        vm.prank(address(gateway));
        vm.expectEmit(false, false, false, false);
        emit BridgeLock.TransferAcknowledged(bytes32(0), true, bytes(""));
        bridge.handleIpcMessage(resultEnvelope);
    }

    // ─── Fuzz tests ──────────────────────────────────────────────────────────

    function testFuzz_lock_variousAmounts(uint128 amount) public {
        vm.assume(amount > 0 && amount <= 1000 ether);
        token.mint(user, uint256(amount));

        vm.startPrank(user);
        token.approve(address(bridge), amount);
        bridge.lock{value: IPC_FEE}(address(token), amount, address(0xFACE));
        vm.stopPrank();

        assertEq(token.balanceOf(address(bridge)), amount);
    }

    function testFuzz_lock_uniqueTransferIds(uint8 n) public {
        vm.assume(n > 1 && n < 20);
        uint256 perLock = 10 ether;
        token.mint(user, uint256(n) * perLock);

        vm.startPrank(user);
        token.approve(address(bridge), uint256(n) * perLock);

        bytes32[] memory ids = new bytes32[](n);
        for (uint i = 0; i < n; i++) {
            vm.recordLogs();
            bridge.lock{value: IPC_FEE}(address(token), perLock, address(0xFACE));
            Vm.Log[] memory logs = vm.getRecordedLogs();
            bytes32 sig = keccak256("TokensLocked(address,address,address,uint256,bytes32)");
            for (uint j = 0; j < logs.length; j++) {
                if (logs[j].topics[0] == sig) {
                    (, bytes32 tid) = abi.decode(logs[j].data, (uint256, bytes32));
                    ids[i] = tid;
                }
            }
        }
        vm.stopPrank();

        // All IDs must be unique
        for (uint i = 0; i < n; i++) {
            for (uint j = i + 1; j < n; j++) {
                assertTrue(ids[i] != ids[j], "duplicate transferId detected");
            }
        }
    }
}
