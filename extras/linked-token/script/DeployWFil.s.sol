// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/WFil.sol";
import "./ConfigManager.sol";

contract DeployWFil is ConfigManager {
    function run() external override {
        vm.startBroadcast();

        WFil erc20Token = new WFil();

        vm.stopBroadcast();

        writeConfig("USDCTest", vm.toString(address(erc20Token)));
    }
}
