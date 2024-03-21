// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "forge-std/Test.sol";
import "../src/LinkedTokenDiamond.sol";
import {IntegrationTestBase} from "@ipc/test/IntegrationTestBase.sol";
import {GatewayDiamond} from "@ipc/src/GatewayDiamond.sol";
import {SubnetIDHelper} from "@ipc/src/lib/SubnetIDHelper.sol";
import {SubnetID, IPCAddress} from "@ipc/src/structs/Subnet.sol";
import {FvmAddressHelper} from "@ipc/src/lib/FvmAddressHelper.sol";
import {FvmAddress} from "@ipc/src/structs/FvmAddress.sol";

import "../src/LinkedTokenControllerFacet.sol";
import "../src/LinkedTokenReplicaFacet.sol";
import "@ipc/src/diamond/DiamondCutFacet.sol";
import "@ipc/src/diamond/DiamondLoupeFacet.sol";
import "@ipc/src/OwnershipFacet.sol";

import "./../script/SelectorLibrary.sol";

import {
    InvalidOriginContract,
    InvalidOriginSubnet
} from "../src/LinkedTokenFacet.sol";


import {IpcEnvelope, CallMsg, IpcMsgKind} from "@ipc/src/structs/CrossNet.sol";

import {SubnetActorDiamond} from "@ipc/src/SubnetActorDiamond.sol";
import {LinkedTokenDiamond} from "../src/LinkedTokenDiamond.sol";
//import {InvalidOriginContract, InvalidOriginSubnet} from "../src/@ipc/src/examples/cross-token/IpcCrossTokenErrors.sol";
import {USDCTest} from "../src/USDCTest.sol";

