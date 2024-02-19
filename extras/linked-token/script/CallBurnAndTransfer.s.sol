// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import "../src/LinkedTokenReplica.sol";

contract CallBurnAndTransfer is Script {
    function run(address contractAddress, address recipient, uint256 amount) public {
        LinkedTokenReplica replica = LinkedTokenReplica(contractAddress);
        vm.startBroadcast();
        replica.burnAndTransfer(recipient, amount);
        vm.stopBroadcast();
        // Log the address of the deployed contract
        console.log("done");
    }
}


