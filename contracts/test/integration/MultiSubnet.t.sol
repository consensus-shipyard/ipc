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

    GatewayDiamond public childGateway;
    GatewayGetterFacet public childGatewayGetter;
    CheckpointingFacet public childGatewayCheckpointer;
    GatewayMessengerFacet public childGatewayMessenger;

    SubnetActorDiamond public childSubnetActor;

    address[] public childPath;

    SubnetID rootName;
    SubnetID childName;

    function setUp() public override {
        rootName = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(rootName.isRoot(), "not root");

        rootGateway = createGatewayDiamond(gatewayParams(rootName));
        rootSubnetActor = createSubnetActor(subnetActorWithParams(address(rootGateway), rootName));
        rootGatewayGetter = GatewayGetterFacet(address(rootGateway));
        rootGatewayMessenger = GatewayMessengerFacet(address(rootGateway));
        rootGatewayManager = GatewayManagerFacet(address(rootGateway));
        rootSubnetActorGetter = SubnetActorGetterFacet(address(rootSubnetActor));
        rootSubnetActorCheckpointer = SubnetActorCheckpointingFacet(address(rootSubnetActor));
        rootSubnetActorManager = SubnetActorManagerFacet(address(rootSubnetActor));

        childPath = new address[](1);
        childPath[0] = address(rootSubnetActor);

        childName = SubnetID({root: ROOTNET_CHAINID, route: childPath});
        childGateway = createGatewayDiamond(gatewayParams(childName));
        childSubnetActor = createSubnetActor(subnetActorWithParams(address(childGateway), childName));
        childGatewayGetter = GatewayGetterFacet(address(childGateway));
        childGatewayMessenger = GatewayMessengerFacet(address(childGateway));
        childGatewayCheckpointer = CheckpointingFacet(address(childGateway));

        console.log("root actor: %s", rootName.getActor());
        console.log("child network actor: %s", childName.getActor());
        console.log("root subnet actor: %s", (address(rootSubnetActor)));
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

    function defaultL2GatewayParams() internal pure returns (GatewayDiamond.ConstructorParams memory) {
        address[] memory path = new address[](2);
        path[0] = vm.addr(10);
        path[1] = vm.addr(20);

        GatewayDiamond.ConstructorParams memory params = GatewayDiamond.ConstructorParams({
            networkName: SubnetID({root: ROOTNET_CHAINID, route: path}),
            bottomUpCheckPeriod: DEFAULT_CHECKPOINT_PERIOD,
            majorityPercentage: DEFAULT_MAJORITY_PERCENTAGE,
            genesisValidators: new Validator[](0),
            activeValidatorsLimit: DEFAULT_ACTIVE_VALIDATORS_LIMIT
        });

        return params;
    }

    function testGatewayDiamond_MultiSubnet_SendCrossMessageFromChildToParent() public {

        address caller = address(new MockIpcContract());

        vm.deal(address(rootSubnetActor), DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 1);
        vm.deal(caller, 1);

        vm.prank(address(rootSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootSubnetActor), rootGateway);

        address funderAddress = address(567);
        vm.deal(funderAddress, 1 ether);
        SubnetID memory fundedSubnetId = rootName.createSubnetId((address(rootSubnetActor)));
        vm.startPrank(funderAddress);
        rootGatewayManager.fund{value: 100000}(fundedSubnetId, FvmAddressHelper.from(address(funderAddress)));
        vm.stopPrank();

        vm.prank(address(caller));
        childGatewayMessenger.sendContractXnetMessage{value: 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: childName,
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({subnetId: rootName, rawAddress: FvmAddressHelper.from(caller)}),
                1,
                0
            )
        );

        BottomUpCheckpoint memory checkpoint = createBottomUpCheckpointInChildSubnet();

        submitCheckpointInParentSubnet(checkpoint);
    }

    function createBottomUpCheckpointInChildSubnet() internal returns (BottomUpCheckpoint memory checkpoint) {
        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        BottomUpMsgBatch memory batch = childGatewayGetter.bottomUpMsgBatch(e);
        require(batch.msgs.length==1, "batch length incorrect");

        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        checkpoint = BottomUpCheckpoint({
            subnetID: childName,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: batch.msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        childGatewayCheckpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        return checkpoint;
    }

    function submitCheckpointInParentSubnet(BottomUpCheckpoint memory checkpoint) internal {
        (uint256[] memory parentKeys, address[] memory parentValidators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory parentPubKeys = new bytes[](3);
        bytes[] memory parentSignatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(parentValidators[i], 10 gwei);
            parentPubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(parentKeys[i]);
            vm.prank(parentValidators[i]);
            rootSubnetActorManager.join{value: 10}(parentPubKeys[i]);
        }

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(parentKeys[i], hash);
            parentSignatures[i] = abi.encodePacked(r, s, v);
        }


        vm.startPrank(address(rootSubnetActor));
        rootSubnetActorCheckpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

}
