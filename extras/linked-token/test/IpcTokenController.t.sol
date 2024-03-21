// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;
import {console} from "forge-std/console.sol";

import "forge-std/Test.sol";
import "../src/LinkedTokenDiamond.sol";
import {IntegrationTestBase} from "@ipc/test/IntegrationTestBase.sol";
import {GatewayDiamond} from "@ipc/src/GatewayDiamond.sol";
import {SubnetIDHelper} from "@ipc/src/lib/SubnetIDHelper.sol";
import {SubnetID, IPCAddress} from "@ipc/src/structs/Subnet.sol";
import {FvmAddressHelper} from "@ipc/src/lib/FvmAddressHelper.sol";
import {FvmAddress} from "@ipc/src/structs/FvmAddress.sol";

import {USDCTest} from "../src/USDCTest.sol";
import "../src/LinkedTokenFacet.sol";
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

import { LinkedTokenTestBase} from "./LinkedTokenTestBase.t.sol";

contract IpcTokenControllerTest is Test, IntegrationTestBase, LinkedTokenTestBase {
    using SubnetIDHelper for SubnetID;

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
         setUpLinkedTokenContracts(gateway, gateway, controllerSubnetUSDC, replicaSubnetName, controllerSubnet);
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


        //TODO investigate valid envelope and the correct type in the invalidContract revert

        vm.expectRevert();
        LinkedTokenControllerFacet(address(controller))._validateEnvelope(invalidContract);

        vm.expectRevert(InvalidOriginSubnet.selector);
        LinkedTokenControllerFacet(address(controller))._validateEnvelope(invalidSubnet);
    }

    function testParentSubnetUSDCAddress() public {
        // Test to check if controllerSubnetUSDC address is correctly set
        assertEq(
            LinkedTokenControllerFacet(address(controller)).getLinkedContract(),
            address(replica),
            "controllerSubnetUSDC address does not match"
        );
    }

    // XXX TODO investigate
    function _testParentSubnet() public {
        assertTrue(
            controllerSubnet.equals(LinkedTokenReplicaFacet(address(replica)).getLinkedSubnet()),
            "replica Subnetdoes not match"
        );
        assertTrue(
            replicaSubnetName.equals(LinkedTokenControllerFacet(address(controller)).getLinkedSubnet()),
            "controller Subnetdoes not match"
        );
    }

    function testControllerGatewayGet() public {
        // Test to check if controllerSubnetUSDC address is correctly set
        assertEq(
            LinkedTokenControllerFacet(address(controller)).getLinkedGateway(),
            address(gateway),
            "controller gateway address does not match"
        );
    }



    function testDepositTokens() public {
        // Test depositTokens function of IpcTokenReplica
        // This is a placeholder test
        assertTrue(true, "depositTokens not implemented");
    }

}
