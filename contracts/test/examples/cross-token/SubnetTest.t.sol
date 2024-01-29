// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import "forge-std/Test.sol";
import "../../../src/examples/cross-token/SubnetTokenBridge.sol";
import "../../../src/examples/cross-token/SubnetUSDCProxy.sol";

contract SubnetTest is Test {
    SubnetUSDCProxy proxy;
    SubnetTokenBridge bridge;
    address parentSubnetUSDC = address(0x123);
    SubnetID parentSubnet;
    address gateway = address(0x456);

    function setUp() public {
        parentSubnet = SubnetID({root: 0, route: new address[](0)});
        bridge = new SubnetTokenBridge(gateway, parentSubnetUSDC, parentSubnet);
        proxy = SubnetUSDCProxy(bridge.getProxyTokenAddress());
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

    function testMint() public {
        // Example: Test mint function of SubnetUSDCProxy
        address to = address(0x2);
        uint256 amount = 1000;

        vm.prank(address(bridge)); // Set msg.sender to bridge address
        proxy.mint(to, amount);

        assertEq(proxy.balanceOf(to), amount, "Minting failed");
    }

    function testGetProxyTokenAddress() public {
        // Test getProxyTokenAddress function of SubnetTokenBridge
        assertEq(bridge.getProxyTokenAddress(), address(proxy), "Proxy token address mismatch");
    }

    function testOnXNetMessageReceived() public {
        // Example: Test onXNetMessageReceived function of SubnetTokenBridge
        // You'll need to simulate the XNet message and expected behavior here
        // This is a placeholder test
        assertTrue(true, "onXNetMessageReceived not implemented");
    }

    function testExtractParameters() public {
        // Test extractParameters function of SubnetTokenBridge
        // This is a placeholder test
        assertTrue(true, "extractParameters not implemented");
    }

    function testDepositTokens() public {
        // Test depositTokens function of SubnetTokenBridge
        // This is a placeholder test
        assertTrue(true, "depositTokens not implemented");
    }
}
