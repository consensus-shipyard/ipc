// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/Gateway.sol";

contract GatewayDeploymentTest is Test {
    int64 constant DEFAULT_CHECKPOINT_PERIOD = 10;
    uint64 constant MIN_COLLATERAL_AMOUNT = 1 ether;
    uint64 constant MAX_NONCE = type(uint64).max;

    Gateway gw;

    function testDeployment(int64 checkpointPeriod) public {
        vm.assume(checkpointPeriod >= DEFAULT_CHECKPOINT_PERIOD);

        gw = new Gateway("/root", checkpointPeriod);
    
        (string memory parent, address actor) = gw.networkName();

        require(
            keccak256(abi.encode(parent)) == keccak256(abi.encode("/root"))
        );
        require(actor == address(0));
        require(gw.minStake() == MIN_COLLATERAL_AMOUNT);
        require(gw.checkPeriod() == checkpointPeriod);
        require(gw.appliedBottomUpNonce() == MAX_NONCE);
    }
}
