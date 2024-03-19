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

        // Log the address of the deployed contract implementation
        writeConfig("LinkedTokenControllerImplementation", vm.toString(address(initialImplementation)));
    }


    function deployIpcTokenControllerProxy(address controller) external{

        vm.startBroadcast();
        TransparentUpgradeableProxy transparentProxy = new TransparentUpgradeableProxy(address(controller), address(msg.sender), "");
        vm.stopBroadcast();

        // Log the address of the deployed contract proxy
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

    function upgradeIpcTokenController(address controllerProxy, address newControllerImplementation) external {
        vm.startBroadcast();
        LinkedTokenController controller = LinkedTokenController(controllerProxy);
        controller.upgradeToAndCall(newControllerImplementation, "");
        vm.stopBroadcast();
    }
}

