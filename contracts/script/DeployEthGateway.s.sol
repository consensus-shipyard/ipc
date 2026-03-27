// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Script.sol";
import {EthGatewayMessenger} from "../contracts/gateway/EthGatewayMessenger.sol";
import {SubnetID} from "../contracts/structs/Subnet.sol";

/**
 * @title DeployEthGateway
 * @notice Foundry deploy script for EthGatewayMessenger on Ethereum Sepolia.
 *
 * Usage:
 *   forge script contracts/script/DeployEthGateway.s.sol \
 *     --rpc-url $ETHEREUM_RPC_URL \
 *     --private-key $PRIVATE_KEY \
 *     --broadcast \
 *     --verify
 *
 * Environment:
 *   PRIVATE_KEY         — deployer key
 *   SUBNET_ROOT         — chain ID for the subnet root (default: 11155111)
 *   GATEWAY_OWNER       — owner address (default: deployer)
 */
contract DeployEthGateway is Script {
    function run() external {
        uint256 pk = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(pk);

        uint64 subnetRoot = uint64(vm.envOr("SUBNET_ROOT", uint256(11155111)));
        address owner     = vm.envOr("GATEWAY_OWNER", deployer);

        address[] memory route = new address[](0);
        SubnetID memory networkName = SubnetID({root: subnetRoot, route: route});

        vm.startBroadcast(pk);

        EthGatewayMessenger gw = new EthGatewayMessenger(owner, networkName);

        vm.stopBroadcast();

        console.log("EthGatewayMessenger deployed at:", address(gw));
        console.log("Owner:                          ", owner);
        console.log("Subnet root:                    ", subnetRoot);
    }
}
