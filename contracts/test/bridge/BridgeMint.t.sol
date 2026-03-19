// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import {IpcEnvelope, CallMsg, ResultMsg, IpcMsgKind, OutcomeType} from "../../contracts/structs/CrossNet.sol";
import {SubnetID, IPCAddress} from "../../contracts/structs/Subnet.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";

import {BridgeMint} from "../../contracts/bridge/BridgeMint.sol";
import {WrappedToken} from "../../contracts/bridge/WrappedToken.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

// ─────────────────────────────────────────────────────────────────────────────
// Mock gateway — delivers IPC messages to BridgeMint
// ─────────────────────────────────────────────────────────────────────────────
contract MockGatewayMint {
    function deliverIpcMessage(address target, IpcEnvelope calldata envelope) external payable {
        BridgeMint(target).handleIpcMessage{value: msg.value}(envelope);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BridgeMint test suite
// ─────────────────────────────────────────────────────────────────────────────
contract BridgeMintTest is Test {
    using FvmAddressHelper for address;

    BridgeMint public bridge;
    MockGatewayMint public gateway;
    WrappedToken public wrappedImpl;
    address public wrappedToken; // proxy

    address admin         = address(0xA11CE);
    address user          = address(0xB0B);
    address attacker      = address(0xBAD);
    address bridgeLockAddr = address(0xF11EC01E);
    address filecoinToken  = address(0xF11E10); // arbitrary Filecoin token addr

    SubnetID srcSubnet; // Filecoin Calibration
    bytes32 constant TRANSFER_ID_1 = keccak256("transfer-1");
    bytes32 constant TRANSFER_ID_2 = keccak256("transfer-2");

    // ─── Setup ───────────────────────────────────────────────────────────────

    function setUp() public {
        gateway = new MockGatewayMint();

        // Filecoin Calibration: chainid 314159, no extra route
        address[] memory route = new address[](0);
        srcSubnet = SubnetID({root: 314159, route: route});

        // Deploy BridgeMint implementation + proxy
        BridgeMint impl = new BridgeMint(address(gateway));
        bytes memory initData = abi.encodeWithSelector(
            BridgeMint.initialize.selector,
            admin,
            srcSubnet,
            bridgeLockAddr
        );
        ERC1967Proxy proxy = new ERC1967Proxy(address(impl), initData);
        bridge = BridgeMint(payable(address(proxy)));

        // Deploy WrappedToken impl + proxy; grant MINTER_ROLE to bridge
        wrappedImpl = new WrappedToken();
        bytes memory wtInit = abi.encodeWithSelector(
            WrappedToken.initialize.selector,
            "Wrapped Test Token",
            "wTT.ipc",
            admin
        );
        wrappedToken = address(new ERC1967Proxy(address(wrappedImpl), wtInit));

        // Grant MINTER_ROLE to bridge on the wrapped token
        vm.prank(admin);
        WrappedToken(wrappedToken).grantRole(keccak256("MINTER_ROLE"), address(bridge));

        // Register asset mapping
        vm.prank(admin);
        bridge.registerAsset(filecoinToken, wrappedToken);

        vm.deal(admin, 10 ether);
        vm.deal(user, 10 ether);
    }

    // ─── Helpers ─────────────────────────────────────────────────────────────

    function _makeLockEnvelope(
        address fToken,
        address recipient,
        uint256 amount,
        bytes32 transferId,
        address fromAddr,
        SubnetID memory fromSubnet
    ) internal pure returns (IpcEnvelope memory) {
        bytes memory params = abi.encode(fToken, recipient, amount, transferId);
        bytes memory method = abi.encodePacked(
            bytes4(keccak256("handleBridgeLock(address,address,uint256,bytes32)"))
        );
        CallMsg memory callMsg = CallMsg({method: method, params: params});

        return IpcEnvelope({
            kind: IpcMsgKind.Call,
            localNonce: 1,
            originalNonce: 1,
            value: 0,
            to: IPCAddress({
                subnetId: SubnetID({root: 0, route: new address[](0)}),
                rawAddress: FvmAddressHelper.from(address(0))
            }),
            from: IPCAddress({
                subnetId: fromSubnet,
                rawAddress: FvmAddressHelper.from(fromAddr)
            }),
            message: abi.encode(callMsg)
        });
    }

    function _validEnvelope(
        address recipient,
        uint256 amount,
        bytes32 transferId
    ) internal view returns (IpcEnvelope memory) {
        return _makeLockEnvelope(filecoinToken, recipient, amount, transferId, bridgeLockAddr, srcSubnet);
    }

    function _deliverValid(address recipient, uint256 amount, bytes32 transferId) internal {
        IpcEnvelope memory env = _validEnvelope(recipient, amount, transferId);
        vm.prank(address(gateway));
        bridge.handleIpcMessage(env);
    }

    // ─── Initialization ──────────────────────────────────────────────────────

    function test_initialize_setsAdmin() public {
        assertTrue(bridge.hasRole(bridge.DEFAULT_ADMIN_ROLE(), admin));
    }

    function test_initialize_setsPauserRole() public {
        assertTrue(bridge.hasRole(bridge.PAUSER_ROLE(), admin));
    }

    function test_initialize_setsBridgeLockOrigin() public {
        assertEq(bridge.bridgeLockAddr(), bridgeLockAddr);
    }

    function test_initialize_revertsZeroAdmin() public {
        BridgeMint impl2 = new BridgeMint(address(gateway));
        bytes memory data = abi.encodeWithSelector(
            BridgeMint.initialize.selector, address(0), srcSubnet, bridgeLockAddr
        );
        vm.expectRevert(BridgeMint.ZeroAddress.selector);
        new ERC1967Proxy(address(impl2), data);
    }

    function test_initialize_revertsZeroBridgeLock() public {
        BridgeMint impl2 = new BridgeMint(address(gateway));
        bytes memory data = abi.encodeWithSelector(
            BridgeMint.initialize.selector, admin, srcSubnet, address(0)
        );
        vm.expectRevert(BridgeMint.ZeroAddress.selector);
        new ERC1967Proxy(address(impl2), data);
    }

    // ─── Mint: happy path ─────────────────────────────────────────────────────

    function test_mint_emitsTokensMinted() public {
        vm.expectEmit(true, true, true, true);
        emit BridgeMint.TokensMinted(wrappedToken, user, 100 ether, TRANSFER_ID_1);
        _deliverValid(user, 100 ether, TRANSFER_ID_1);
    }

    function test_mint_creditsMintedTokens() public {
        _deliverValid(user, 50 ether, TRANSFER_ID_1);
        assertEq(WrappedToken(wrappedToken).balanceOf(user), 50 ether);
    }

    function test_mint_recordsTransferId() public {
        _deliverValid(user, 50 ether, TRANSFER_ID_1);
        assertTrue(bridge.isProcessed(TRANSFER_ID_1));
    }

    function test_mint_multipleDifferentTransfers() public {
        _deliverValid(user, 50 ether, TRANSFER_ID_1);
        _deliverValid(user, 30 ether, TRANSFER_ID_2);
        assertEq(WrappedToken(wrappedToken).balanceOf(user), 80 ether);
        assertTrue(bridge.isProcessed(TRANSFER_ID_1));
        assertTrue(bridge.isProcessed(TRANSFER_ID_2));
    }

    // ─── Replay protection ────────────────────────────────────────────────────

    function test_mint_rejectsReplay() public {
        _deliverValid(user, 50 ether, TRANSFER_ID_1);
        // Second delivery of same transferId must revert
        IpcEnvelope memory env = _validEnvelope(user, 50 ether, TRANSFER_ID_1);
        vm.prank(address(gateway));
        vm.expectRevert(abi.encodeWithSelector(BridgeMint.DuplicateTransfer.selector, TRANSFER_ID_1));
        bridge.handleIpcMessage(env);
    }

    function testFuzz_mint_replayProtection(bytes32 tid, uint128 amount) public {
        vm.assume(amount > 0);
        _deliverValid(user, amount, tid);
        assertTrue(bridge.isProcessed(tid));
        // Replay must revert
        IpcEnvelope memory env = _validEnvelope(user, amount, tid);
        vm.prank(address(gateway));
        vm.expectRevert(abi.encodeWithSelector(BridgeMint.DuplicateTransfer.selector, tid));
        bridge.handleIpcMessage(env);
    }

    // ─── Access control: gateway-only ─────────────────────────────────────────

    function test_mint_rejectsDirectCallerNotGateway() public {
        IpcEnvelope memory env = _validEnvelope(user, 50 ether, TRANSFER_ID_1);
        // Direct call from attacker (not gateway) must revert
        vm.prank(attacker);
        vm.expectRevert();
        bridge.handleIpcMessage(env);
    }

    function test_mint_rejectsWrongOriginAddress() public {
        // Correct subnet, wrong BridgeLock address
        IpcEnvelope memory env = _makeLockEnvelope(
            filecoinToken, user, 50 ether, TRANSFER_ID_1,
            attacker,        // wrong addr
            srcSubnet
        );
        vm.prank(address(gateway));
        vm.expectRevert(BridgeMint.UnauthorizedOrigin.selector);
        bridge.handleIpcMessage(env);
    }

    function test_mint_rejectsWrongOriginSubnet() public {
        // Correct address, wrong subnet
        address[] memory route = new address[](0);
        SubnetID memory wrongSubnet = SubnetID({root: 1, route: route}); // Ethereum mainnet root
        IpcEnvelope memory env = _makeLockEnvelope(
            filecoinToken, user, 50 ether, TRANSFER_ID_1,
            bridgeLockAddr,
            wrongSubnet
        );
        vm.prank(address(gateway));
        vm.expectRevert(BridgeMint.UnauthorizedOrigin.selector);
        bridge.handleIpcMessage(env);
    }

    function test_mint_rejectsUnknownMethod() public {
        bytes memory badMethod = abi.encodePacked(bytes4(keccak256("badMethod()")));
        bytes memory params = abi.encode(filecoinToken, user, 50 ether, TRANSFER_ID_1);
        CallMsg memory callMsg = CallMsg({method: badMethod, params: params});

        IpcEnvelope memory env = IpcEnvelope({
            kind: IpcMsgKind.Call,
            localNonce: 1,
            originalNonce: 1,
            value: 0,
            to: IPCAddress({
                subnetId: SubnetID({root: 0, route: new address[](0)}),
                rawAddress: FvmAddressHelper.from(address(0))
            }),
            from: IPCAddress({subnetId: srcSubnet, rawAddress: FvmAddressHelper.from(bridgeLockAddr)}),
            message: abi.encode(callMsg)
        });
        vm.prank(address(gateway));
        vm.expectRevert();
        bridge.handleIpcMessage(env);
    }

    // ─── Access control: unregistered asset ──────────────────────────────────

    function test_mint_rejectsUnregisteredAsset() public {
        address unknownToken = address(0xDEAD);
        IpcEnvelope memory env = _makeLockEnvelope(
            unknownToken, user, 50 ether, TRANSFER_ID_1, bridgeLockAddr, srcSubnet
        );
        vm.prank(address(gateway));
        vm.expectRevert(abi.encodeWithSelector(BridgeMint.AssetNotRegistered.selector, unknownToken));
        bridge.handleIpcMessage(env);
    }

    // ─── Pause ───────────────────────────────────────────────────────────────

    function test_pause_haltsMintig() public {
        vm.prank(admin);
        bridge.pause();

        IpcEnvelope memory env = _validEnvelope(user, 50 ether, TRANSFER_ID_1);
        vm.prank(address(gateway));
        vm.expectRevert();
        bridge.handleIpcMessage(env);
    }

    function test_unpause_resumesMinting() public {
        vm.prank(admin);
        bridge.pause();
        vm.prank(admin);
        bridge.unpause();

        _deliverValid(user, 50 ether, TRANSFER_ID_1);
        assertEq(WrappedToken(wrappedToken).balanceOf(user), 50 ether);
    }

    function test_pause_revertsForNonPauser() public {
        vm.prank(attacker);
        vm.expectRevert();
        bridge.pause();
    }

    // ─── Admin: registerAsset ─────────────────────────────────────────────────

    function test_registerAsset_setsMapping() public {
        address newFtok = address(0xFEED);
        address newWtok = address(0xBEEF);
        vm.prank(admin);
        bridge.registerAsset(newFtok, newWtok);
        assertEq(bridge.getWrappedToken(newFtok), newWtok);
    }

    function test_registerAsset_revertsNonAdmin() public {
        vm.prank(attacker);
        vm.expectRevert();
        bridge.registerAsset(address(0x1), address(0x2));
    }

    function test_registerAsset_revertsZeroFilecoinToken() public {
        vm.prank(admin);
        vm.expectRevert(BridgeMint.ZeroAddress.selector);
        bridge.registerAsset(address(0), address(0x1));
    }

    function test_registerAsset_revertsZeroWrapped() public {
        vm.prank(admin);
        vm.expectRevert(BridgeMint.ZeroAddress.selector);
        bridge.registerAsset(address(0x1), address(0));
    }

    // ─── Admin: deployAndRegisterAsset ────────────────────────────────────────

    function test_deployAndRegisterAsset_deploysAndRegisters() public {
        address newFtok = address(0xFEED);
        vm.prank(admin);
        address deployed = bridge.deployAndRegisterAsset(newFtok, "New Token", "NT.ipc", address(wrappedImpl));

        assertEq(bridge.getWrappedToken(newFtok), deployed);
        // Check minting works via the bridge
        // Grant was done in deployAndRegisterAsset (admin = address(bridge))
        vm.prank(admin);
        bridge.registerAsset(newFtok, deployed); // re-register (already registered)
        // Deliver a mint to verify the full flow
        bytes32 tid = keccak256("new-asset-transfer");
        IpcEnvelope memory env = _makeLockEnvelope(newFtok, user, 10 ether, tid, bridgeLockAddr, srcSubnet);
        vm.prank(address(gateway));
        bridge.handleIpcMessage(env);
        assertEq(WrappedToken(deployed).balanceOf(user), 10 ether);
    }

    // ─── Admin: setBridgeLockOrigin ───────────────────────────────────────────

    function test_setBridgeLockOrigin_updates() public {
        address newLock = address(0x9999);
        vm.prank(admin);
        bridge.setBridgeLockOrigin(srcSubnet, newLock);
        assertEq(bridge.bridgeLockAddr(), newLock);
    }

    function test_setBridgeLockOrigin_revertsNonAdmin() public {
        vm.prank(attacker);
        vm.expectRevert();
        bridge.setBridgeLockOrigin(srcSubnet, address(0x9999));
    }

    // ─── UUPS upgrade ─────────────────────────────────────────────────────────

    function test_upgrade_onlyAdmin() public {
        BridgeMint impl2 = new BridgeMint(address(gateway));
        vm.prank(attacker);
        vm.expectRevert();
        bridge.upgradeToAndCall(address(impl2), bytes(""));

        vm.prank(admin);
        bridge.upgradeToAndCall(address(impl2), bytes(""));
    }

    // ─── WrappedToken standalone ──────────────────────────────────────────────

    function test_wrappedToken_mintAndBurn() public {
        vm.prank(address(bridge)); // bridge has MINTER_ROLE
        WrappedToken(wrappedToken).mint(user, 100 ether);
        assertEq(WrappedToken(wrappedToken).balanceOf(user), 100 ether);

        vm.prank(address(bridge));
        WrappedToken(wrappedToken).burn(user, 40 ether);
        assertEq(WrappedToken(wrappedToken).balanceOf(user), 60 ether);
    }

    function test_wrappedToken_rejectsMintWithoutRole() public {
        vm.prank(attacker);
        vm.expectRevert();
        WrappedToken(wrappedToken).mint(attacker, 100 ether);
    }

    // ─── Fuzz: various amounts ─────────────────────────────────────────────────

    function testFuzz_mint_variousAmounts(uint128 amount) public {
        vm.assume(amount > 0);
        _deliverValid(user, amount, TRANSFER_ID_1);
        assertEq(WrappedToken(wrappedToken).balanceOf(user), amount);
    }

    function testFuzz_mint_multipleRecipients(address recipient, uint64 amount) public {
        vm.assume(recipient != address(0) && amount > 0);
        vm.assume(recipient.code.length == 0); // skip contracts that can't receive
        bytes32 tid = keccak256(abi.encodePacked(recipient, amount));
        IpcEnvelope memory env = _validEnvelope(recipient, amount, tid);
        vm.prank(address(gateway));
        bridge.handleIpcMessage(env);
        assertEq(WrappedToken(wrappedToken).balanceOf(recipient), amount);
    }
}
