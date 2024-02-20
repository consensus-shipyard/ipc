// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../src/LinkedTokenController.sol";
import "@ipc/src/structs/Subnet.sol";

contract DeployIpcTokenController is Script {
    function run(address _gateway, address _tokenContractAddress, uint64 _rootNetChainId, address[] memory _route, address _destinationContract) external {
        // Example for setting up the SubnetID, adjust according to your actual setup
        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});

        vm.startBroadcast();
        LinkedTokenController controller = new LinkedTokenController(_gateway, _tokenContractAddress, destinationSubnet);
        vm.stopBroadcast();

        // Log the address of the deployed contract
        console.log("IpcTokenController deployed at:", address(controller));
    }
}

