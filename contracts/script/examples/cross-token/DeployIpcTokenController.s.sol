// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../../../src/examples/cross-token/IpcTokenController.sol";
import "../../../src/structs/Subnet.sol";

contract DeployIpcTokenController is Script {
    function run(address _gateway, address _tokenContractAddress, uint64 _rootNetChainId, address[] memory _route, address _destinationContract) external {
        // Example for setting up the SubnetID, adjust according to your actual setup
        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});

        vm.startBroadcast();

        // Deploy the IpcTokenController contract
        IpcTokenController controller = new IpcTokenController(_gateway, _tokenContractAddress, destinationSubnet, _destinationContract);

        vm.stopBroadcast();


        // Log the address of the deployed contract
        console.log("IpcTokenController deployed at:", address(controller));
    }
}

