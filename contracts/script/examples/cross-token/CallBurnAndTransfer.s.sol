// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../../../src/examples/cross-token/IpcTokenReplica.sol";
import "../../../src/structs/Subnet.sol";

contract CallBurnAndTransfer is Script {
    function run(address contractAddress, address recipient, uint256 amount) public {
        IpcTokenReplica replica = IpcTokenReplica(contractAddress);
        vm.startBroadcast();
        replica.burnAndTransfer(recipient, amount);
        vm.stopBroadcast();
        // Log the address of the deployed contract
        console.log("done");
    }
}


