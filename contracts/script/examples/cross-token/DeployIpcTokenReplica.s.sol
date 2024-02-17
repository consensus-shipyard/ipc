// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../../../src/examples/cross-token/IpcTokenReplica.sol";
import "../../../src/structs/Subnet.sol";

contract DeployIpcTokenReplica is Script {
    function run(address _gateway, address _controller, uint64 _rootNetChainId, address[] memory _route) external {
        // Assuming SubnetID is a struct and can be instantiated like this
        SubnetID memory controllerSubnet = SubnetID({root: _rootNetChainId , route: _route});
        uint256 privateKey = vm.envUint("PRIVATE_KEY");


        vm.startBroadcast(privateKey);

        // Deploy the IpcTokenReplica contract
        IpcTokenReplica replica = new IpcTokenReplica(_gateway, _controller, controllerSubnet);

        vm.stopBroadcast();

        // Log the address of the deployed contract
        console.log("IpcTokenReplica deployed at:", address(replica));
    }
}

