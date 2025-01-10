// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/errors/IPCErrors.sol";
import {EMPTY_BYTES} from "../../contracts/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, IpcMsgKind, OutcomeType} from "../../contracts/structs/CrossNet.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../../contracts/structs/Subnet.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";
import {Asset, AssetKind} from "../../contracts/structs/Subnet.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {GatewayDiamond} from "../../contracts/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../../contracts/SubnetActorDiamond.sol";
import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../contracts/subnet/SubnetActorCheckpointingFacet.sol";
import {GatewayGetterFacet} from "../../contracts/gateway/GatewayGetterFacet.sol";
import {LibGateway} from "../../contracts/lib/LibGateway.sol";
import {TopDownFinalityFacet} from "../../contracts/gateway/router/TopDownFinalityFacet.sol";
import {CheckpointingFacet} from "../../contracts/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../contracts/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../contracts/gateway/GatewayMessengerFacet.sol";
import {IntegrationTestBase, RootSubnetDefinition, TestSubnetDefinition} from "../IntegrationTestBase.sol";
import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractResult} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";
import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";
import {SubnetActorFacetsHelper} from "../helpers/SubnetActorFacetsHelper.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";

import {ActivityHelper} from "../helpers/ActivityHelper.sol";

import "forge-std/console.sol";

