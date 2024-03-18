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
        vm.stopBroadcast();

        // Log the address of the deployed contract implementation
        writeConfig("LinkedTokenReplicaImplementation", vm.toString(address(initialImplementation)));
    }

    function deployIpcTokenReplicaProxy(address replica) external{

        vm.startBroadcast();
        TransparentUpgradeableProxy transparentProxy = new TransparentUpgradeableProxy(replica, address(msg.sender), "");
        vm.stopBroadcast();

        // Log the address of the deployed contract proxy
        writeConfig("LinkedTokenController", vm.toString(address(transparentProxy)));
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

