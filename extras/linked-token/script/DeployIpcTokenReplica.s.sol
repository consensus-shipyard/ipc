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

    function deployIpcTokenReplicaProxy(address initialImplementation, address gateway, address tokenContractAddress, uint64 _rootNetChainId, address[] memory _route) external {

        vm.startBroadcast();

        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});

        bytes memory initCall = abi.encodeCall(LinkedTokenReplica.initialize, (gateway, tokenContractAddress, destinationSubnet, 0x0000000000000000000000000000000000000000));
        TransparentUpgradeableProxy transparentProxy = new TransparentUpgradeableProxy(initialImplementation, address(msg.sender), initCall);
        vm.stopBroadcast();
        writeConfig("LinkedTokenReplica", vm.toString(address(transparentProxy)));
    }

    function upgradeIpcTokenReplica(address replicaProxy, address newReplicaImplementation, address gateway, address tokenContractAddress, uint64 _rootNetChainId, address[] memory _route, address controllerProxy) external {
        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});
        bytes memory initCall = abi.encodeCall(LinkedTokenReplica.reinitialize, (gateway, tokenContractAddress, destinationSubnet, controllerProxy));

        vm.startBroadcast();
        LinkedTokenReplica replica = LinkedTokenReplica(replicaProxy);
        replica.upgradeToAndCall(newReplicaImplementation, initCall);
        vm.stopBroadcast();
    }
}

