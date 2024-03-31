// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/StrategyBase.sol";
import "./ConfigManager.sol";
import "forge-std/console.sol";
import "../src/StrategyManager.sol";
import "openzeppelin-contracts/token/ERC20/IERC20.sol";

contract DeployStrategies is ConfigManager {
    function run() external override {
        console.logAddress(readConfigAddress(".StrategyManager"));
        console.logAddress(readConfigAddress(".WFil"));
        vm.startBroadcast();

        StrategyBase wFilStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".WFil"))
        );

        vm.stopBroadcast();

        writeConfig("WFilStrategy", vm.toString(address(wFilStrategy)));
    }
}
