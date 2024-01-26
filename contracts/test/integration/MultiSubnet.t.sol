// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../src/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint} from "../../src/structs/CrossNet.sol";
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
import {CheckpointingFacet} from "../../src/gateway/router/CheckpointingFacet.sol";
import {XnetMessagingFacet} from "../../src/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../src/gateway/GatewayMessengerFacet.sol";
import {DiamondLoupeFacet} from "../../src/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {IntegrationTestBase} from "../IntegrationTestBase.sol";
import {L2GatewayActorDiamond, L1GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {TestUtils, MockIpcContract} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {MerkleTreeHelper} from "../helpers/MerkleTreeHelper.sol";

import {IERC20} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ERC20PresetFixedSupply} from "../helpers/ERC20PresetFixedSupply.sol";
import {IERC20Errors} from "openzeppelin-contracts/interfaces/draft-IERC6093.sol";

import "forge-std/console.sol";

contract MultiSubnet is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;
    using CrossMsgHelper for IpcEnvelope;

    GatewayDiamond public rootGateway;
    GatewayGetterFacet public rootGatewayGetter;
    GatewayManagerFacet public rootGatewayManager;
    GatewayMessengerFacet public rootGatewayMessenger;

    SubnetActorDiamond public rootSubnetActor;
    SubnetActorGetterFacet public rootSubnetActorGetter;
    SubnetActorCheckpointingFacet public rootSubnetActorCheckpointer;
    SubnetActorManagerFacet public rootSubnetActorManager;

    SubnetActorDiamond public rootTokenSubnetActor;
    SubnetActorGetterFacet public rootTokenSubnetActorGetter;
    SubnetActorCheckpointingFacet public rootTokenSubnetActorCheckpointer;
    SubnetActorManagerFacet public rootTokenSubnetActorManager;

    GatewayDiamond public tokenSubnetGateway;

    GatewayDiamond public childGateway;
    GatewayGetterFacet public childGatewayGetter;
    CheckpointingFacet public childGatewayCheckpointer;
    GatewayMessengerFacet public childGatewayMessenger;

    SubnetActorDiamond public childSubnetActor;

    address[] public childPath;
    address[] public tokenPath;

    SubnetID rootName;
    SubnetID childName;
    SubnetID tokenSubnetName;

    IERC20 public token;

    function setUp() public override {
        token = new ERC20PresetFixedSupply("TestToken", "TEST", 1_000_000, address(this));

        rootName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(rootName.isRoot(), "not root");

        rootGateway = createGatewayDiamond(gatewayParams(rootName));
        rootGatewayGetter = GatewayGetterFacet(address(rootGateway));
        rootGatewayMessenger = GatewayMessengerFacet(address(rootGateway));
        rootGatewayManager = GatewayManagerFacet(address(rootGateway));

        rootSubnetActor = createSubnetActor(subnetActorWithParams(address(rootGateway), rootName));
        rootSubnetActorGetter = SubnetActorGetterFacet(address(rootSubnetActor));
        rootSubnetActorCheckpointer = SubnetActorCheckpointingFacet(address(rootSubnetActor));
        rootSubnetActorManager = SubnetActorManagerFacet(address(rootSubnetActor));

        rootTokenSubnetActor = createSubnetActor(subnetActorWithParams(address(rootGateway), rootName, address(token)));
        rootTokenSubnetActorGetter = SubnetActorGetterFacet(address(rootTokenSubnetActor));
        rootTokenSubnetActorCheckpointer = SubnetActorCheckpointingFacet(address(rootTokenSubnetActor));
        rootTokenSubnetActorManager = SubnetActorManagerFacet(address(rootTokenSubnetActor));

        tokenPath = new address[](1);
        tokenPath[0] = address(rootTokenSubnetActor);
        tokenSubnetName = SubnetID({root: ROOTNET_CHAINID, route: tokenPath});

        childPath = new address[](1);
        childPath[0] = address(rootSubnetActor);

        childName = SubnetID({root: ROOTNET_CHAINID, route: childPath});
        childGateway = createGatewayDiamond(gatewayParams(childName));
        childSubnetActor = createSubnetActor(subnetActorWithParams(address(childGateway), childName));
        childGatewayGetter = GatewayGetterFacet(address(childGateway));
        childGatewayMessenger = GatewayMessengerFacet(address(childGateway));
        childGatewayCheckpointer = CheckpointingFacet(address(childGateway));

        tokenSubnetGateway = createGatewayDiamond(gatewayParams(tokenSubnetName));

        console.log("root actor: %s", rootName.getActor());
        console.log("child network actor: %s", childName.getActor());
        console.log("root subnet actor: %s", (address(rootSubnetActor)));
        console.log("root token subnet actor: %s", (address(rootTokenSubnetActor)));
        console.log("root name: %s", rootName.toString());
        console.log("child name: %s", childName.toString());
    }

    function gatewayParams(SubnetID memory id) internal pure returns (GatewayDiamond.ConstructorParams memory) {
        GatewayDiamond.ConstructorParams memory params = GatewayDiamond.ConstructorParams({
            networkName: id,
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: DEFAULT_ACTIVE_VALIDATORS_LIMIT
        });

        return params;
    }

    function testGatewayDiamond_MultiSubnet_SendCrossMessageFromChildToParent() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());

        vm.deal(address(rootSubnetActor), DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 1);
        vm.deal(caller, 3);

        vm.prank(address(rootSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootSubnetActor), rootGateway);

        address funderAddress = address(567);
        vm.deal(funderAddress, 1 ether);
        SubnetID memory fundedSubnetId = rootName.createSubnetId((address(rootSubnetActor)));
        vm.startPrank(funderAddress);
        rootGatewayManager.fund{value: 100000}(fundedSubnetId, FvmAddressHelper.from(address(funderAddress)));
        vm.stopPrank();

        vm.prank(address(caller));
        GatewayMessengerFacet messenger = GatewayMessengerFacet(address(childGateway));
        messenger.sendContractXnetMessage{value: 3}(
            TestUtils.newXnetCallMsg(
                IPCAddress({subnetId: childName, rawAddress: FvmAddressHelper.from(caller)}),
                IPCAddress({subnetId: rootName, rawAddress: FvmAddressHelper.from(recipient)}),
                3,
                0
            )
        );

        BottomUpCheckpoint memory checkpoint = createBottomUpCheckpointInChildSubnet(childName, address(childGateway));

        submitCheckpointInParentSubnet(checkpoint, address(rootSubnetActor));

        assertEq(recipient.balance, 3);
    }

    function testGatewayDiamond_MultiSubnet_Token_ChildToParentCall() public {
        address caller = address(new MockIpcContract());
        address recipient = address(new MockIpcContract());

        uint256 value = 3;

        // Fund an account in the subnet.
        token.transfer(caller, 100);
        vm.prank(caller);
        token.approve(address(rootGateway), 100);

        vm.deal(address(rootTokenSubnetActor), DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 1);
        vm.deal(address(token), DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 1);

        vm.prank(address(rootTokenSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootTokenSubnetActor), rootGateway);

        address funderAddress = address(567);
        vm.deal(funderAddress, 1 ether);
        SubnetID memory fundedSubnetId = rootName.createSubnetId((address(rootTokenSubnetActor)));
        vm.startPrank(caller);
        token.approve(address(funderAddress), 100);
        rootGatewayManager.fundWithToken(fundedSubnetId, FvmAddressHelper.from(address(funderAddress)), 15);
        vm.stopPrank();

        SubnetID memory subnetId = rootGatewayGetter.getNetworkName().createSubnetId(address(rootTokenSubnetActor));

        IPCAddress memory from = IPCAddress({subnetId: subnetId, rawAddress: FvmAddressHelper.from(caller)});
        IPCAddress memory to = IPCAddress({subnetId: rootName, rawAddress: FvmAddressHelper.from(recipient)});
        bytes4 method = bytes4(0x11223344);
        bytes memory params = bytes("hello");
        IpcEnvelope memory envelope = CrossMsgHelper.createCallMsg(from, to, value, method, params);
        printEnvelope(envelope);

        vm.deal(caller, 10000);
        vm.prank(address(caller));
        GatewayMessengerFacet messenger = GatewayMessengerFacet(address(tokenSubnetGateway));
        messenger.sendContractXnetMessage{value: value}(envelope);

        BottomUpCheckpoint memory checkpoint = createBottomUpCheckpointInChildSubnet(
            subnetId,
            address(tokenSubnetGateway)
        );

        submitCheckpointInParentSubnet(checkpoint, address(rootTokenSubnetActor));

        assertEq(token.balanceOf(recipient), value);
    }

    function createBottomUpCheckpointInChildSubnet(
        SubnetID memory subnet,
        address gateway
    ) internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        GatewayGetterFacet getter = GatewayGetterFacet(address(gateway));

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
        childGatewayCheckpointer.createBottomUpCheckpoint(
            checkpoint,
            membershipRoot,
            weights[0] + weights[1] + weights[2]
        );
        vm.stopPrank();

        return checkpoint;
    }

    function printEnvelope(IpcEnvelope memory envelope) internal {
        console.log("from %s:", envelope.from.subnetId.toString());
        console.log("to %s:", envelope.to.subnetId.toString());
    }

    function submitCheckpointInParentSubnet(BottomUpCheckpoint memory checkpoint, address subnetActor) internal {
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
        checkpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }
}
