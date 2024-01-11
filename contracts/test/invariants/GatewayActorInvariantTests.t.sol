// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {StdInvariant} from "forge-std/StdInvariant.sol";
import {GatewayDiamond} from "../../src/GatewayDiamond.sol";
import {L1GatewayActorDiamond, L2GatewayActorDiamond, L3GatewayActorDiamond} from "../IntegrationTestPresets.sol";
import {GatewayActorHandler} from "./handlers/GatewayActorHandler.sol";
import {GatewayActorBasicProperties} from "./GatewayActorProperties.sol";

contract GatewayActorInvariantTests is StdInvariant, L1GatewayActorDiamond, GatewayActorBasicProperties {
    GatewayActorHandler private gatewayActorHandler;

    function setUp() public override {
        L1GatewayActorDiamond.setUp();
        gatewayActorHandler = new GatewayActorHandler(gatewayDiamond);
        targetContract(address(gatewayActorHandler));

        // assert specific properties of the infrastructure.
        assertEq(gwGetter.getNetworkName().route.length, 1);
    }
}

contract L2GatewayActorInvariantTests is L2GatewayActorDiamond, GatewayActorBasicProperties {
    GatewayActorHandler private gatewayActorHandler;

    function setUp() public override {
        L2GatewayActorDiamond.setUp();
        gatewayActorHandler = new GatewayActorHandler(gatewayDiamond);
        targetContract(address(gatewayActorHandler));

        // assert specific properties of the infrastructure.
        assertEq(gwGetter.getNetworkName().route.length, 2);
    }
}

contract L3GatewayActorInvariantTests is L3GatewayActorDiamond, GatewayActorBasicProperties {
    GatewayActorHandler private gatewayActorHandler;

    function setUp() public override {
        L3GatewayActorDiamond.setUp();
        gatewayActorHandler = new GatewayActorHandler(gatewayDiamond);
        targetContract(address(gatewayActorHandler));

        // assert specific properties of the infrastructure.
        assertEq(gwGetter.getNetworkName().route.length, 3);
    }
}
