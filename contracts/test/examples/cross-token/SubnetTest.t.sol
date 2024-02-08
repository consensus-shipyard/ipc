// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../../src/examples/cross-token/IpcTokenReplica.sol";
import {IntegrationTestBase} from "../../IntegrationTestBase.sol";
import {GatewayDiamond} from "../../../src/GatewayDiamond.sol";
import {SubnetIDHelper} from "../../../src/lib/SubnetIDHelper.sol";

contract SubnetTest is Test, IntegrationTestBase {
    using SubnetIDHelper for SubnetID;

    IpcTokenReplica bridge;
    address parentSubnetUSDC = address(0x123);
    SubnetID parentSubnet;
    address gateway;
    GatewayDiamond public rootGateway;

    function setUp() public override {
        parentSubnet = SubnetID({root: ROOTNET_CHAINID, route: new address[](0)});
        require(parentSubnet.isRoot(), "not root");
        rootGateway = createGatewayDiamond(gatewayParams(parentSubnet));
        gateway = address(rootGateway);
        bridge = new IpcTokenReplica(gateway, parentSubnetUSDC, parentSubnet);
    }

    function testParentSubnetUSDCAddress() public {
        // Test to check if parentSubnetUSDC address is correctly set
        assertEq(bridge.parentSubnetUSDC(), parentSubnetUSDC, "parentSubnetUSDC address does not match");
    }

    function testParentSubnet() public {
        // Test if parentSubnet is correctly set
        assertEq(bridge.getParentSubnet().root, parentSubnet.root, "parentSubnet.root does not match");
        assertEq(bridge.getParentSubnet().route, parentSubnet.route, "parentSubnet.route does not match");
    }

    function testDepositTokens() public {
        // Test depositTokens function of IpcTokenReplica
        // This is a placeholder test
        assertTrue(true, "depositTokens not implemented");
    }
}
