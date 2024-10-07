// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../contracts/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint, ParentFinality, IpcMsgKind, OutcomeType} from "../../contracts/structs/CrossNet.sol";
import {FvmAddress} from "../../contracts/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../../contracts/structs/Subnet.sol";
import {SubnetIDHelper} from "../../contracts/lib/SubnetIDHelper.sol";
import {AssetHelper} from "../../contracts/lib/AssetHelper.sol";
import {Asset, AssetKind} from "../../contracts/structs/Subnet.sol";
import {FvmAddressHelper} from "../../contracts/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../contracts/lib/CrossMsgHelper.sol";
import {GatewayDiamond, FEATURE_MULTILEVEL_CROSSMSG} from "../../contracts/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../../contracts/SubnetActorDiamond.sol";
import {SubnetActorGetterFacet} from "../../contracts/subnet/SubnetActorGetterFacet.sol";
import {SubnetActorManagerFacet} from "../../contracts/subnet/SubnetActorManagerFacet.sol";
import {SubnetActorCheckpointingFacet} from "../../contracts/subnet/SubnetActorCheckpointingFacet.sol";
import {GatewayGetterFacet} from "../../contracts/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../contracts/gateway/GatewayManagerFacet.sol";
import {LibGateway} from "../../contracts/lib/LibGateway.sol";
import {TopDownFinalityFacet} from "../../contracts/gateway/router/TopDownFinalityFacet.sol";
import {CheckpointingFacet} from "../../contracts/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../contracts/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../contracts/gateway/GatewayMessengerFacet.sol";
import {DiamondLoupeFacet} from "../../contracts/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../contracts/diamond/DiamondCutFacet.sol";
import {IntegrationTestBase, RootSubnetDefinition, TestSubnetDefinition} from "../IntegrationTestBase.sol";
import {L2GatewayActorDiamond, L1GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractResult, MockIpcContractRevert, MockIpcContractFallback} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/contracts/utils/FilAddress.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";
import {ERC20Deflationary} from "../helpers/ERC20Deflationary.sol";
import {ERC20Inflationary} from "../helpers/ERC20Inflationary.sol";
import {ERC20Nil} from "../helpers/ERC20Nil.sol";

import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";

import {GatewayFacetsHelper} from "../helpers/GatewayFacetsHelper.sol";
import {SubnetActorFacetsHelper} from "../helpers/SubnetActorFacetsHelper.sol";

import "forge-std/console.sol";

