// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "forge-std/Script.sol";

import {BridgeMint} from "../contracts/bridge/BridgeMint.sol";
import {WrappedToken} from "../contracts/bridge/WrappedToken.sol";
import {SubnetID} from "../contracts/structs/Subnet.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/**
 * @title DeployBridgeMint
 * @notice Deploys BridgeMint (implementation + UUPS proxy) and WrappedToken implementation
 *         to Ethereum Sepolia.
 *
 * Required environment variables:
 *   GATEWAY_ADDR           — IPC gateway address on Ethereum Sepolia
 *   ADMIN_ADDR             — Address to receive DEFAULT_ADMIN_ROLE and PAUSER_ROLE
 *   BRIDGE_LOCK_SUBNET_ROOT — Root chainId of the source subnet (e.g. 314159 for Filecoin Calibration)
 *   BRIDGE_LOCK_ADDR        — BridgeLock contract address on Filecoin Calibration
 *
 * Usage:
 *   forge script contracts/script/DeployBridgeMint.s.sol \
 *     --rpc-url $ETHEREUM_SEPOLIA_RPC \
 *     --broadcast \
 *     --verify \
 *     -vvvv
 */
contract DeployBridgeMint is Script {
    function run() external {
        address gatewayAddr     = vm.envAddress("GATEWAY_ADDR");
        address adminAddr       = vm.envAddress("ADMIN_ADDR");
        uint64 srcRoot          = uint64(vm.envUint("BRIDGE_LOCK_SUBNET_ROOT"));
        address bridgeLockAddr  = vm.envAddress("BRIDGE_LOCK_ADDR");

        address[] memory route = new address[](0);
        SubnetID memory srcSubnet = SubnetID({root: srcRoot, route: route});

        vm.startBroadcast();

        // 1. Deploy WrappedToken implementation (shared across all bridged assets)
        WrappedToken wrappedImpl = new WrappedToken();
        console2.log("WrappedToken implementation:   ", address(wrappedImpl));

        // 2. Deploy BridgeMint implementation
        BridgeMint impl = new BridgeMint(gatewayAddr);
        console2.log("BridgeMint implementation:     ", address(impl));

        // 3. Deploy BridgeMint UUPS proxy
        bytes memory initData = abi.encodeWithSelector(
            BridgeMint.initialize.selector,
            adminAddr,
            srcSubnet,
            bridgeLockAddr
        );
        ERC1967Proxy proxy = new ERC1967Proxy(address(impl), initData);
        console2.log("BridgeMint proxy (use this):   ", address(proxy));

        vm.stopBroadcast();

        // Summary
        console2.log("=== BridgeMint Deployment Summary ===");
        console2.log("  WrappedToken impl: ", address(wrappedImpl));
        console2.log("  BridgeMint impl:   ", address(impl));
        console2.log("  BridgeMint proxy:  ", address(proxy));
        console2.log("  Gateway:           ", gatewayAddr);
        console2.log("  Admin:             ", adminAddr);
        console2.log("  Src subnet root:   ", srcRoot);
        console2.log("  BridgeLock addr:   ", bridgeLockAddr);
        console2.log("");
        console2.log("Next steps:");
        console2.log("  1. Set BRIDGE_MINT_ADDR=<proxy> in your env");
        console2.log("  2. For each bridged asset, call deployAndRegisterAsset() or registerAsset()");
        console2.log("     on the BridgeMint proxy, passing the WrappedToken impl address");
        console2.log("  3. Grant MINTER_ROLE on each WrappedToken to the BridgeMint proxy");
    }
}
