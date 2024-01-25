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

    GatewayDiamond public parentGateway;
    GatewayDiamond public childGateway;
    SubnetActorDiamond public parentSubnetActor;
    SubnetActorDiamond public childSubnetActor;

    SubnetActorGetterFacet public parentSubnetActorGetter;
    SubnetActorGetterFacet public childSubnetActorGetter;

    GatewayGetterFacet public parentGatewayGetter;
    GatewayManagerFacet public parentGatewayManager;
    GatewayGetterFacet public childGatewayGetter;
    CheckpointingFacet public childGatewayCheckpointer;

    GatewayMessengerFacet public parentGatewayMessenger;
    GatewayMessengerFacet public childGatewayMessenger;

    SubnetActorCheckpointingFacet public parentSubnetActorCheckpointer;
    SubnetActorManagerFacet public parentSubnetActorManager;

    address[] public parentPath;
    address[] public childPath;

    SubnetID parentId;
    SubnetID childId;

    function setUp() public override {
        parentPath = new address[](1);
        parentPath[0] = vm.addr(10);

        parentId = SubnetID({root: ROOTNET_CHAINID, route: parentPath});

        parentGateway = createGatewayDiamond(gatewayParams(parentId));
        parentSubnetActor = createSubnetActor(subnetActorWithParams(address(parentGateway), parentId));
        parentGatewayGetter = GatewayGetterFacet(address(parentGateway));
        parentGatewayMessenger = GatewayMessengerFacet(address(parentGateway));
        parentGatewayManager = GatewayManagerFacet(address(parentGateway));
        parentSubnetActorGetter = SubnetActorGetterFacet(address(parentSubnetActor));
        parentSubnetActorCheckpointer = SubnetActorCheckpointingFacet(address(parentSubnetActor));
        parentSubnetActorManager = SubnetActorManagerFacet(address(parentSubnetActor));

        childPath = new address[](2);
        childPath[0] = vm.addr(10);
        childPath[1] = address(parentSubnetActor);

        childId = SubnetID({root: ROOTNET_CHAINID, route: childPath});
        childGateway = createGatewayDiamond(gatewayParams(childId));
        childSubnetActor = createSubnetActor(subnetActorWithParams(address(childGateway), childId));
        childGatewayGetter = GatewayGetterFacet(address(childGateway));
        childGatewayMessenger = GatewayMessengerFacet(address(childGateway));
        childSubnetActorGetter = SubnetActorGetterFacet(address(childSubnetActor));
        childGatewayCheckpointer = CheckpointingFacet(address(childGateway));


        console.logAddress(childId.getActor());
        console.logAddress(parentId.getActor());
        console.logAddress((address(parentSubnetActor)));
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


    function testGatewayDiamond_MultiSubnet_Baseline() public {
        console.logString(parentGatewayGetter.getNetworkName().toString());
        console.logString(childGatewayGetter.getNetworkName().toString());

        //console.logInt(parentSubnetActorGetter.consensus());
        console.logString(parentSubnetActorGetter.getParent().toString());

        //console.logInt(childSubnetActorGetter.consensus());
        console.logString(childSubnetActorGetter.getParent().toString());

    }

    function testGatewayDiamond_MultiSubnet_SendCrossMessageFromChildToParent() public {
        // Caller of general-purpose messages must be a contract, not a EoA

        address caller = address(new MockIpcContract());

        //vm.deal(address(parentSubnetActor), 2*DEFAULT_COLLATERAL_AMOUNT +2*DEFAULT_CROSS_MSG_FEE + 2);
        vm.deal(address(parentSubnetActor), 2*DEFAULT_COLLATERAL_AMOUNT +2*DEFAULT_CROSS_MSG_FEE + 2);
        vm.deal(caller, 2*DEFAULT_COLLATERAL_AMOUNT +2*DEFAULT_CROSS_MSG_FEE + 2);

        vm.prank(address(parentSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(parentSubnetActor), childGateway);

        vm.prank(address(parentSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(parentSubnetActor), parentGateway);


        address funderAddress = address(567);
        uint256 fundAmount = 1 ether;

        vm.deal(funderAddress, fundAmount + 1);

        SubnetID memory fundedSubnetId = parentSubnetActorGetter.getParent().createSubnetId(address(parentSubnetActor));

        vm.startPrank(funderAddress);
        parentGatewayManager.fund{value: 100000}(fundedSubnetId, FvmAddressHelper.from(address(funderAddress)));
        vm.stopPrank();

        childGatewayMessenger.sendContractXnetMessage{value: 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: parentId,
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({subnetId: parentId, rawAddress: FvmAddressHelper.from(caller)}),
                1,
                0
            )
        );

        uint256 e = getNextEpoch(block.number, DEFAULT_CHECKPOINT_PERIOD);

        BottomUpMsgBatch memory batch = childGatewayGetter.bottomUpMsgBatch(e);
        require(batch.msgs.length==1, "batch length incorrect");

        (uint256[] memory privKeys, address[] memory addrs, uint256[] memory weights) = TestUtils.getFourValidators(vm);

        (bytes32 membershipRoot, bytes32[][] memory membershipProofs) = MerkleTreeHelper
            .createMerkleProofsForValidators(addrs, weights);

        BottomUpCheckpoint memory checkpoint = BottomUpCheckpoint({
            subnetID: childId,
            blockHeight: batch.blockHeight,
            blockHash: keccak256("block1"),
            nextConfigurationNumber: 0,
            msgs: batch.msgs
        });

        vm.startPrank(FilAddress.SYSTEM_ACTOR);
        childGatewayCheckpointer.createBottomUpCheckpoint(checkpoint, membershipRoot, weights[0] + weights[1] + weights[2]);
        vm.stopPrank();

        // adds signatures

        uint8 v;
        bytes32 r;
        bytes32 s;
        bytes[] memory signatures = new bytes[](4);

        for (uint64 i = 0; i < 4; i++) {
            (v, r, s) = vm.sign(privKeys[i], keccak256(abi.encode(checkpoint)));
            signatures[i] = abi.encodePacked(r, s, v);

            vm.startPrank(vm.addr(privKeys[i]));
            childGatewayCheckpointer.addCheckpointSignature(
                checkpoint.blockHeight,
                membershipProofs[i],
                weights[i],
                signatures[i]
            );
            vm.stopPrank();
        }

        require(childGatewayGetter.getCheckpointInfo(checkpoint.blockHeight).reached == true, "threshold not reached");

        // parent

        submitCheckpointInParent(checkpoint);
    }

    function submitCheckpointInParent(BottomUpCheckpoint memory checkpoint) internal {
        (uint256[] memory parentKeys, address[] memory parentValidators, ) = TestUtils.getThreeValidators(vm);
        bytes[] memory parentPubKeys = new bytes[](3);
        bytes[] memory parentSignatures = new bytes[](3);

        for (uint256 i = 0; i < 3; i++) {
            vm.deal(parentValidators[i], 10 gwei);
            parentPubKeys[i] = TestUtils.deriveValidatorPubKeyBytes(parentKeys[i]);
            vm.prank(parentValidators[i]);
            parentSubnetActorManager.join{value: 10}(parentPubKeys[i]);
        }

        bytes32 hash = keccak256(abi.encode(checkpoint));

        for (uint256 i = 0; i < 3; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(parentKeys[i], hash);
            parentSignatures[i] = abi.encodePacked(r, s, v);
        }


        vm.startPrank(address(parentSubnetActor));
        parentSubnetActorCheckpointer.submitCheckpoint(checkpoint, parentValidators, parentSignatures);
        vm.stopPrank();
    }

    function getNextEpoch(uint256 blockNumber, uint256 checkPeriod) internal pure returns (uint256) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }



}
