// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../src/LinkedTokenController.sol";

contract CallLockAndTransfer is Script {
    function run(address contractAddress, address recipient, uint256 amount) public {
        LinkedTokenController controller = LinkedTokenController(contractAddress);
        vm.startBroadcast();
        controller.linkedTransfer(recipient, amount);
        vm.stopBroadcast();
        // Log the address of the deployed contract
        console.log("done");
    }
}


