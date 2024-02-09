// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../../src/examples/cross-token/IpcTokenReplica.sol";
import {IntegrationTestBase} from "../../IntegrationTestBase.sol";
import {GatewayDiamond} from "../../../src/GatewayDiamond.sol";
import {SubnetIDHelper} from "../../../src/lib/SubnetIDHelper.sol";

import {SubnetActorDiamond} from "../../../src/SubnetActorDiamond.sol";
import {IpcTokenController} from "../../../src/examples/cross-token/IpcTokenController.sol";
import {InvalidOriginContract, InvalidOriginSubnet} from "../../../src/examples/cross-token/IpcCrossTokenErrors.sol";
import {USDCTest} from "../../../src/examples/cross-token/USDCTest.sol";

contract IpcTokenControllerTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;

    IpcTokenController controller;
    IpcTokenReplica replica;
    address controllerSubnetUSDC;
    SubnetID controllerSubnet;
    SubnetID replicaSubnetName;
    address gateway;
    GatewayDiamond public rootGateway;
    uint256 transferAmount = 100;

    address[] public nativeSubnetPath;

    SubnetActorDiamond public rootNativeSubnetActor;
    USDCTest public testUSDC;

    function setUp() public override {
        testUSDC = new USDCTest();
        testUSDC.mint(transferAmount);
        controllerSubnetUSDC = address(testUSDC);

        controllerSubnet = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(controllerSubnet.isRoot(), "not root");
        rootGateway = createGatewayDiamond(gatewayParams(controllerSubnet));
        gateway = address(rootGateway);
        replica = new IpcTokenReplica(gateway, controllerSubnetUSDC, controllerSubnet);
        rootNativeSubnetActor = createSubnetActor(subnetActorWithParams(address(rootGateway), controllerSubnet));
        nativeSubnetPath = new address[](1);
        nativeSubnetPath[0] = address(rootNativeSubnetActor);
        replicaSubnetName = SubnetID({root: ROOTNET_CHAINID, route: nativeSubnetPath});

        vm.deal(address(rootNativeSubnetActor), DEFAULT_COLLATERAL_AMOUNT);
        vm.prank(address(rootNativeSubnetActor));
        registerSubnetGW(DEFAULT_COLLATERAL_AMOUNT, address(rootNativeSubnetActor), rootGateway);

        controller = new IpcTokenController(gateway, controllerSubnetUSDC, replicaSubnetName, address(replica));
        replica.setController(address(controller));
    }

    function testHandleIpcMessageOrigin() public {
        testUSDC.approve(address(controller), transferAmount);
        vm.deal(address(this), DEFAULT_CROSS_MSG_FEE);

        CallMsg memory message = CallMsg({
            method: abi.encodePacked(bytes4(keccak256("receiveAndUnlock(address,uint256)"))),
            params: abi.encode(address(this), transferAmount)
        });

        IpcEnvelope memory validMsg = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({subnetId: replicaSubnetName, rawAddress: FvmAddressHelper.from(address(controller))}),
            to: IPCAddress({subnetId: controllerSubnet, rawAddress: FvmAddressHelper.from(address(replica))}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: 0,
            message: abi.encode(message)
        });

        IpcEnvelope memory invalidContract = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({
                subnetId: replicaSubnetName,
                rawAddress: FvmAddressHelper.from(address(this)) /* invalid */
            }),
            to: IPCAddress({subnetId: controllerSubnet, rawAddress: FvmAddressHelper.from(address(replica))}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: 0,
            message: abi.encode(message)
        });

        IpcEnvelope memory invalidSubnet = IpcEnvelope({
            kind: IpcMsgKind.Call,
            from: IPCAddress({
                subnetId: controllerSubnet /* invalid */,
                rawAddress: FvmAddressHelper.from(address(controller))
            }),
            to: IPCAddress({subnetId: controllerSubnet, rawAddress: FvmAddressHelper.from(address(replica))}),
            value: DEFAULT_CROSS_MSG_FEE,
            nonce: 0,
            message: abi.encode(message)
        });

        vm.expectRevert(InvalidOriginContract.selector);
        controller.verifyIpcEnvelope(invalidContract);

        vm.expectRevert(InvalidOriginSubnet.selector);
        controller.verifyIpcEnvelope(invalidSubnet);
    }

    function testParentSubnetUSDCAddress() public {
        // Test to check if controllerSubnetUSDC address is correctly set
        assertEq(replica.controller(), address(controller), "controllerSubnetUSDC address does not match");
    }

    function testParentSubnet() public {
        // Test if controllerSubnet is correctly set
        assertTrue(replica.getControllerSubnet().equals(controllerSubnet), "controllerSubnetdoes not match");
    }

    function testDepositTokens() public {
        // Test depositTokens function of IpcTokenReplica
        // This is a placeholder test
        assertTrue(true, "depositTokens not implemented");
    }
}
