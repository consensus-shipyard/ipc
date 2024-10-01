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
import {TestUtils, MockIpcContract, MockIpcContractPayable, MockIpcContractRevert, MockIpcContractFallback} from "../helpers/TestUtils.sol";
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

    RootSubnetDefinition public rootNetwork;
    TestSubnetDefinition public nativeSubnet;
    TestSubnetDefinition[] public nativeL3Subnets;

    // TODO use later
    IERC20 public token;

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

        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1_000_000, address(this));

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

    function createTokenSubnet(
        address tokenAddress,
        address rootGatewayAddress,
        SubnetID memory rootNetworkName
    ) internal returns (TestSubnetDefinition memory tokenSubnet) {
        SubnetActorDiamond rootTokenSubnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(rootGatewayAddress, rootNetworkName, tokenAddress)
        );
        address[] memory tokenSubnetPath = new address[](1);
        tokenSubnetPath[0] = address(rootTokenSubnetActor);
        SubnetID memory tokenSubnetName = SubnetID({root: ROOTNET_CHAINID, route: tokenSubnetPath});
        GatewayDiamond tokenSubnetGateway = createGatewayDiamond(gatewayParams(tokenSubnetName));

        tokenSubnet = TestSubnetDefinition({
            gateway: tokenSubnetGateway,
            gatewayAddr: address(tokenSubnetGateway),
            id: tokenSubnetName,
            subnetActor: rootTokenSubnetActor,
            subnetActorAddr: address(rootTokenSubnetActor),
            path: tokenSubnetPath
        });
    }

    //--------------------
    // Call flow tests.
    //---------------------

    function testL2PlusSubnet_Native_SendCrossMessageFromChildToParentWithResult() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContractPayable());
        uint256 amount = 3;

        // register L2 into root nerwork
        vm.deal(nativeSubnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(nativeSubnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, nativeSubnet.subnetActorAddr, rootNetwork.gateway);

        TestSubnetDefinition memory nativeL3Subnet = nativeL3Subnets[0];

        // register L3 into L2 subnet
        vm.deal(nativeL3Subnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(nativeL3Subnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, nativeL3Subnet.subnetActorAddr, nativeSubnet.gateway);

        vm.deal(caller, 1 ether);
        vm.prank(caller);
        // fund from root network to L2
        rootNetwork.gateway.manager().fund{value: 100000}(nativeSubnet.id, FvmAddressHelper.from(address(caller)));

        // fund from L2 to L3
        nativeSubnet.gateway.manager().fund{value: 100000}(nativeL3Subnet.id, FvmAddressHelper.from(address(caller)));

        // create the xnet message on the subnet L3 - it's local gateway
        GatewayMessengerFacet messenger = nativeL3Subnet.gateway.messenger();
        vm.prank(address(caller));

        IpcEnvelope memory crossMessage = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: nativeL3Subnet.id, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: rootNetwork.id, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        messenger.sendContractXnetMessage{value: amount}(crossMessage);

        // this would normally be done by Fendermint. It call the local gateway and creates checkpoint.
        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeL3Subnet.id,
            nativeL3Subnet.gateway
        );

        // this would normally submitted by releayer. It call the subnet actor on the L2 network.
        submitBottomUpCheckpoint(checkpoint, nativeL3Subnet.subnetActor);

        // create checkpoint in L2 and submit it to the root network (L2 subnet actor)
        BottomUpCheckpoint memory checkpointL2 = callCreateBottomUpCheckpointFromChildSubnet(
            nativeSubnet.id,
            nativeSubnet.gateway
        );

        // expected result top down message from root to L2. This is a response to the xnet call.
        IpcEnvelope memory resultMessage = crossMessage.createResultMsg(OutcomeType.Ok, abi.encode(EMPTY_BYTES));
        resultMessage.nonce = 1;

        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            checkpointL2,
            nativeSubnet.subnetActor,
            resultMessage,
            nativeSubnet.subnetActorAddr,
            rootNetwork.gatewayAddr
        );

        assertEq(recipient.balance, 3);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromParentToChild() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(nativeSubnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        vm.prank(nativeSubnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, nativeSubnet.subnetActorAddr, rootNetwork.gateway);

        // register L3 into L2 subnet
        TestSubnetDefinition memory nativeL3Subnet = nativeL3Subnets[0];

        vm.deal(nativeL3Subnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(nativeL3Subnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, nativeL3Subnet.subnetActorAddr, nativeSubnet.gateway);

        vm.prank(caller);
        rootNetwork.gateway.manager().fund{value: 100000}(nativeSubnet.id, FvmAddressHelper.from(address(caller)));

        IpcEnvelope memory xnetCallMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: rootNetwork.id, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: nativeL3Subnet.id, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        IpcEnvelope memory committedEvent = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: rootNetwork.id, rawAddress: FvmAddressHelper.from(caller)}),
            to: xnetCallMsg.to,
            value: xnetCallMsg.value,
            message: xnetCallMsg.message,
            nonce: 1
        });

        vm.prank(address(caller));
        vm.expectEmit(true, true, true, true, rootNetwork.gatewayAddr);
        emit LibGateway.NewTopDownMessage({subnet: nativeSubnet.subnetActorAddr, message: committedEvent});
        rootNetwork.gateway.messenger().sendContractXnetMessage{value: amount}(xnetCallMsg);

        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = xnetCallMsg;

        commitParentFinality(nativeSubnet.gatewayAddr);
        executeTopDownMsgs(msgs, nativeSubnet.gateway);

        commitParentFinality(nativeL3Subnet.gatewayAddr);
        executeTopDownMsgs(msgs, nativeL3Subnet.gateway);

        assertEq(address(recipient).balance, amount);
    }

    function testL2PlusSubnet_Native_SendCrossMessageFromSiblingToSibling() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());
        uint256 amount = 3;

        vm.deal(nativeSubnet.subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
        vm.deal(caller, 1 ether);

        // register L2 into root nerwork
        vm.prank(nativeSubnet.subnetActorAddr);
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, nativeSubnet.subnetActorAddr, rootNetwork.gateway);

        // register L3s into L2 subnet
        for (uint256 i; i < nativeL3Subnets.length; i++) {
            vm.deal(nativeL3Subnets[i].subnetActorAddr, DEFAULT_COLLATERAL_AMOUNT);
            vm.prank(nativeL3Subnets[i].subnetActorAddr);
            registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, nativeL3Subnets[i].subnetActorAddr, nativeSubnet.gateway);
        }

        // fund account in the L3-0 subnet
        vm.prank(caller);
        nativeSubnet.gateway.manager().fund{value: 100000}(nativeL3Subnets[0].id, FvmAddressHelper.from(address(caller)));

        // create the xnet message to send fund from L3-0 to L3-1
        IpcEnvelope memory xnetCallMsg = TestUtils.newXnetCallMsg(
            IPCAddress({subnetId: nativeL3Subnets[0].id, rawAddress: FvmAddressHelper.from(caller)}),
            IPCAddress({subnetId: nativeL3Subnets[1].id, rawAddress: FvmAddressHelper.from(recipient)}),
            amount,
            0
        );

        GatewayMessengerFacet messenger = nativeL3Subnets[0].gateway.messenger();
        vm.prank(address(caller));
        messenger.sendContractXnetMessage{value: amount}(xnetCallMsg);

        // this would normally be done by Fendermint. It call the local gateway and creates checkpoint.
        BottomUpCheckpoint memory checkpoint = callCreateBottomUpCheckpointFromChildSubnet(
            nativeL3Subnets[0].id,
            nativeL3Subnets[0].gateway
        );

        submitBottomUpCheckpointAndExpectTopDownMessageEvent(
            checkpoint,
            nativeL3Subnets[0].subnetActor,
            xnetCallMsg,
            nativeL3Subnets[1].subnetActorAddr,
            nativeSubnet.gatewayAddr
        );

        // mimics the execution of the top down messages in the L3-1 subnet
        IpcEnvelope[] memory msgs = new IpcEnvelope[](1);
        msgs[0] = xnetCallMsg;

        commitParentFinality(nativeL3Subnets[1].gatewayAddr);
        executeTopDownMsgs(msgs, nativeL3Subnets[1].gateway);

        assertEq(recipient.balance, 3);
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

        // TODO: how to emulate increase of circulation supply?

        vm.prank(FilAddress.SYSTEM_ACTOR);
        GatewayMessengerFacet messenger = gw.messenger();
        messenger.propagateAll();

        vm.prank(FilAddress.SYSTEM_ACTOR);
        XnetMessagingFacet xnetMessenger = gw.xnetMessenger();
        xnetMessenger.applyCrossMessages(msgs);
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
        emit LibGateway.NewTopDownMessage({subnet: expectedSubnetAddr, message: expectedMessage});
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
