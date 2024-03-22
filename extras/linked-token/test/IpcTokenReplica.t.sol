// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "forge-std/Test.sol";
import "../src/LinkedTokenReplica.sol";
import {IntegrationTestBase} from "@ipc/test/IntegrationTestBase.sol";
import {GatewayDiamond} from "@ipc/src/GatewayDiamond.sol";
import {SubnetIDHelper} from "@ipc/src/lib/SubnetIDHelper.sol";
import {SubnetID, IPCAddress} from "@ipc/src/structs/Subnet.sol";
import {FvmAddressHelper} from "@ipc/src/lib/FvmAddressHelper.sol";
import {FvmAddress} from "@ipc/src/structs/FvmAddress.sol";

import {IpcEnvelope, CallMsg, IpcMsgKind} from "@ipc/src/structs/CrossNet.sol";

import {SubnetActorDiamond} from "@ipc/src/SubnetActorDiamond.sol";
import {LinkedTokenController} from "../src/LinkedTokenController.sol";
//import {InvalidOriginContract, InvalidOriginSubnet} from "@ipc/src/examples/cross-token/IpcCrossTokenErrors.sol";
import {USDCTest} from "../src/USDCTest.sol";
import {
    InvalidOriginContract,
    InvalidOriginSubnet
} from "../src/LinkedToken.sol";

contract IpcTokenReplicaTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;

    LinkedTokenController controller;
    LinkedTokenReplica replica;
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

        controllerSubnet = SubnetID({
            root: ROOTNET_CHAINID,
            route: new address[](0)
        });
        require(controllerSubnet.isRoot(), "not root");
        rootGateway = createGatewayDiamond(gatewayParams(controllerSubnet));
        gateway = address(rootGateway);
        replica = new LinkedTokenReplica(
            gateway,
            controllerSubnetUSDC,
            controllerSubnet
        );
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

        controller = new LinkedTokenController(
            gateway,
            controllerSubnetUSDC,
            replicaSubnetName
        );
        replica.initialize(address(controller));
        controller.initialize(address(replica));
    }

    function testHandleIpcMessageOrigin() public {
        CallMsg memory message =
            CallMsg({
                method: abi.encodePacked(
                    bytes4(keccak256("receiveAndMint(address,uint256)"))
                ),
                params: abi.encode(address(this), transferAmount)
            });

        IpcEnvelope memory validMsg =
            IpcEnvelope({
                kind: IpcMsgKind.Call,
                from: IPCAddress({
                    subnetId: controllerSubnet,
                    rawAddress: FvmAddressHelper.from(address(replica))
                }),
                to: IPCAddress({
                    subnetId: replicaSubnetName,
                    rawAddress: FvmAddressHelper.from(address(controller))
                }),
                value: DEFAULT_CROSS_MSG_FEE,
                nonce: 0,
                message: abi.encode(message)
            });

        IpcEnvelope memory invalidContract =
            IpcEnvelope({
                kind: IpcMsgKind.Call,
                from: IPCAddress({
                    subnetId: controllerSubnet,
                    rawAddress: FvmAddressHelper.from(address(replica))
                }),
                to: IPCAddress({
                    subnetId: replicaSubnetName,
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
                    subnetId: replicaSubnetName,
                    rawAddress: FvmAddressHelper.from(address(replica))
                }),
                to: IPCAddress({
                    subnetId: replicaSubnetName,
                    rawAddress: FvmAddressHelper.from(address(controller))
                }),
                value: DEFAULT_CROSS_MSG_FEE,
                nonce: 0,
                message: abi.encode(message)
            });

        vm.expectRevert(InvalidOriginContract.selector);
        replica._validateEnvelope(invalidContract);

        vm.expectRevert(InvalidOriginSubnet.selector);
        replica._validateEnvelope(invalidSubnet);
    }
}
