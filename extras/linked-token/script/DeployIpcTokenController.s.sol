// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "./ConfigManager.sol";
import "../src/LinkedTokenController.sol";
import "@ipc/src/structs/Subnet.sol";
import "openzeppelin-contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

contract DeployIpcTokenController is ConfigManager {
    function deployIpcTokenController() external {

        vm.startBroadcast();
        LinkedTokenController initialImplementation = new LinkedTokenController();
        vm.stopBroadcast();

        vm.startBroadcast();
        TransparentUpgradeableProxy transparentProxy = new TransparentUpgradeableProxy(address(initialImplementation), address(msg.sender), "");
        vm.stopBroadcast();

        // Log the address of the deployed contract
        writeConfig("LinkedTokenController", vm.toString(address(transparentProxy)));
    }

    function initializeIpcTokenController(address controllerProxy, address gateway, address tokenContractAddress, uint64 _rootNetChainId, address[] memory _route, address linkedContract) external {

        // Example for setting up the SubnetID, adjust according to your actual setup
        SubnetID memory destinationSubnet = SubnetID({root: _rootNetChainId, route: _route});

        vm.startBroadcast();
        LinkedTokenController controller = LinkedTokenController(controllerProxy);
        controller.initialize(gateway, tokenContractAddress, destinationSubnet, linkedContract);
        vm.stopBroadcast();
    }
}