contract IpcTokenControllerTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;

    LinkedTokenControllerFacet controller;
    LinkedTokenReplicaFacet replica;
    LinkedTokenDiamond controllerDiamond;
    LinkedTokenDiamond replicaDiamond;
    address controllerSubnetUSDC;
    SubnetID controllerSubnet;
    SubnetID replicaSubnetName;
    address gateway;
    GatewayDiamond public rootGateway;
    uint256 transferAmount = 100;
    using FvmAddressHelper for FvmAddress;

    address[] public nativeSubnetPath;

    SubnetActorDiamond public rootNativeSubnetActor;
    USDCTest public testUSDC;

    function setUp() public override {
        testUSDC = new USDCTest();
        testUSDC.mint(transferAmount);
        controllerSubnetUSDC = address(testUSDC);

        controllerSubnet = SubnetID({
            root: ROOTNET_CHAINID,
            route: new address[](0)
        });
        require(controllerSubnet.isRoot(), "not root");
        rootGateway = createGatewayDiamond(gatewayParams(controllerSubnet));
        gateway = address(rootGateway);
        rootNativeSubnetActor = createSubnetActor(
            defaultSubnetActorParamsWith(address(rootGateway), controllerSubnet)
        );
        nativeSubnetPath = new address[](1);
        nativeSubnetPath[0] = address(rootNativeSubnetActor);
        replicaSubnetName = SubnetID({
            root: ROOTNET_CHAINID,
            route: nativeSubnetPath
        });

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(
            DEFAULT_COLLATERAL_AMOUNT,
            address(rootNativeSubnetActor),
            rootGateway
        );
/*
        controller = new LinkedTokenController(
            gateway,
            controllerSubnetUSDC,
            replicaSubnetName
        );

        replica = new LinkedTokenReplica(
            gateway,
            controllerSubnetUSDC,
            controllerSubnet
        );
        replica.initialize(address(controller));
        controller.initialize(address(replica));
*/
        //replica, controller = setUpLinkedTokenContracts(gatway, controllerSubnetUSDC, replicaSubnetName, controllerSubnet);
    }


    function setUpLinkedTokenContracts (address gateway, address controllerSubnetUSDC, SubnetID memory replicaSubnetName,  SubnetID memory controllerSubnet) internal returns (address, address) {

        //Controller 

        // Deploy controller facets
        DiamondCutFacet cutFacetC = new DiamondCutFacet();
        DiamondLoupeFacet loupeFacetC = new DiamondLoupeFacet();
        OwnershipFacet ownershipFacetC = new OwnershipFacet();
        LinkedTokenControllerFacet linkedTokenControllerFacetC = new LinkedTokenControllerFacet();

        // controller diamond constructor params
        LinkedTokenDiamond.ConstructorParams memory paramsController;
        paramsController.gateway=gateway;
        paramsController.underlyingToken=controllerSubnetUSDC;
        paramsController.linkedSubnet = replicaSubnetName;

        // Prepare diamond cut with all facets
        IDiamond.FacetCut[] memory cuts = new IDiamond.FacetCut[](4);

        cuts[0] = createCut(address(cutFacetC), SelectorLibrary.resolveSelectors("DiamondCutFacet"));
        cuts[1] = createCut(address(loupeFacetC), SelectorLibrary.resolveSelectors("DiamondLoupeFacet"));
        cuts[2] = createCut(address(ownershipFacetC), SelectorLibrary.resolveSelectors("OwnershipFacet"));
        cuts[3] = createCut(address(linkedTokenControllerFacetC), SelectorLibrary.resolveSelectors("LinkedTokenControllerFacet"));
        //
        // Deploy the diamond with all facet cuts

        controllerDiamond = new LinkedTokenDiamond(cuts, paramsController);

      

        //Replica
        
        //Deploy replica facets
        DiamondCutFacet cutFacetR = new DiamondCutFacet();
        DiamondLoupeFacet loupeFacetR = new DiamondLoupeFacet();
        OwnershipFacet ownershipFacetR = new OwnershipFacet();
        LinkedTokenReplicaFacet linkedTokenReplicaFacetR = new LinkedTokenReplicaFacet();

        // replica diamond constructor params
        LinkedTokenDiamond.ConstructorParams memory paramsReplica;
        paramsReplica.gateway=gateway;
        paramsReplica.underlyingToken=controllerSubnetUSDC;
        paramsReplica.linkedSubnet = controllerSubnet;


        // Prepare diamond cut with all facets
        IDiamond.FacetCut[] memory cutsR = new IDiamond.FacetCut[](4);

        cutsR[0] = createCut(address(cutFacetR), SelectorLibrary.resolveSelectors("DiamondCutFacet"));
        cutsR[1] = createCut(address(loupeFacetR), SelectorLibrary.resolveSelectors("DiamondLoupeFacet"));
        cutsR[2] = createCut(address(ownershipFacetR), SelectorLibrary.resolveSelectors("OwnershipFacet"));
        cutsR[3] = createCut(address(linkedTokenReplicaFacetR), SelectorLibrary.resolveSelectors("LinkedTokenReplicaFacetR"));
        //
        // Deploy the diamond with all facet cuts


        replicaDiamond = new LinkedTokenDiamond(cutsR, paramsReplica);
    }

    function testHandleIpcMessageOrigin() public {
        testUSDC.approve(address(controller), transferAmount);
        vm.deal(address(this), DEFAULT_CROSS_MSG_FEE);

        CallMsg memory message =
            CallMsg({
                method: abi.encodePacked(
                    bytes4(keccak256("receiveAndUnlock(address,uint256)"))
                ),
                params: abi.encode(address(this), transferAmount)
            });

        IpcEnvelope memory validMsg =
            IpcEnvelope({
                kind: IpcMsgKind.Call,
                from: IPCAddress({
                    subnetId: replicaSubnetName,
                    rawAddress: FvmAddressHelper.from(address(controller))
                }),
                to: IPCAddress({
                    subnetId: controllerSubnet,
                    rawAddress: FvmAddressHelper.from(address(replica))
                }),
                value: DEFAULT_CROSS_MSG_FEE,
                nonce: 0,
                message: abi.encode(message)
            });

        IpcEnvelope memory invalidContract =
            IpcEnvelope({
                kind: IpcMsgKind.Call,
                from: IPCAddress({
                    subnetId: replicaSubnetName,
                    rawAddress: FvmAddressHelper.from(address(this)) /* invalid */
                }),
                to: IPCAddress({
                    subnetId: controllerSubnet,
                    rawAddress: FvmAddressHelper.from(address(replica))
                }),
                value: DEFAULT_CROSS_MSG_FEE,
                nonce: 0,
                message: abi.encode(message)
            });

        IpcEnvelope memory invalidSubnet =
            IpcEnvelope({
                kind: IpcMsgKind.Call,
                from: IPCAddress({
                    subnetId: controllerSubnet, /* invalid */
                    rawAddress: FvmAddressHelper.from(address(controller))
                }),
                to: IPCAddress({
                    subnetId: controllerSubnet,
                    rawAddress: FvmAddressHelper.from(address(replica))
                }),
                value: DEFAULT_CROSS_MSG_FEE,
                nonce: 0,
                message: abi.encode(message)
            });

        vm.expectRevert(InvalidOriginContract.selector);
        controller._validateEnvelope(invalidContract);

        vm.expectRevert(InvalidOriginSubnet.selector);
        controller._validateEnvelope(invalidSubnet);
    }

    function testParentSubnetUSDCAddress() public {
        // Test to check if controllerSubnetUSDC address is correctly set
        assertEq(
            controller.getLinkedContract(),
            address(replica),
            "controllerSubnetUSDC address does not match"
        );
    }

    function testParentSubnet() public {
        assertTrue(
            controllerSubnet.equals(replica.getLinkedSubnet()),
            "replica Subnetdoes not match"
        );
        assertTrue(
            replicaSubnetName.equals(controller.getLinkedSubnet()),
            "controller Subnetdoes not match"
        );
    }

    function testDepositTokens() public {
        // Test depositTokens function of IpcTokenReplica
        // This is a placeholder test
        assertTrue(true, "depositTokens not implemented");
    }

    function createCut(address _facet, bytes4[] memory _selectors) internal pure returns (IDiamond.FacetCut memory cut) {
        return IDiamond.FacetCut({
            facetAddress: _facet,
            action: IDiamond.FacetCutAction.Add,
            functionSelectors: _selectors
        });
    }
}
