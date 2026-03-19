// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Script.sol";

import {BridgeLock} from "../contracts/bridge/BridgeLock.sol";
import {SubnetID} from "../contracts/structs/Subnet.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/**
 * @title DeployBridgeLock
 * @notice Deploys BridgeLock (implementation + UUPS proxy) to Filecoin Calibration.
 *
 * Required environment variables:
 *   GATEWAY_ADDR        — IPC gateway address on Filecoin Calibration
 *   ADMIN_ADDR          — Address to receive DEFAULT_ADMIN_ROLE and PAUSER_ROLE
 *   DEST_SUBNET_ROOT    — Root chainId of the destination subnet (e.g. 11155111 for Ethereum Sepolia)
 *   DEST_RECEIVER       — BridgeMint contract address on the destination chain
 *   IPC_FEE             — Native value (wei) forwarded with each cross-message (e.g. 10000000000000000 = 0.01 ether)
 *
 * Usage:
 *   forge script contracts/script/DeployBridgeLock.s.sol \
 *     --rpc-url $FILECOIN_CALIBRATION_RPC \
 *     --broadcast \
 *     --verify \
 *     -vvvv
 */
contract DeployBridgeLock is Script {
    function run() external {
        address gatewayAddr  = vm.envAddress("GATEWAY_ADDR");
        address adminAddr    = vm.envAddress("ADMIN_ADDR");
        uint64 destRoot      = uint64(vm.envUint("DEST_SUBNET_ROOT"));
        address destReceiver = vm.envAddress("DEST_RECEIVER");
        uint256 ipcFee       = vm.envUint("IPC_FEE");

        // Build destination SubnetID (Ethereum Sepolia: no route, just root chainId)
        address[] memory route = new address[](0);
        SubnetID memory destSubnet = SubnetID({root: destRoot, route: route});

        vm.startBroadcast();

        // 1. Deploy implementation
        BridgeLock impl = new BridgeLock(gatewayAddr);
        console2.log("BridgeLock implementation deployed at:", address(impl));

        // 2. Encode initializer call
        bytes memory initData = abi.encodeWithSelector(
            BridgeLock.initialize.selector,
            adminAddr,
            destSubnet,
            destReceiver,
            ipcFee
        );

        // 3. Deploy UUPS proxy
        ERC1967Proxy proxy = new ERC1967Proxy(address(impl), initData);
        console2.log("BridgeLock proxy deployed at:         ", address(proxy));

        vm.stopBroadcast();

        // Summary
        console2.log("=== BridgeLock Deployment Summary ===");
        console2.log("  Implementation: ", address(impl));
        console2.log("  Proxy (use this): ", address(proxy));
        console2.log("  Gateway:         ", gatewayAddr);
        console2.log("  Admin:           ", adminAddr);
        console2.log("  Dest root:       ", destRoot);
        console2.log("  Dest receiver:   ", destReceiver);
        console2.log("  IPC fee (wei):   ", ipcFee);
    }
}