contract L2PlusSubnetTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using GatewayFacetsHelper for GatewayDiamond;
    using SubnetActorFacetsHelper for SubnetActorDiamond;
    using AssetHelper for Asset;

    RootSubnetDefinition public rootNetwork;
    // native subnets
    TestSubnetDefinition public nativeL2Subnet;
    TestSubnetDefinition[] public nativeL3Subnets;

    // token subnets
    IERC20 public token;
    TestSubnetDefinition public tokenL2Subnet;
    TestSubnetDefinition[] public nativeL3SubnetsWithTokenParent;
    IERC20 public tokenL3;
    TestSubnetDefinition[] public tokenL3SubnetsWithTokenParent;

    function setUp() public override {
        SubnetID memory rootNetworkName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(rootNetworkName.isRoot(), "not root");

        GatewayDiamond rootGateway = createGatewayDiamond(gatewayParams(rootNetworkName));

        rootNetwork = RootSubnetDefinition({
            gateway: rootGateway,
            gatewayAddr: address(rootGateway),
            id: rootNetworkName
        });

        nativeL2Subnet = createNativeSubnet(rootNetwork.gatewayAddr, rootNetwork.id);

        nativeL3Subnets.push(createNativeSubnet(nativeL2Subnet.gatewayAddr, nativeL2Subnet.id));
        nativeL3Subnets.push(createNativeSubnet(nativeL2Subnet.gatewayAddr, nativeL2Subnet.id));

        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1_000_000, address(this));
        tokenL2Subnet = createTokenSubnet(address(token), rootNetwork.gatewayAddr, rootNetworkName);

        nativeL3SubnetsWithTokenParent.push(createNativeSubnet(tokenL2Subnet.gatewayAddr, tokenL2Subnet.id));
        nativeL3SubnetsWithTokenParent.push(createNativeSubnet(tokenL2Subnet.gatewayAddr, tokenL2Subnet.id));

        tokenL3 = new ERC20PresetFixedSupply("TestL3Token", "TEST3", 1_000_000, address(this));

        tokenL3SubnetsWithTokenParent.push(
            createTokenSubnet(address(tokenL3), tokenL2Subnet.gatewayAddr, tokenL2Subnet.id)
        );
        tokenL3SubnetsWithTokenParent.push(
            createTokenSubnet(address(tokenL3), tokenL2Subnet.gatewayAddr, tokenL2Subnet.id)
        );

        printActors();
    }

    function createNativeSubnet(
        address parentGatewayAddress,
        SubnetID memory parentNetworkName
    ) internal returns (TestSubnetDefinition memory) {
        SubnetActorDiamond subnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(parentGatewayAddress, parentNetworkName)
        );

        return createSubnet(parentNetworkName.route, subnetActor);
    }

    function createTokenSubnet(
        address tokenAddress,
        address parentGatewayAddress,
        SubnetID memory parentNetworkName
    ) internal returns (TestSubnetDefinition memory) {
        SubnetActorDiamond subnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(parentGatewayAddress, parentNetworkName, tokenAddress)
        );

        return createSubnet(parentNetworkName.route, subnetActor);
    }

    function createSubnet(
        address[] memory subnetPath,
        SubnetActorDiamond subnetActor
    ) internal returns (TestSubnetDefinition memory) {
        address[] memory newPath = new address[](subnetPath.length + 1);
        for (uint i = 0; i < subnetPath.length; i++) {
            newPath[i] = subnetPath[i];
        }

        newPath[subnetPath.length] = address(subnetActor);

        SubnetID memory subnetName = SubnetID({root: ROOTNET_CHAINID, route: newPath});
        GatewayDiamond subnetGateway = createGatewayDiamond(gatewayParams(subnetName));

        return
            TestSubnetDefinition({
                gateway: subnetGateway,
                gatewayAddr: address(subnetGateway),
                id: subnetName,
                subnetActor: subnetActor,
                subnetActorAddr: address(subnetActor),
                path: newPath
            });
    }

    struct Params {
        RootSubnetDefinition root;
        TestSubnetDefinition subnet;
        TestSubnetDefinition subnetL3;
        MockIpcContractResult caller;
        address callerAddr;
        address recipientAddr;
        uint256 callerAmount;
        uint256 fundAmount;
        uint256 amount;
        OutcomeType expectedOutcome;
        bytes expectedRet;
    }

    //--------------------
    // Call flow tests.
    //---------------------

    // Native supply source subnets
    function testL2PlusSubnet_Native_SendCrossMessageFromChildToParentWithOkResult() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: nativeL2Subnet,
            subnetL3: nativeL3Subnets[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: address(new MockIpcContractPayable()),
            amount: 3,
            expectedOutcome: OutcomeType.Ok,
            expectedRet: abi.encode(EMPTY_BYTES),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromChildToParentWithResult(params);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromParentToChildWithOkResult() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: nativeL2Subnet,
            subnetL3: nativeL3Subnets[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: address(new MockIpcContractPayable()),
            amount: 3,
            expectedOutcome: OutcomeType.Ok,
            expectedRet: abi.encode(EMPTY_BYTES),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromParentToChildWithResult(params);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromSiblingToSiblingWithOkResult() public {
        sendCrossMessageFromSiblingToSiblingWithOkResult(rootNetwork, nativeL2Subnet, nativeL3Subnets);
    }

    // Token supply source subnets
    function testL2PlusSubnet_Token_SendCrossMessageFromChildToParentWithOkResult() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: tokenL2Subnet,
            subnetL3: nativeL3SubnetsWithTokenParent[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: address(new MockIpcContractPayable()),
            amount: 3,
            expectedOutcome: OutcomeType.Ok,
            expectedRet: abi.encode(EMPTY_BYTES),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromChildToParentWithResult(params);
    }

    function testL2PlusSubnet_Token_SendCrossMessageFromParentToChildWithOkResult() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: tokenL2Subnet,
            subnetL3: nativeL3SubnetsWithTokenParent[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: address(new MockIpcContractPayable()),
            amount: 3,
            expectedOutcome: OutcomeType.Ok,
            expectedRet: abi.encode(EMPTY_BYTES),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromParentToChildWithResult(params);
    }

    function testL2PlusSubnet_Token_SendCrossMessageFromSiblingToSiblingWithOkResult() public {
        sendCrossMessageFromSiblingToSiblingWithOkResult(rootNetwork, tokenL2Subnet, nativeL3SubnetsWithTokenParent);
    }

    function testL2PlusSubnet_TokenMixed_SendCrossMessageFromSiblingToSiblingWithOkResult() public {
        sendCrossMessageFromSiblingToSiblingWithOkResult(rootNetwork, tokenL2Subnet, tokenL3SubnetsWithTokenParent);
    }

    // Error scenarios
    function testL2PlusSubnet_Native_SendCrossMessageFromChildToNonExistingActorError() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: nativeL2Subnet,
            subnetL3: nativeL3Subnets[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: 0x53c82507aA03B1a6e695000c302674ef1ecb880B,
            amount: 3,
            expectedOutcome: OutcomeType.ActorErr,
            expectedRet: abi.encodeWithSelector(InvalidSubnetActor.selector),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromChildToParentWithResult(params);
    }

    function testL2PlusSubnet_Token_SendCrossMessageFromChildToNonExistingActorError() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: tokenL2Subnet,
            subnetL3: nativeL3SubnetsWithTokenParent[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: 0x53c82507aA03B1a6e695000c302674ef1ecb880B,
            amount: 3,
            expectedOutcome: OutcomeType.ActorErr,
            expectedRet: abi.encodeWithSelector(InvalidSubnetActor.selector),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromChildToParentWithResult(params);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromParentToNonExistingActorError() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: nativeL2Subnet,
            subnetL3: nativeL3Subnets[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: 0x53c82507aA03B1a6e695000c302674ef1ecb880B,
            amount: 3,
            expectedOutcome: OutcomeType.ActorErr,
            expectedRet: abi.encodeWithSelector(InvalidSubnetActor.selector),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromParentToChildWithResult(params);
    }

    function testL2PlusSubnet_Token_SendCrossMessageFromParentToNonExistingActorError() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: tokenL2Subnet,
            subnetL3: nativeL3SubnetsWithTokenParent[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: 0x53c82507aA03B1a6e695000c302674ef1ecb880B,
            amount: 3,
            expectedOutcome: OutcomeType.ActorErr,
            expectedRet: abi.encodeWithSelector(InvalidSubnetActor.selector),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        sendCrossMessageFromParentToChildWithResult(params);
    }

    function testL2PlusSubnet_ParentToChildTopDownNoncePropagatedCorrectly() public {
        MockIpcContractResult caller = new MockIpcContractResult();
        Params memory params = Params({
            root: rootNetwork,
            subnet: nativeL2Subnet,
            subnetL3: nativeL3Subnets[0],
            caller: caller,
            callerAddr: address(caller),
            recipientAddr: address(new MockIpcContractPayable()),
            amount: 3,
            expectedOutcome: OutcomeType.Ok,
            expectedRet: abi.encode(EMPTY_BYTES),
            callerAmount: 1 ether,
            fundAmount: 100000
        });

        // register L2 into root network
        registerSubnet(params.subnet.subnetActorAddr, params.root.gateway);
        // register L3 into L2 subnet
        registerSubnet(params.subnetL3.subnetActorAddr, params.subnet.gateway);

        vm.deal(params.callerAddr, params.callerAmount);

        IpcEnvelope memory fundCrossMessage = CrossMsgHelper.createFundMsg({
            subnet: params.subnet.id,
            signer: params.callerAddr,
            to: FvmAddressHelper.from(params.callerAddr),
            value: params.amount
        });

        // 0 is default but we set it explicitly here to make it clear
        fundCrossMessage.localNonce = 0;

        vm.prank(params.callerAddr);
        vm.expectEmit(true, true, true, true, params.root.gatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: params.subnet.subnetActorAddr,
            message: fundCrossMessage,
            id: fundCrossMessage.toTracingId()
        });

        params.root.gateway.manager().fund{value: params.amount}(
            params.subnet.id,
            FvmAddressHelper.from(params.callerAddr)
        );

        IpcEnvelope memory callCrossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: params.root.id, rawAddress: FvmAddressHelper.from(params.callerAddr)}),
            IPCAddress({subnetId: params.subnetL3.id, rawAddress: FvmAddressHelper.from(params.recipientAddr)}),
            params.amount,
            1
        );

        // send the cross message from the root network to the L3 subnet
        vm.prank(params.callerAddr);
        vm.expectEmit(true, true, true, true, params.root.gatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: params.subnet.subnetActorAddr,
            message: callCrossMessage,
            id: callCrossMessage.toTracingId()
        });

        params.root.gateway.messenger().sendContractXnetMessage{value: params.amount}(callCrossMessage);
        (, uint64 rootTopDownNonce) = params.root.gateway.getter().getTopDownNonce(params.subnet.id);
        assertEq(rootTopDownNonce, 2, "wrong root top down nonce");

        IpcEnvelope[] memory msgsForL2 = new IpcEnvelope[](2);
        msgsForL2[0] = fundCrossMessage;
        msgsForL2[1] = callCrossMessage;

        // the expected nonce for the top down message for L3 subnet is 0 because no previous message was sent
        // from L2 to L3
        msgsForL2[1].localNonce = 0;
        vm.prank(FilAddress.SYSTEM_ACTOR);
        vm.expectEmit(true, true, true, true, params.subnet.gatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: params.subnetL3.subnetActorAddr,
            message: callCrossMessage,
            id: callCrossMessage.toTracingId()
        });

        // nonce needs to be 1 because of the fund message.
        msgsForL2[1].localNonce = 1;
        params.subnet.gateway.xnetMessenger().applyCrossMessages(msgsForL2);

        uint64 subnetAppliedTopDownNonce = params.subnet.gateway.getter().appliedTopDownNonce();
        assertEq(subnetAppliedTopDownNonce, 2, "wrong L2 subnet applied top down nonce");

        IpcEnvelope[] memory msgsForL3 = new IpcEnvelope[](1);
        msgsForL3[0] = callCrossMessage;

        vm.prank(FilAddress.SYSTEM_ACTOR);
        // nonce is zero because this is a first message touching the L3 subnet
        msgsForL3[0].localNonce = 0;
        params.subnetL3.gateway.xnetMessenger().applyCrossMessages(msgsForL3);

        uint64 subnetL3AppliedTopDownNonce = params.subnetL3.gateway.getter().appliedTopDownNonce();
        assertEq(subnetL3AppliedTopDownNonce, 1, "wrong L3 subnet applied top down nonce");

        // now fund from L2 to L3 to check to nonce propagation
        vm.deal(params.callerAddr, params.callerAmount);

        IpcEnvelope memory fundCrossMessageL3 = CrossMsgHelper.createFundMsg({
            subnet: params.subnetL3.id,
            signer: params.callerAddr,
            to: FvmAddressHelper.from(params.callerAddr),
            value: params.amount
        });

        // nonce should be 1 because this is the first cross message from L1 to L3
        fundCrossMessageL3.localNonce = 1;

        vm.prank(params.callerAddr);
        vm.expectEmit(true, true, true, true, params.subnet.gatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: params.subnetL3.subnetActorAddr,
            message: fundCrossMessageL3,
            id: fundCrossMessageL3.toTracingId()
        });

        params.subnet.gateway.manager().fund{value: params.amount}(
            params.subnetL3.id,
            FvmAddressHelper.from(params.callerAddr)
        );

        uint64 subnetL3AppliedTopDownNonceAfterFund = params.subnetL3.gateway.getter().appliedTopDownNonce();
        assertEq(subnetL3AppliedTopDownNonceAfterFund, 1, "wrong L3 subnet applied top down nonce");
    }

    function fundSubnet(
        GatewayDiamond gateway,
        TestSubnetDefinition memory subnet,
        address callerAddr,
        uint256 amount
    ) internal {
        Asset memory subnetSupply = subnet.subnetActor.getter().supplySource();
        if (subnetSupply.kind == AssetKind.ERC20) {
            IERC20(subnetSupply.tokenAddress).approve(address(gateway), amount);
            gateway.manager().fundWithToken(subnet.id, FvmAddressHelper.from(callerAddr), amount);
        } else {
            gateway.manager().fund{value: amount}(subnet.id, FvmAddressHelper.from(callerAddr));
        }
    }

    function registerSubnet(address subnetActorAddr, GatewayDiamond gateway) internal {
        vm.deal(subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnetActorAddr, gateway);
        vm.stopPrank();
    }

    function sendCrossMessageFromChildToParentWithResult(Params memory params) public {
        // register L2 into root nerwork
        registerSubnet(params.subnet.subnetActorAddr, params.root.gateway);
        // register L3 into L2 subnet
        registerSubnet(params.subnetL3.subnetActorAddr, params.subnet.gateway);

        vm.deal(params.callerAddr, params.callerAmount);
        vm.prank(params.callerAddr);

        fundSubnet(params.root.gateway, params.subnet, params.callerAddr, params.fundAmount);
        fundSubnet(params.subnet.gateway, params.subnetL3, params.callerAddr, params.fundAmount);

        // create the xnet message on the subnet L3 - it's local gateway
        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: params.subnetL3.id, rawAddress: FvmAddressHelper.from(params.callerAddr)}),
            IPCAddress({subnetId: params.root.id, rawAddress: FvmAddressHelper.from(params.recipientAddr)}),
            params.amount,
            0
        );

        vm.prank(params.callerAddr);
        params.subnetL3.gateway.messenger().sendContractXnetMessage{value: params.amount}(crossMessage);

        // this would normally submitted by releayer. It call the subnet actor on the L2 network.
        submitBottomUpCheckpoint(
            callCreateBottomUpCheckpointFromChildSubnet(params.subnetL3.id, params.subnetL3.gateway),
            params.subnetL3.subnetActor
        );

        // create checkpoint in L2 and submit it to the root network (L2 subnet actor)
        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            params.subnet.id,
            params.subnet.gateway
        );

        // expected result top down message from root to L2. This is a response to the xnet call.
        IpcEnvelope memory resultMessage = crossMessage.createResultMsg(params.expectedOutcome, params.expectedRet);
        resultMessage.localNonce = 1;

        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            checkpoint,
            params.subnet.subnetActor,
            resultMessage,
            params.subnet.subnetActorAddr,
            params.root.gatewayAddr
        );

        // apply the result message in the L2 subnet and expect another top down message to be emitted
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = cloneIpcEnvelopeWithDifferentNonce(resultMessage, 0);

        commitParentFinality(params.subnet.gatewayAddr);
        executeTopDownMsgsAndExpectTopDownMessageEvent(
            msgs,
            params.subnet.gateway,
            resultMessage,
            params.subnetL3.subnetActorAddr,
            params.subnet.gatewayAddr
        );

        // apply the result and check it was propagated to the correct actor
        commitParentFinality(params.subnetL3.gatewayAddr);
        executeTopDownMsgs(msgs, params.subnetL3.gateway);

        assertTrue(params.caller.hasResult(), "missing result");
        assertTrue(params.caller.result().outcome == params.expectedOutcome, "wrong result outcome");
        assertTrue(keccak256(params.caller.result().ret) == keccak256(params.expectedRet), "wrong result outcome");
    }

    function sendCrossMessageFromParentToChildWithResult(Params memory params) public {
        // register L2 into root network
        registerSubnet(params.subnet.subnetActorAddr, params.root.gateway);
        // register L3 into L2 subnet
        registerSubnet(params.subnetL3.subnetActorAddr, params.subnet.gateway);

        vm.deal(params.callerAddr, params.callerAmount);
        vm.prank(params.callerAddr);

        fundSubnet(params.root.gateway, params.subnet, params.callerAddr, params.fundAmount);

        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: params.root.id, rawAddress: FvmAddressHelper.from(params.callerAddr)}),
            IPCAddress({subnetId: params.subnetL3.id, rawAddress: FvmAddressHelper.from(params.recipientAddr)}),
            params.amount,
            0
        );

        Asset memory subnetSupply = params.subnet.subnetActor.getter().supplySource();

        if (subnetSupply.kind == AssetKind.ERC20) {
            // increase callerAddr's token balance
            IERC20(subnetSupply.tokenAddress).transfer(params.callerAddr, params.amount);
            // increase allowance so that send xnet msg will make it
            vm.prank(params.callerAddr);
            IERC20(subnetSupply.tokenAddress).approve(address(params.root.gatewayAddr), params.amount);
        }

        crossMessage.localNonce = 1;
        // send the cross message from the root network to the L3 subnet
        vm.prank(params.callerAddr);
        vm.expectEmit(true, true, true, true, params.root.gatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: params.subnet.subnetActorAddr,
            message: crossMessage,
            id: crossMessage.toTracingId()
        });

        crossMessage.localNonce = 0;
        if (subnetSupply.kind == AssetKind.ERC20) {
            params.root.gateway.messenger().sendContractXnetMessage(crossMessage);
        } else {
            params.root.gateway.messenger().sendContractXnetMessage{value: params.amount}(crossMessage);
        }

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = crossMessage;

        // propagate the message from the L2 to the L3 subnet
        executeTopDownMsgsAndExpectTopDownMessageEvent(
            msgs,
            params.subnet.gateway,
            crossMessage,
            params.subnetL3.subnetActorAddr,
            params.subnet.gatewayAddr
        );
        // apply the cross message in the L3 subnet
        executeTopDownMsgs(msgs, params.subnetL3.gateway);
        // submit checkoint so the result message can be propagated to L2
        submitBottomUpCheckpoint(
            callCreateBottomUpCheckpointFromChildSubnet(params.subnetL3.id, params.subnetL3.gateway),
            params.subnetL3.subnetActor
        );
        // submit checkoint so the result message can be propagated to root network
        submitBottomUpCheckpoint(
            callCreateBottomUpCheckpointFromChildSubnet(params.subnet.id, params.subnet.gateway),
            params.subnet.subnetActor
        );
        assertTrue(params.caller.hasResult(), "missing result");
        assertTrue(params.caller.result().outcome == params.expectedOutcome, "wrong result outcome");
        assertTrue(keccak256(params.caller.result().ret) == keccak256(params.expectedRet), "wrong result outcome");
    }

    function sendCrossMessageFromSiblingToSiblingWithOkResult(
        RootSubnetDefinition memory root,
        TestSubnetDefinition memory subnet,
        TestSubnetDefinition[] memory subnetL3s
    ) public {
        MockIpcContractResult caller = new MockIpcContractResult();
        address callerAddr = address(caller);
        address recipientAddr = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(callerAddr, 1 ether);

        // register L2 into root nerwork
        registerSubnet(subnet.subnetActorAddr, root.gateway);

        // register L3s into L2 subnet
        for (uint256 i; i < subnetL3s.length; i++) {
            registerSubnet(subnetL3s[i].subnetActorAddr, subnet.gateway);
        }

        // fund account in the L3-0 subnet
        vm.prank(callerAddr);

        fundSubnet(subnet.gateway, subnetL3s[0], callerAddr, 100000);

        // create the xnet message to send fund from L3-0 to L3-1
        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: subnetL3s[0].id, rawAddress: FvmAddressHelper.from(callerAddr)}),
            IPCAddress({subnetId: subnetL3s[1].id, rawAddress: FvmAddressHelper.from(recipientAddr)}),
            amount,
            0
        );

        vm.prank(callerAddr);
        subnetL3s[0].gateway.messenger().sendContractXnetMessage{value: amount}(crossMessage);

        // submit the checkpoint from L3-0 to L2
        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            subnetL3s[0].id,
            subnetL3s[0].gateway
        );

        // submit the checkpoint in L2 produces top down message to L3-1
        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            checkpoint,
            subnetL3s[0].subnetActor,
            crossMessage,
            subnetL3s[1].subnetActorAddr,
            subnet.gatewayAddr
        );

        // mimics the execution of the top down messages in the L3-1 subnet
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = crossMessage;

        executeTopDownMsgs(msgs, subnetL3s[1].gateway);

        // submit the checkpoint from L3-1 to L2 for result propagation
        BottomUpCheckpoint memory resultCheckpoint = callCreateBottomUpCheckpointFromChildSubnet(
            subnetL3s[1].id,
            subnetL3s[1].gateway
        );

        // expected result top down message from L2 to L3. This is a response to the xnet call.
        IpcEnvelope memory resultMessage = crossMessage.createResultMsg(OutcomeType.Ok, abi.encode(EMPTY_BYTES));
        resultMessage.localNonce = 1;

        // submit the checkpoint in L2 produces top down message to L3-1
        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            resultCheckpoint,
            subnetL3s[1].subnetActor,
            resultMessage,
            subnetL3s[0].subnetActorAddr,
            subnet.gatewayAddr
        );

        // apply the result message in the L3-1 subnet
        resultMessage.localNonce = 0;
        IpcEnvelope[] memory resultMsgs = new IpcEnvelope[](1);
        resultMsgs[0] = resultMessage;

        executeTopDownMsgs(resultMsgs, subnetL3s[0].gateway);
        assertTrue(caller.hasResult(), "missing result");
        assertTrue(caller.result().outcome == OutcomeType.Ok, "wrong result outcome");
    }

    function commitParentFinality(address gateway) internal {
        vm.roll(10);
        ParentFinality memory finality = ParentFinality({height: block.number, blockHash: bytes32(0)});

        TopDownFinalityFacet gwTopDownFinalityFacet = TopDownFinalityFacet(address(gateway));

        vm.prank(FilAddress.SYSTEM_ACTOR);
        gwTopDownFinalityFacet.commitParentFinality(finality);
    }

    function executeTopDownMsgs(IpcEnvelope[] memory msgs, GatewayDiamond gw) internal {
        uint256 minted_tokens;

        for (uint256 i; i < msgs.length; ) {
            minted_tokens += msgs[i].value;
            unchecked {
                ++i;
            }
        }
        console.log("minted tokens in executed top-downs: %d", minted_tokens);

        // The implementation of the function in fendermint is in
        // https://github.com/consensus-shipyard/ipc/blob/main/fendermint/vm/interpreter/contracts/fvm/topdown.rs#L43

        // This emulates minting tokens.
        vm.deal(address(gw), minted_tokens);

        vm.prank(FilAddress.SYSTEM_ACTOR);
        XnetMessagingFacet xnetMessenger = gw.xnetMessenger();
        xnetMessenger.applyCrossMessages(msgs);
    }

    function executeTopDownMsgsAndExpectTopDownMessageEvent(
        IpcEnvelope[] memory msgs,
        GatewayDiamond gw,
        IpcEnvelope memory expectedMessage,
        address expectedSubnetAddr,
        address expectedGatewayAddr
    ) internal {
        uint256 minted_tokens;

        for (uint256 i; i < msgs.length; ) {
            minted_tokens += msgs[i].value;
            unchecked {
                ++i;
            }
        }
        console.log("minted tokens in executed top-downs: %d", minted_tokens);

        // The implementation of the function in fendermint is in
        // https://github.com/consensus-shipyard/ipc/blob/main/fendermint/vm/interpreter/contracts/fvm/topdown.rs#L43

        // This emulates minting tokens.
        vm.deal(address(gw), minted_tokens);

        vm.prank(FilAddress.SYSTEM_ACTOR);
        vm.expectEmit(true, true, true, true, expectedGatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: expectedSubnetAddr,
            message: expectedMessage,
            id: expectedMessage.toTracingId()
        });
        XnetMessagingFacet xnetMessenger = gw.xnetMessenger();
        xnetMessenger.applyCrossMessages(msgs);
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        GatewayDiamond gw
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = gw.getter();
        CheckpointingFacet checkpointer = gw.checkpointer();

        BottomUpMsgBatch memory batch = getter.bottomUpMsgBatch(e);
        require(batch.msgs.length == 1, "batch length incorrect");

        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: batch.msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        return checkpoint;
    }

    function callCreateBottomUpCheckpointFromChildSubnet(
        SubnetID memory subnet,
        GatewayDiamond gw,
        IpcEnvelope[] memory msgs
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        CheckpointingFacet checkpointer = gw.checkpointer();

        (, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, ) = MerkleTreeHelper.createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: subnet,
            blockHeight: e,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: msgs,
            activity: ActivityHelper.newCompressedActivityRollup(1, 3, bytes32(uint256(0)))
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2],
            ActivityHelper.dummyActivityRollup()
        );
        vm.stopPrank();

        return checkpoint;
    }

    function prepareValidatorsSignatures(
        BottomUpCheckpoint memory checkpoint,
        SubnetActorDiamond sa
    ) internal returns (address[] memory, bytes[] memory) {
        (uint256[] memory parentKeys, address[] memory parentValidators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory parentPubKeys = new bytes[](3);
        bytes[] memory parentSignatures = new bytes[](3);

        SubnetActorManagerFacet manager = sa.manager();

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(parentValidators[i], 10 gwei);
            parentPubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(parentKeys[i]);
            vm.prank(parentValidators[i]);
            manager.join{value: 10}(parentPubKeys[i], 10);
        }

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(parentKeys[i], hash);
            parentSignatures[i] = abi.encodePacked(r, s, v);
        }

        return (parentValidators, parentSignatures);
    }

    function submitBottomUpCheckpoint(BottomUpCheckpoint memory checkpoint, SubnetActorDiamond sa) internal {
        (address[] memory parentValidators, bytes[] memory parentSignatures) = prepareValidatorsSignatures(
            checkpoint,
            sa
        );

        SubnetActorCheckpointingFacet checkpointer = sa.checkpointer();

        vm.startPrank(address(sa));
        checkpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function submitBottomUpCheckpointAndExpectTopDownMessageEvent(
        BottomUpCheckpoint memory checkpoint,
        SubnetActorDiamond subnetActor,
        IpcEnvelope memory expectedMessage,
        address expectedSubnetAddr,
        address expectedGatewayAddr
    ) internal {
        (address[] memory parentValidators, bytes[] memory parentSignatures) = prepareValidatorsSignatures(
            checkpoint,
            subnetActor
        );

        SubnetActorCheckpointingFacet checkpointer = subnetActor.checkpointer();

        vm.startPrank(address(subnetActor));
        vm.expectEmit(true, true, true, true, expectedGatewayAddr);
        emit LibGateway.NewTopDownMessage({
            subnet: expectedSubnetAddr,
            message: expectedMessage,
            id: expectedMessage.toTracingId()
        });
        checkpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function submitBottomUpCheckpointRevert(BottomUpCheckpoint memory checkpoint, SubnetActorDiamond sa) internal {
        vm.expectRevert();
        submitBottomUpCheckpoint(checkpoint, sa);
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    function cloneIpcEnvelopeWithDifferentNonce(
        IpcEnvelope memory original,
        uint64 newNonce
    ) internal pure returns (IpcEnvelope memory) {
        return
            IpcEnvelope({
                kind: original.kind,
                to: original.to,
                from: original.from,
                localNonce: newNonce,
                originalNonce: 0,
                value: original.value,
                message: original.message
            });
    }

    function printActors() internal view {
        console.log("root name: %s", rootNetwork.id.toString());
        console.log("root gateway: %s", rootNetwork.gatewayAddr);
        console.log("root actor: %s", rootNetwork.id.getActor());
        console.log("--------------------");

        console.log("native L2 subnet name: %s", nativeL2Subnet.id.toString());
        console.log("native L2 subnet gateway: %s", nativeL2Subnet.gatewayAddr);
        console.log("native L2 subnet actor: %s", (nativeL2Subnet.subnetActorAddr));

        for (uint256 i; i < nativeL3Subnets.length; i++) {
            console.log("--------------------");
            console.log("native L3-%d subnet name: %s", i, nativeL3Subnets[i].id.toString());
            console.log("native L3-%d subnet gateway: %s", i, nativeL3Subnets[i].gatewayAddr);
            console.log("native L3-%d subnet actor: %s", i, (nativeL3Subnets[i].subnetActorAddr));
        }

        for (uint256 i; i < nativeL3SubnetsWithTokenParent.length; i++) {
            console.log("--------------------");
            console.log(
                "native L3-%d subnet with token parent name: %s",
                i,
                nativeL3SubnetsWithTokenParent[i].id.toString()
            );
            console.log(
                "native L3-%d subnet with token parent gateway: %s",
                i,
                nativeL3SubnetsWithTokenParent[i].gatewayAddr
            );
            console.log(
                "native L3-%d subnet with token parent actor: %s",
                i,
                (nativeL3SubnetsWithTokenParent[i].subnetActorAddr)
            );
        }
    }
}
