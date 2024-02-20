// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "../src/LinkedTokenReplica.sol";
import "./ConfigManager.sol";
import "@ipc/src/structs/Subnet.sol";

contract DeployIpcTokenReplica is ConfigManager {
    function run(address gateway, address USDCTest, uint64 _rootNetChainId, address[] memory _route) external {
        SubnetID memory controllerSubnet = SubnetID({root: _rootNetChainId , route: _route});

        vm.startBroadcast();
        LinkedTokenReplica replica = new LinkedTokenReplica(gateway, USDCTest, controllerSubnet);
        vm.stopBroadcast();

        writeConfig("LinkedTokenReplica", vm.toString(address(replica)));
    }
}

