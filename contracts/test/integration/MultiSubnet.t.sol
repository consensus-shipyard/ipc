// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../src/errors/IPCErrors.sol";
import {EMPTY_BYTES, METHOD_SEND} from "../../src/constants/Constants.sol";
import {IpcEnvelope} from "../../src/structs/CrossNet.sol";
import {FvmAddress} from "../../src/structs/FvmAddress.sol";
import {SubnetID, Subnet, IPCAddress, Validator} from "../../src/structs/Subnet.sol";
import {SubnetIDHelper} from "../../src/lib/SubnetIDHelper.sol";
import {FvmAddressHelper} from "../../src/lib/FvmAddressHelper.sol";
import {CrossMsgHelper} from "../../src/lib/CrossMsgHelper.sol";
import {GatewayDiamond, FEATURE_MULTILEVEL_CROSSMSG} from "../../src/GatewayDiamond.sol";
import {SubnetActorDiamond} from "../../src/SubnetActorDiamond.sol";
import {SubnetActorGetterFacet} from "../../src/subnet/SubnetActorGetterFacet.sol";
import {GatewayGetterFacet} from "../../src/gateway/GatewayGetterFacet.sol";
import {GatewayManagerFacet} from "../../src/gateway/GatewayManagerFacet.sol";
import {XnetMessagingFacet} from "../../src/gateway/router/XnetMessagingFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {GatewayMessengerFacet} from "../../src/gateway/GatewayMessengerFacet.sol";
import {DiamondLoupeFacet} from "../../src/diamond/DiamondLoupeFacet.sol";
import {DiamondCutFacet} from "../../src/diamond/DiamondCutFacet.sol";
import {IntegrationTestBase} from "../IntegrationTestBase.sol";
import {L2GatewayActorDiamond, L1GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {TestUtils, MockIpcContract} from "../helpers/TestUtils.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
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
    GatewayGetterFacet public childGatewayGetter;

    GatewayMessengerFacet public parentGatewayMessenger;
    GatewayMessengerFacet public childGatewayMessenger;

    address[] public parentPath;
    address[] public childPath;

    SubnetID parentId;
    SubnetID childId;

    function setUp() public override {
        parentPath = new address[](1);
        parentPath[0] = vm.addr(10);

        parentId = SubnetID({root: ROOTNET_CHAINID, route: parentPath});

        childPath = new address[](2);
        childPath[0] = vm.addr(10);
        childPath[1] = vm.addr(20);

        childId = SubnetID({root: ROOTNET_CHAINID, route: childPath});

        parentGateway = createGatewayDiamond(gatewayParams(parentId));
        childGateway = createGatewayDiamond(gatewayParams(childId));

        parentSubnetActor = createSubnetActor(subnetActorWithParams(address(parentGateway), parentId));
        childSubnetActor = createSubnetActor(subnetActorWithParams(address(childGateway), childId));

        parentGatewayGetter = GatewayGetterFacet(address(parentGateway));
        childGatewayGetter = GatewayGetterFacet(address(childGateway));
        parentGatewayMessenger = GatewayMessengerFacet(address(parentGateway));
        childGatewayMessenger = GatewayMessengerFacet(address(childGateway));

        parentSubnetActorGetter = SubnetActorGetterFacet(address(parentSubnetActor));
        childSubnetActorGetter = SubnetActorGetterFacet(address(childSubnetActor));
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
        vm.startPrank(caller);
        vm.deal(caller, DEFAULT_COLLATERAL_AMOUNT + DEFAULT_CROSS_MSG_FEE + 2);

        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, caller, childGateway);

        SubnetID memory destinationSubnet = SubnetID(0, new address[](0));

        childGatewayMessenger.sendContractXnetMessage{value: 1}(
            TestUtils.newXnetCallMsg(
                IPCAddress({
                    subnetId: childId,
                    rawAddress: FvmAddressHelper.from(caller)
                }),
                IPCAddress({subnetId: parentId, rawAddress: FvmAddressHelper.from(caller)}),
                1,
                0
            )
        );
    }



}
