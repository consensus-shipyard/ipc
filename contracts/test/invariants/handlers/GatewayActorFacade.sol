// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {CommonBase} from "forge-std/Base.sol";
import {FvmAddress} from "../../../src/structs/FvmAddress.sol";
import {SubnetID} from "../../../src/structs/Subnet.sol";
import {GatewayDiamond} from "../../../src/GatewayDiamond.sol";
import {GatewayManagerFacet} from "../../../src/gateway/GatewayManagerFacet.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

uint256 constant ETH_SUPPLY = 129_590_000 ether;

contract GatewayActorFacade is CommonBase {
    GatewayManagerFacet managerFacet;

    uint256 private constant DEFAULT_MIN_VALIDATOR_STAKE = 10 ether;

    constructor(GatewayDiamond _gw) {
        managerFacet = GatewayManagerFacet(address(_gw));

        deal(address(this), ETH_SUPPLY);
    }

    function register(uint256 amount) external payable {
        managerFacet.register(amount);
    }

    function addStake() external payable {
        managerFacet.addStake();
    }

    function releaseStake(uint256 amount) external {
        managerFacet.releaseStake(amount);
    }

    function kill() external {
        managerFacet.kill();
    }

    function fund(SubnetID calldata subnetId, FvmAddress calldata to) external payable {
        managerFacet.fund(subnetId, to);
    }

    function fundWithToken(SubnetID calldata subnetId, FvmAddress calldata to, uint256 amount) external {
        managerFacet.fundWithToken(subnetId, to, amount);
    }

    function release(FvmAddress calldata to) external payable {
        managerFacet.release(to);
    }

    function forward(address callee, bytes calldata _data) public {
        callee.delegatecall(_data);
    }

    receive() external payable {}
}
