// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/StrategyBase.sol";
import "./ConfigManager.sol";
import "forge-std/console.sol";
import "../src/StrategyManager.sol";
import "openzeppelin-contracts/token/ERC20/IERC20.sol";

contract DeployLSTStrategies is ConfigManager {
    function run() external override {
        console.logAddress(readConfigAddress(".StrategyManager"));
        vm.startBroadcast();

        StrategyBase glifStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".Glif"))
        );

        StrategyBase stfilStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".Stfil"))
        );

        StrategyBase collectifDaoStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".CollectifDao"))
        );

        StrategyBase replStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".Repl"))
        );

        StrategyBase sftProtocolStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".SftProtocol"))
        );

        StrategyBase filetFinanceStrategy = new StrategyBase(
            StrategyManager(readConfigAddress(".StrategyManager")),
            IERC20(readConfigAddress(".FiletFinance"))
        );

        vm.stopBroadcast();

        writeConfig("GlifStrategy", vm.toString(address(glifStrategy)));
        writeConfig("StfilStrategy", vm.toString(address(stfilStrategy)));
        writeConfig(
            "CollectifDaoStrategy",
            vm.toString(address(collectifDaoStrategy))
        );
        writeConfig("ReplStrategy", vm.toString(address(replStrategy)));
        writeConfig(
            "SftProtocolStrategy",
            vm.toString(address(sftProtocolStrategy))
        );
        writeConfig(
            "FiletFinanceStrategy",
            vm.toString(address(filetFinanceStrategy))
        );
    }
}
