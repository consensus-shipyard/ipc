// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "./ConfigManager.sol";
import "../src/LinkedTokenController.sol";
import "@ipc/src/structs/Subnet.sol";

contract DeployIpcTokenController is ConfigManager {
    function run(address gateway, address tokenContractAddress, uint64 _rootNetChainId, address[] memory _route) external {
        // Example for setting up the SubnetID, adjust according to your actual setup
        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});

        vm.startBroadcast();
        LinkedTokenController controller = new LinkedTokenController(gateway, tokenContractAddress, destinationSubnet);
        vm.stopBroadcast();

        // Log the address of the deployed contract
        writeConfig("LinkedTokenController", vm.toString(address(controller)));
    }
}

