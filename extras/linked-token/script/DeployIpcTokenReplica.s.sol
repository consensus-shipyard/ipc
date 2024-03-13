// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "../src/LinkedTokenReplica.sol";
import "./ConfigManager.sol";
import "@ipc/src/structs/Subnet.sol";
import "openzeppelin-contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

contract DeployIpcTokenReplica is ConfigManager {
    function deployIpcTokenReplica() external {

        vm.startBroadcast();
        LinkedTokenReplica initialImplementation = new LinkedTokenReplica();
        TransparentUpgradeableProxy transparentProxy = new TransparentUpgradeableProxy(address(initialImplementation), address(this), "");
        LinkedTokenReplica replica = LinkedTokenReplica(address(transparentProxy));
        vm.stopBroadcast();

        // Log the address of the deployed contract
        writeConfig("LinkedTokenController", vm.toString(address(replica)));
    }

    function initializeIpcTokenReplica(address replicaProxy, address gateway, address tokenContractAddress, uint64 _rootNetChainId, address[] memory _route, address linkedContract) external {

        // Example for setting up the SubnetID, adjust according to your actual setup
        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});

        vm.startBroadcast();
        LinkedTokenReplica replica = LinkedTokenReplica(replicaProxy);
        replica.initialize(gateway, tokenContractAddress, destinationSubnet, linkedContract);
        vm.stopBroadcast();
    }
}

