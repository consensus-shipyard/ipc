// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/StrategyManager.sol";
import "./ConfigManager.sol";

contract DeployStrategyManager is ConfigManager {
    function run() external override {
        vm.startBroadcast();

        StrategyManager strategyManager = new StrategyManager();

        vm.stopBroadcast();

        writeConfig("StrategyManager", vm.toString(address(strategyManager)));
    }
}
