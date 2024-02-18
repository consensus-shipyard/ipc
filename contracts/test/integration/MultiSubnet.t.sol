// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../src/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, IpcMsgKind, ResultMsg, CallMsg} from "../../src/structs/CrossNet.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../../src/structs/Subnet.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../src/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../src/lib/CrossMsgHelper.sol";
import {GatewayDiamond, FEATURE_MULTILEVEL_CROSSMSG} from "../../src/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../../src/SubnetActorDiamond.sol";
import {SubnetActorGetterFacet} from "../../src/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorManagerFacet} from "../../src/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../src/subnet/SubnetActorCheckpointingFacet.sol";
import {GatewayGetterFacet} from "../../src/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../src/gateway/GatewayManagerFacet.sol";
import {LibGateway} from "../../src/lib/LibGateway.sol";
import {TopDownFinalityFacet} from "../../src/gateway/router/TopDownFinalityFacet.sol";
import {CheckpointingFacet} from "../../src/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../src/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../src/gateway/GatewayMessengerFacet.sol";
import {DiamondLoupeFacet} from "../../src/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";

import {IpcTokenReplica} from "../../src/examples/cross-token/IpcTokenReplica.sol";
import {IpcTokenController} from "../../src/examples/cross-token/IpcTokenController.sol";
import {USDCTest} from "../../src/examples/cross-token/USDCTest.sol";
import {IpcHandler, IpcExchange} from "../../sdk/IpcContract.sol";

import {IntegrationTestBase} from "../IntegrationTestBase.sol";
import {L2GatewayActorDiamond, L1GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractFallback} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";
import {IERC20Errors} from "openzeppelin-contracts/interfaces/draft-IERC6093.sol";

import "forge-std/console.sol";

