// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../src/LinkedTokenReplica.sol";
import "@ipc/src/structs/Subnet.sol";

contract DeployIpcTokenReplica is Script {
    function run(address _gateway, address _controller, uint64 _rootNetChainId, address[] memory _route) external {
        // Assuming SubnetID is a struct and can be instantiated like this
        SubnetID memory controllerSubnet = SubnetID({root: _rootNetChainId , route: _route});

        vm.startBroadcast();
        LinkedTokenReplica replica = new LinkedTokenReplica(_gateway, _controller, controllerSubnet);
        vm.stopBroadcast();

        // Log the address of the deployed contract
        console.log("IpcTokenReplica deployed at:", address(replica));
    }
}

