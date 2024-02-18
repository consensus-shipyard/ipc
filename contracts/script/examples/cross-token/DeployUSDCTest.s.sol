// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "forge-std/Script.sol";
import "../../../src/examples/cross-token/USDCTest.sol";

contract DeployUSDCTest is Script {
    function run() external {
        vm.startBroadcast();

        USDCTest erc20Token = new USDCTest();

        vm.stopBroadcast();
    }
}