contract MultiSubnetTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;

    GatewayDiamond public rootGateway;
    GatewayGetterFacet public rootGatewayGetter;
    GatewayManagerFacet public rootGatewayManager;

    SubnetActorDiamond public rootNativeSubnetActor;
    SubnetActorDiamond public rootTokenSubnetActor;

    GatewayDiamond public tokenSubnetGateway;
    GatewayDiamond public nativeSubnetGateway;

    address[] public nativeSubnetPath;
    address[] public tokenSubnetPath;

    SubnetID rootSubnetName;
    SubnetID nativeSubnetName;
    SubnetID tokenSubnetName;

    IERC20 public token;

    IpcTokenReplica ipcTokenReplica;
    IpcTokenController ipcTokenController;

    function setUp() public override {
        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1 ether, address(this));

        rootSubnetName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(rootSubnetName.isRoot(), "not root");

        rootGateway = createGatewayDiamond(gatewayParams(rootSubnetName));
        rootGatewayGetter = GatewayGetterFacet(address(rootGateway));
        rootGatewayManager = GatewayManagerFacet(address(rootGateway));

        rootNativeSubnetActor = createSubnetActor(subnetActorWithParams(address(rootGateway), rootSubnetName));

        rootTokenSubnetActor = createSubnetActor(
            subnetActorWithParams(address(rootGateway), rootSubnetName, address(token))
        );

        tokenSubnetPath = new address[](1);
        tokenSubnetPath[0] = address(rootTokenSubnetActor);
        tokenSubnetName = SubnetID({root: ROOTNET_CHAINID, route: tokenSubnetPath});
        tokenSubnetGateway = createGatewayDiamond(gatewayParams(tokenSubnetName));

        nativeSubnetPath = new address[](1);
        nativeSubnetPath[0] = address(rootNativeSubnetActor);
        nativeSubnetName = SubnetID({root: ROOTNET_CHAINID, route: nativeSubnetPath});
        nativeSubnetGateway = createGatewayDiamond(gatewayParams(nativeSubnetName));

        printActors();
    }

    //--------------------
    // Fund flow tests.
    //---------------------

    function testMultiSubnet_Native_FundingFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootNativeSubnetActor), expected);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(recipient)));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        // TODO: commitParentFinality doesn't not affect anything in this test.
        commitParentFinality(address(nativeSubnetGateway));

        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Native_NonPayable_FundingFromParentToChildFails() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            nativeSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootNativeSubnetActor), expected);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(recipient)));

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        // TODO: commitParentFinality doesn't not affect anything in this test.
        commitParentFinality(address(nativeSubnetGateway));

        vm.expectRevert();
        executeTopDownMsgsRevert(msgs, tokenSubnetName, address(tokenSubnetGateway));
    }

    function testMultiSubnet_Erc20_FundingFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootTokenSubnetActor), expected);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(recipient)), amount);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        // TODO: commitParentFinality doesn not affect anything in this test.
        commitParentFinality(address(tokenSubnetGateway));

        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Erc20_NonPayable_FundingFromParentToChildFails() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, amount);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        IpcEnvelope memory expected = CrossMsgHelper.createFundMsg(
            tokenSubnetName,
            caller,
            FvmAddressHelper.from(recipient),
            amount
        );

        vm.prank(caller);
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage(address(rootTokenSubnetActor), expected);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(recipient)), amount);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = expected;

        commitParentFinality(address(tokenSubnetGateway));

        vm.expectRevert();
        executeTopDownMsgsRevert(msgs, tokenSubnetName, address(tokenSubnetGateway));
    }

    //--------------------
    // Release flow tests.
    //---------------------

    function testMultiSubnet_Native_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));

        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));
        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Native_NonPayable_ReleaseFromChildToParentFails() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 6);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        vm.expectRevert();
        submitBottomUpCheckpointRevert(checkpoint, address(rootNativeSubnetActor));
    }

    function testMultiSubnet_Native_ReleaseFromChildToParent_DifferentFunderAndSenderInParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 6);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: amount}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayManagerFacet manager = GatewayManagerFacet(address(nativeSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Erc20_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);

        GatewayManagerFacet manager = GatewayManagerFacet(address(tokenSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), amount);
    }

    function testMultiSubnet_Erc20_Transfer_NonPayable_ReleaseFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractFallback());
        uint256 amount = 3;

        token.transfer(caller, amount);
        vm.prank(caller);
        token.approve(address(rootGateway), amount);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), amount);

        GatewayManagerFacet manager = GatewayManagerFacet(address(tokenSubnetGateway));
        vm.prank(caller);
        manager.release{value: amount}(FvmAddressHelper.from(address(recipient)));

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));
        assertEq(token.balanceOf(recipient), amount);
        assertEq(recipient.balance, 0);
    }

    //--------------------
    // Call flow tests.
    //---------------------

    function testMultiSubnet_Native_SendCrossMessageFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: 100000}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        GatewayMessengerFacet messenger = GatewayMessengerFacet(address(nativeSubnetGateway));
        vm.prank(address(caller));
        messenger.sendContractXnetMessage{value: amount}(
            TestUtils.newXnetCallMsg(
                IPCAddress({subnetId: nativeSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
                IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(recipient)}),
                amount,
                0
            )
        );

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        assertEq(recipient.balance, amount);
    }

    function testMultiSubnet_Native_SendCrossMessageFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fund{value: 100000}(nativeSubnetName, FvmAddressHelper.from(address(caller)));

        IpcEnvelope memory xnetCallMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: nativeSubnetName, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        IpcEnvelope memory committedEvent = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            to: xnetCallMsg.to,
            value: xnetCallMsg.value,
            message: xnetCallMsg.message,
            nonce: 1
        });

        GatewayMessengerFacet rootGatewayMessenger = GatewayMessengerFacet(address(rootGateway));
        vm.prank(address(caller));
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage({subnet: address(rootNativeSubnetActor), message: committedEvent});
        rootGatewayMessenger.sendContractXnetMessage{value: amount}(xnetCallMsg);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = xnetCallMsg;

        commitParentFinality(address(nativeSubnetGateway));
        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        assertEq(address(recipient).balance, amount);
    }

    function testMultiSubnet_Token_CallFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(address(token), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        // Fund an account in the subnet.
        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), 15);

        IPCAddress memory from = IPCAddress({subnetId: tokenSubnetName, rawAddress: FvmAddressHelper.from(caller)});
        IPCAddress memory to = IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(recipient)});
        bytes4 method = bytes4(0x11223344);
        bytes memory params = bytes("hello");
        IpcEnvelope memory envelope = CrossMsgHelper.createCallMsg(from, to, amount, method, params);

        GatewayMessengerFacet messenger = GatewayMessengerFacet(address(tokenSubnetGateway));
        vm.prank(address(caller));
        messenger.sendContractXnetMessage{value: amount}(envelope);

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            tokenSubnetName,
            address(tokenSubnetGateway)
        );

        submitBottomUpCheckpoint(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), amount);
    }

    function testMultiSubnet_Erc20_SendCrossMessageFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 3);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.prank(caller);
        rootGatewayManager.fundWithToken(tokenSubnetName, FvmAddressHelper.from(address(caller)), 15);

        IpcEnvelope memory xnetCallMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: tokenSubnetName, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        IpcEnvelope memory committedEvent = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: rootSubnetName, rawAddress: FvmAddressHelper.from(caller)}),
            to: xnetCallMsg.to,
            value: xnetCallMsg.value,
            message: xnetCallMsg.message,
            nonce: 1
        });

        GatewayMessengerFacet rootGatewayMessenger = GatewayMessengerFacet(address(rootGateway));
        vm.prank(address(caller));
        vm.expectEmit(true, true, true, true, address(rootGateway));
        emit LibGateway.NewTopDownMessage({subnet: address(rootTokenSubnetActor), message: committedEvent});
        rootGatewayMessenger.sendContractXnetMessage{value: amount}(xnetCallMsg);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = xnetCallMsg;

        commitParentFinality(address(tokenSubnetGateway));
        executeTopDownMsgs(msgs, tokenSubnetName, address(tokenSubnetGateway));

        assertEq(address(recipient).balance, amount);
    }

    function commitParentFinality(address gateway) internal {
        vm.roll(10);
        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        TopDownFinalityFacet gwTopDownFinalityFacet = TopDownFinalityFacet(address(gateway));

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gwTopDownFinalityFacet.commitParentFinality(finality);
    }

    function executeTopDownMsgs(IpcEnvelope[] memory msgs, SubnetID memory subnet, address gateway) internal {
        XnetMessagingFacet xnet = XnetMessagingFacet(address(gateway));

        uint256 minted_tokens;

        for (uint256 i; i < msgs.length; ) {
            minted_tokens += msgs[i].value;
            unchecked {
                ++i;
            }
        }
        console.log("minted tokens in executed top-downs: %d", minted_tokens);

        // The implementation of the function in fendermint is in
        // https://github.com/consensus-shipyard/ipc/blob/main/fendermint/vm/interpreter/src/fvm/topdown.rs#L43

        // This emulates minting tokens.
        vm.deal(address(gateway), minted_tokens);

        // TODO: how to emulate increase of circulation supply?

        vm.prank(FilAddress.SYSTEM_ACTOR);
        xnet.applyCrossMessages(msgs);
    }

    function executeTopDownMsgsRevert(IpcEnvelope[] memory msgs, SubnetID memory subnet, address gateway) internal {
        vm.expectRevert();
        executeTopDownMsgs(msgs, subnet, gateway);
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        address gateway
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = GatewayGetterFacet(address(gateway));
        CheckpointingFacet checkpointer = CheckpointingFacet(address(gateway));

        BottomUpMsgBatch memory batch = getter.bottomUpMsgBatch(e);
        console.log("batch length %d", batch.msgs.length);
        require(batch.msgs.length > 0, "batch length incorrect");
        if (batch.msgs.length == 2) {
            printEnvelope(batch.msgs[0]);
            printEnvelope(batch.msgs[1]);
        }

        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: batch.msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        return checkpoint;
    }

    function submitBottomUpCheckpoint(BottomUpCheckpoint memory checkpoint, address subnetActor) internal {
        (uint256[] memory parentKeys, address[] memory parentValidators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory parentPubKeys = new bytes[](3);
        bytes[] memory parentSignatures = new bytes[](3);

        SubnetActorManagerFacet manager = SubnetActorManagerFacet(subnetActor);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(parentValidators[i], 10 gwei);
            parentPubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(parentKeys[i]);
            vm.prank(parentValidators[i]);
            manager.join{value: 10}(parentPubKeys[i]);
        }

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(parentKeys[i], hash);
            parentSignatures[i] = abi.encodePacked(r, s, v);
        }

        SubnetActorCheckpointingFacet checkpointer = SubnetActorCheckpointingFacet(subnetActor);

        vm.startPrank(subnetActor);
        console.log("submitCheckpoint");
        checkpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function submitBottomUpCheckpointRevert(BottomUpCheckpoint memory checkpoint, address subnetActor) internal {
        vm.expectRevert();
        submitBottomUpCheckpoint(checkpoint, subnetActor);
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    function printActors() internal view {
        console.log("root gateway: %s", address(rootGateway));
        console.log("root actor: %s", rootSubnetName.getActor());
        console.log("root native subnet actor: %s", (address(rootNativeSubnetActor)));
        console.log("root token subnet actor: %s", (address(rootTokenSubnetActor)));
        console.log("root name: %s", rootSubnetName.toString());
        console.log("native subnet name: %s", nativeSubnetName.toString());
        console.log("token subnet name: %s", tokenSubnetName.toString());
        console.log("native subnet getActor(): %s", address(nativeSubnetName.getActor()));
        console.log("native subnet gateway(): %s", address(nativeSubnetGateway));
    }

    //prints any IpcEnvelope for debugging
    function printEnvelope(IpcEnvelope memory envelope) public {
        console.log("\nPrint Envelope");
        console.log("from %s:", envelope.from.subnetId.toString());
        console.log("to %s:", envelope.to.subnetId.toString());
        console.log("Nonce");
        console.log(envelope.nonce);
        console.log("Value");
        console.log(envelope.value);
        console.log("Message");
        console.logBytes(envelope.message);
        console.log("Hash");
        console.logBytes32(envelope.toHash());
        if (envelope.kind == IpcMsgKind.Result) {
            ResultMsg memory result = abi.decode(envelope.message, (ResultMsg));
            console.log("Result id");
            console.logBytes32(result.id);
        } else if (envelope.kind == IpcMsgKind.Call) {
            CallMsg memory call = abi.decode(envelope.message, (CallMsg));
            console.log("Call Msg");
            console.logBytes(call.method);
            console.logBytes(call.params);
        }
    }

    // @dev This test verifies that USDC bridge connects correctly
    // a contract from native subnet with a contract in token subnet through the rootnet.
    function testMultiSubnet_Native_FundFromParentToChild_USDCBridge() public {
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        IpcEnvelope memory expected;

        address holder = vm.addr(100);
        address recipient = vm.addr(200);
        address owner = address(this);
        uint256 transferAmount = 300;
        uint256 holderTotalAmount = 1000;

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        console.log("--------------- transfer and mint (top-down) ---------------");

        USDCTest testUSDC = new USDCTest();

        testUSDC.mint(100_000);
        testUSDC.transfer(holder, holderTotalAmount);

        require(testUSDC.owner() == owner, "unexpected owner");
        require(testUSDC.balanceOf(holder) == holderTotalAmount, "unexpected balance");

        // the token replica sits in a native supply child subnet.
        ipcTokenReplica = new IpcTokenReplica({
            _gateway: address(nativeSubnetGateway),
            _controller: address(ipcTokenController),
            _controllerSubnet: rootSubnetName
        });

        // the token controller sits in the root network.
        ipcTokenController = new IpcTokenController({
            _gateway: address(rootGateway),
            _tokenContractAddress: address(testUSDC),
            _destinationSubnet: nativeSubnetName,
            _destinationContract: address(ipcTokenReplica)
        });
        ipcTokenReplica.setController(address(ipcTokenController));

        vm.prank(holder);
        testUSDC.approve(address(ipcTokenController), transferAmount);

        console.log("mock usdc contract: %s", address(testUSDC));
        console.log("mock usdc owner: %s", owner);
        console.log("mock usdc holder: %s", address(holder));
        console.log("ipcTokenController: %s", address(ipcTokenController));
        console.log(
            "controller allowance for holder: %d",
            testUSDC.allowance(address(holder), address(ipcTokenController))
        );

        vm.prank(address(holder));
        IpcEnvelope memory lockAndTransferEnvelope = ipcTokenController.lockAndTransferWithReturn(
            recipient,
            transferAmount
        );

        // Check that the message is in unconfirmedTransfers
        (address receiptSender, uint256 receiptValue) = ipcTokenController.getUnconfirmedTransfer(
            lockAndTransferEnvelope.toHash()
        );
        require(receiptSender == address(holder), "Transfer sender incorrect in unconfirmedTransfers");
        require(receiptValue == transferAmount, "Transfer amount incorrect in unconfirmedTransfers");

        //confirm that token replica only accept calls to Ipc from the gateway
        vm.prank(owner);
        vm.expectRevert(IpcHandler.CallerIsNotGateway.selector);
        ipcTokenReplica.handleIpcMessage(expected);

        // the message the root gateway's postbox is being executed in the token subnet's gateway

        expected = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({
                subnetId: rootSubnetName,
                rawAddress: FvmAddressHelper.from(address(ipcTokenController))
            }),
            to: lockAndTransferEnvelope.to,
            value: 0,
            message: lockAndTransferEnvelope.message,
            nonce: 0 // nonce will be updated by LibGateway.commitCrossMessage
        });

        msgs[0] = expected;
        executeTopDownMsgs(msgs, nativeSubnetName, address(nativeSubnetGateway));

        //ensure that tokens are delivered on subnet
        require(IERC20(ipcTokenReplica).balanceOf(recipient) == transferAmount, "incorrect proxy token balance");

        console.log("--------------- withdraw token (bottom-up)---------------");

        // ensure that USDC holder has 0 tokens in the root chain
        // require(0 == testUSDC.balanceOf(holder), "unexpected holder balance in withdraw flow");

        vm.prank(recipient);
        expected = ipcTokenReplica.burnAndTransfer(holder, transferAmount);

        // check that the message is in unconfirmedTransfers
        (receiptSender, receiptValue) = ipcTokenReplica.getUnconfirmedTransfer(expected.toHash());
        require(receiptSender == recipient, "Transfer sender incorrect in unconfirmedTransfers");
        require(receiptValue == transferAmount, "Transfer amount incorrect in unconfirmedTransfers");

        console.log("Begin bottom up checkpoint");

        // TODO: This is already tested in IpcContract.t.sol. No need to retest here.
        //confirm that token controller only accept calls to Ipc from the gateway
        // vm.prank(holder);
        // vm.expectRevert(IpcHandler.CallerIsNotGateway.selector);
        // ipcTokenController.handleIpcMessage(expected);

        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnetName,
            address(nativeSubnetGateway)
        );
        submitBottomUpCheckpoint(checkpoint, address(rootNativeSubnetActor));

        //ensure that usdc tokens are returned on root net
        require(holderTotalAmount == testUSDC.balanceOf(holder), "unexpected holder balance after withdrawal");
        //ensure that the tokens are the subnet are minted and the token bridge and the usdc holder does not own any
        require(0 == ipcTokenReplica.balanceOf(holder), "unexpected holder balance in ipcTokenReplica");
        require(
            0 == ipcTokenReplica.balanceOf(address(ipcTokenReplica)),
            "unexpected ipcTokenReplica balance in ipcTokenReplica"
        );
    }
}
