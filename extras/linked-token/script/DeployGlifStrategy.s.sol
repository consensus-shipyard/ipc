// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/StrategyBase.sol";
import "./ConfigManager.sol";
import "forge-std/console.sol";
import "../src/StrategyManager.sol";
import "openzeppelin-contracts/token/ERC20/IERC20.sol";

contract DeployGlifStrategy is ConfigManager {
    function run() external override {
        console.logAddress(readConfigAddress(".StrategyManager"));
        vm.startBroadcast();

        StrategyBase glifStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".Glif"))
        );

        vm.stopBroadcast();

        writeConfig("GlifStrategy", vm.toString(address(glifStrategy)));
    }
}
