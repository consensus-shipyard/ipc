// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import "../src/MinerSlasher.sol";
import "./ConfigManager.sol";
import "forge-std/console.sol";
import "../src/DelegationManager.sol";

contract DeployMinerSlasher is ConfigManager {
    function run() external override {
        console.logAddress(readConfigAddress(".DelegationManager"));
        vm.startBroadcast();

        MinerSlasher minerSlasher = new MinerSlasher(DelegationManager(readConfigAddress(".DelegationManager")));

        vm.stopBroadcast();

        writeConfig("WFilStrategy", vm.toString(address(minerSlasher)));
    }
}