contract L2PlusSubnetTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;
    using GatewayFacetsHelper for GatewayDiamond;
    using SubnetActorFacetsHelper for SubnetActorDiamond;
    using AssetHelper for Asset;

    RootSubnetDefinition public rootNetwork;
    // native subnets
    TestSubnetDefinition public nativeSubnet;
    TestSubnetDefinition[] public nativeL3Subnets;

    // token subnets
    IERC20 public token;
    TestSubnetDefinition public tokenSubnet;
    TestSubnetDefinition[] public tokenL3Subnets;

    function setUp() public override {
        SubnetID memory rootNetworkName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(rootNetworkName.isRoot(), "not root");

        GatewayDiamond rootGateway = createGatewayDiamond(gatewayParams(rootNetworkName));

        rootNetwork = RootSubnetDefinition({
            gateway: rootGateway,
            gatewayAddr: address(rootGateway),
            id: rootNetworkName
        });

        address[] memory nativeSubnetPath;
        nativeSubnet = createNativeSubnet(nativeSubnetPath, rootNetwork.gatewayAddr, rootNetwork.id);

        nativeL3Subnets.push(createNativeSubnet(nativeSubnet.path, nativeSubnet.gatewayAddr, nativeSubnet.id));
        nativeL3Subnets.push(createNativeSubnet(nativeSubnet.path, nativeSubnet.gatewayAddr, nativeSubnet.id));

        address[] memory tokenSubnetPath;
        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1_000_000, address(this));
        tokenSubnet = createTokenSubnet(address(token), tokenSubnetPath, rootNetwork.gatewayAddr, rootNetworkName);

        tokenL3Subnets.push(createNativeSubnet(tokenSubnet.path, tokenSubnet.gatewayAddr, tokenSubnet.id));
        tokenL3Subnets.push(createNativeSubnet(tokenSubnet.path, tokenSubnet.gatewayAddr, tokenSubnet.id));

        printActors();
    }

    function createNativeSubnet(
        address[] memory subnetPath,
        address parentGatewayAddress,
        SubnetID memory parentNetworkName
    ) internal returns (TestSubnetDefinition memory) {
        SubnetActorDiamond subnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(parentGatewayAddress, parentNetworkName)
        );

        return createSubnet(subnetPath, parentGatewayAddress, parentNetworkName, subnetActor);
    }

    function createTokenSubnet(
        address tokenAddress,
        address[] memory subnetPath,
        address parentGatewayAddress,
        SubnetID memory parentNetworkName
    ) internal returns (TestSubnetDefinition memory tokenSubnet) {
        SubnetActorDiamond subnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(parentGatewayAddress, parentNetworkName, tokenAddress)
        );
        
        return createSubnet(subnetPath, parentGatewayAddress, parentNetworkName, subnetActor);
    }

    function createSubnet(
        address[] memory subnetPath,
        address parentGatewayAddress,
        SubnetID memory parentNetworkName,
        SubnetActorDiamond subnetActor
    ) internal returns (TestSubnetDefinition memory) {
        address[] memory newPath = new address[](subnetPath.length + 1);
        for (uint i = 0; i < subnetPath.length; i++) {
            newPath[i] = subnetPath[i];
        }

        newPath[subnetPath.length] = address(subnetActor);

        SubnetID memory subnetName = SubnetID({root: ROOTNET_CHAINID, route: newPath});
        GatewayDiamond subnetGateway = createGatewayDiamond(gatewayParams(subnetName));

        return TestSubnetDefinition({
            gateway: subnetGateway,
            gatewayAddr: address(subnetGateway),
            id: subnetName,
            subnetActor: subnetActor,
            subnetActorAddr: address(subnetActor),
            path: newPath
        });
    }

    //--------------------
    // Call flow tests.
    //---------------------

    // Native supply source subnets
    function testL2PlusSubnet_Native_SendCrossMessageFromChildToParentWithResult() public {
        sendCrossMessageFromChildToParentWithResult(rootNetwork, nativeSubnet, nativeL3Subnets[0]);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromParentToChildWithResult() public {
        sendCrossMessageFromParentToChildWithResult(rootNetwork, nativeSubnet, nativeL3Subnets[0]);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromSiblingToSiblingWithResult() public {
        sendCrossMessageFromSiblingToSiblingWithResult(rootNetwork, nativeSubnet, nativeL3Subnets);
    }
    
    // Token supply source subnets
    function testL2PlusSubnet_Token_SendCrossMessageFromChildToParentWithResult() public {
        sendCrossMessageFromChildToParentWithResult(rootNetwork, tokenSubnet, tokenL3Subnets[0]);
    }

    function testL2PlusSubnet_Token_SendCrossMessageFromParentToChildWithResult() public {
        sendCrossMessageFromParentToChildWithResult(rootNetwork, tokenSubnet, tokenL3Subnets[0]);
    }

    function testL2PlusSubnet_Token_SendCrossMessageFromSiblingToSiblingWithResult() public {
        sendCrossMessageFromSiblingToSiblingWithResult(rootNetwork, tokenSubnet, tokenL3Subnets);
    }

    // Error scenarios
    function testL2PlusSubnet_Native_SendCrossMessageFromChildToParentNonExisitingActor() public {
        sendCrossMessageFromChildToParentWithResult(rootNetwork, nativeSubnet, nativeL3Subnets[0]);
    }

    function sendCrossMessageFromChildToParentWithResult(
        RootSubnetDefinition memory root,
        TestSubnetDefinition memory subnet,
        TestSubnetDefinition memory subnetL3
    ) public {
        MockIpcContractResult caller = new MockIpcContractResult();
        address callerAddr = address(caller);
        address recipientAddr = address(new MockIpcContractPayable());
        uint256 amount = 3;

        // register L2 into root nerwork
        vm.deal(subnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(subnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnet.subnetActorAddr, root.gateway);

        // register L3 into L2 subnet
        vm.deal(subnetL3.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(subnetL3.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnetL3.subnetActorAddr, subnet.gateway);

        vm.deal(callerAddr, 1 ether);
        vm.prank(callerAddr);

        Asset memory subnetSupply = subnet.subnetActor.getter().supplySource();
        
        // fund from root network to L2
        if (subnetSupply.kind == AssetKind.ERC20) {
            IERC20(subnetSupply.tokenAddress).approve(root.gatewayAddr, 100000);
            root.gateway.manager().fundWithToken(subnet.id, FvmAddressHelper.from(callerAddr), 100000);
        } else {
            root.gateway.manager().fund{value: 100000}(subnet.id, FvmAddressHelper.from(callerAddr));
        }

        // fund from L2 to L3
        subnet.gateway.manager().fund{value: 100000}(subnetL3.id, FvmAddressHelper.from(callerAddr));

        // create the xnet message on the subnet L3 - it's local gateway
        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: subnetL3.id, rawAddress: FvmAddressHelper.from(callerAddr)}),
            IPCAddress({subnetId: root.id, rawAddress: FvmAddressHelper.from(recipientAddr)}),
            amount,
            0
        );

        vm.prank(callerAddr);
        subnetL3.gateway.messenger().sendContractXnetMessage{value: amount}(crossMessage);

        // this would normally submitted by releayer. It call the subnet actor on the L2 network.
        submitBottomUpCheckpoint(callCreateBottomUpCheckpointFromChildSubnet(
            subnetL3.id,
            subnetL3.gateway
        ), subnetL3.subnetActor);

        // create checkpoint in L2 and submit it to the root network (L2 subnet actor)
        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            subnet.id,
            subnet.gateway
        );

        // expected result top down message from root to L2. This is a response to the xnet call.
        IpcEnvelope memory resultMessage = crossMessage.createResultMsg(OutcomeType.Ok, abi.encode(EMPTY_BYTES));
        resultMessage.nonce = 1;

        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            checkpoint,
            subnet.subnetActor,
            resultMessage,
            subnet.subnetActorAddr,
            root.gatewayAddr
        );

        assertEq(subnetSupply.balanceOf(recipientAddr), amount, "wrong recipient balance");
    
        // apply the result message in the L2 subnet and expect another top down message to be emitted
        resultMessage.nonce = 0;
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = resultMessage;
        
        commitParentFinality(subnet.gatewayAddr);
        executeTopDownMsgs(msgs, subnet.gateway);

        resultMessage.nonce = 1;
        propagateAllAndExpectTopDownMessageEvent(
            subnet.gateway,
            resultMessage,
            subnetL3.subnetActorAddr,
            subnet.gatewayAddr
        );

        resultMessage.nonce = 0;
        // apply the result and check it was propagated to the correct actor
        commitParentFinality(subnetL3.gatewayAddr);
        executeTopDownMsgs(msgs, subnetL3.gateway);

        assertTrue(caller.hasResult(), "missing result");
        assertTrue(caller.result().outcome == OutcomeType.Ok, "wrong result outcome");
    }

    function sendCrossMessageFromParentToChildWithResult(
        RootSubnetDefinition memory root,
        TestSubnetDefinition memory subnet,
        TestSubnetDefinition memory subnetL3
    ) public {
        MockIpcContractResult caller = new MockIpcContractResult();
        address callerAddr = address(caller);
        address recipientAddr = address(new MockIpcContractPayable());
        uint256 amount = 3;

        vm.deal(subnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(callerAddr, 1 ether);

         // register L2 into root network
        vm.prank(subnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnet.subnetActorAddr, root.gateway);

        // register L3 into L2 subnet
        vm.deal(subnetL3.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(subnetL3.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnetL3.subnetActorAddr, subnet.gateway);

        vm.prank(callerAddr);

        // Asset memory subnetSupply = subnet.subnetActor.getter().supplySource();
        
        // fund from root network to L2
        if (subnet.subnetActor.getter().supplySource().kind == AssetKind.ERC20) {
            IERC20(subnet.subnetActor.getter().supplySource().tokenAddress).approve(root.gatewayAddr, 100000);
            root.gateway.manager().fundWithToken(subnet.id, FvmAddressHelper.from(callerAddr), 100000);
        } else {
            root.gateway.manager().fund{value: 100000}(subnet.id, FvmAddressHelper.from(callerAddr));
        }

        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: root.id, rawAddress: FvmAddressHelper.from(callerAddr)}),
            IPCAddress({subnetId: subnetL3.id, rawAddress: FvmAddressHelper.from(recipientAddr)}),
            amount,
            0
        );

        IpcEnvelope memory expectedMessage = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: crossMessage.from,
            to: crossMessage.to,
            value: crossMessage.value,
            message: crossMessage.message,
            nonce: 1
        });

        // send the cross message from the root network to the L3 subnet
        vm.prank(callerAddr);
        vm.expectEmit(true, true, true, true, root.gatewayAddr);
        emit LibGateway.NewTopDownMessage({subnet: subnet.subnetActorAddr, message: expectedMessage, id: expectedMessage.toHash()});
        root.gateway.messenger().sendContractXnetMessage{value: amount}(crossMessage);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = crossMessage;

        // propagate the message from the L2 to the L3 subnet
        executeTopDownMsgs(msgs, subnet.gateway);

        // check that the cross message is propagated to the L3 subnet
        expectedMessage.nonce = 0;
        propagateAllAndExpectTopDownMessageEvent(
            subnet.gateway,
            expectedMessage,
            subnetL3.subnetActorAddr,
            subnet.gatewayAddr
        );

        // apply the cross message in the L3 subnet
        executeTopDownMsgs(msgs, subnetL3.gateway);
        assertEq(recipientAddr.balance, amount);

        // submit checkoint so the result message can be propagated to L2
        submitBottomUpCheckpoint(callCreateBottomUpCheckpointFromChildSubnet(
            subnetL3.id,
            subnetL3.gateway
        ), subnetL3.subnetActor);

        // submit checkoint so the result message can be propagated to root network
        submitBottomUpCheckpoint(callCreateBottomUpCheckpointFromChildSubnet(
            subnet.id,
            subnet.gateway
        ), subnet.subnetActor);

        assertTrue(caller.hasResult(), "missing result");
        assertTrue(caller.result().outcome == OutcomeType.Ok, "wrong result outcome");
    }

    function sendCrossMessageFromSiblingToSiblingWithResult(
        RootSubnetDefinition memory root,
        TestSubnetDefinition memory subnet,
        TestSubnetDefinition[] memory subnetL3s
    ) public {
        MockIpcContractResult caller = new MockIpcContractResult();
        address callerAddr = address(caller);
        address recipientAddr = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(subnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(callerAddr, 1 ether);

        // register L2 into root nerwork
        vm.prank(subnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnet.subnetActorAddr, root.gateway);

        // register L3s into L2 subnet
        for (uint256 i; i < subnetL3s.length; i++) {
            vm.deal(subnetL3s[i].subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
            vm.prank(subnetL3s[i].subnetActorAddr);
            registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, subnetL3s[i].subnetActorAddr, subnet.gateway);
        }

        // fund account in the L3-0 subnet
        vm.prank(callerAddr);
        subnet.gateway.manager().fund{value: 100000}(subnetL3s[0].id, FvmAddressHelper.from(callerAddr));

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
        assertEq(recipientAddr.balance, amount);

        // submit the checkpoint from L3-1 to L2 for result propagation
        BottomUpCheckpoint memory resultCheckpoint = callCreateBottomUpCheckpointFromChildSubnet(
            subnetL3s[1].id,
            subnetL3s[1].gateway
        );

        // expected result top down message from L2 to L3. This is a response to the xnet call.
        IpcEnvelope memory resultMessage = crossMessage.createResultMsg(OutcomeType.Ok, abi.encode(EMPTY_BYTES));
        resultMessage.nonce = 1;

        // submit the checkpoint in L2 produces top down message to L3-1
        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            resultCheckpoint,
            subnetL3s[1].subnetActor,
            resultMessage,
            subnetL3s[0].subnetActorAddr,
            subnet.gatewayAddr
        );

        // apply the result message in the L3-1 subnet
        resultMessage.nonce = 0;
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

    function propagateAllAndExpectTopDownMessageEvent(
        GatewayDiamond gw,
        IpcEnvelope memory expectedMessage,
        address expectedSubnetAddr,
        address expectedGatewayAddr
    ) internal {
        vm.prank(FilAddress.SYSTEM_ACTOR);
        vm.expectEmit(true, true, true, true, expectedGatewayAddr);
        emit LibGateway.NewTopDownMessage({subnet: expectedSubnetAddr, message: expectedMessage, id: expectedMessage.toHash()});
        GatewayMessengerFacet messenger = gw.messenger();
        messenger.propagateAll();
    }

    function executeTopDownMsgsRevert(IpcEnvelope[] memory msgs, GatewayDiamond gw) internal {
        vm.expectRevert();
        executeTopDownMsgs(msgs, gw);
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
            msgs: batch.msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
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
            msgs: msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        checkpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        return checkpoint;
    }

    function prepareValidatorsSignatures(BottomUpCheckpoint memory checkpoint, SubnetActorDiamond sa) internal returns (address[] memory, bytes[] memory) {
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
        (address[] memory parentValidators, bytes[] memory parentSignatures) = prepareValidatorsSignatures(checkpoint,sa);

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
        (address[] memory parentValidators, bytes[] memory parentSignatures) = prepareValidatorsSignatures(checkpoint,subnetActor);

        SubnetActorCheckpointingFacet checkpointer = subnetActor.checkpointer();

        vm.startPrank(address(subnetActor));
        vm.expectEmit(true, true, true, true, expectedGatewayAddr);
        emit LibGateway.NewTopDownMessage({subnet: expectedSubnetAddr, message: expectedMessage, id: expectedMessage.toHash()});
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

    function printActors() internal view {
        console.log("root name: %s", rootNetwork.id.toString());
        console.log("root gateway: %s", rootNetwork.gatewayAddr);
        console.log("root actor: %s", rootNetwork.id.getActor());
        console.log("--------------------");

        console.log("native L2 subnet name: %s", nativeSubnet.id.toString());
        console.log("native L2 subnet gateway: %s", nativeSubnet.gatewayAddr);
        console.log("native L2 subnet actor: %s", (nativeSubnet.subnetActorAddr));

        for (uint256 i; i < nativeL3Subnets.length; i++) {
            console.log("--------------------");
            console.log("native L3-%d subnet name: %s", i, nativeL3Subnets[i].id.toString());
            console.log("native L3-%d subnet gateway: %s", i, nativeL3Subnets[i].gatewayAddr);
            console.log("native L3-%d subnet actor: %s", i, (nativeL3Subnets[i].subnetActorAddr));
        }
    }

    function printEnvelope(IpcEnvelope memory envelope) internal view {
        console.log("from %s:", envelope.from.subnetId.toString());
        console.log("to %s:", envelope.to.subnetId.toString());
    }
}
