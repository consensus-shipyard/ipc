// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorRewarder} from "../activities/IValidatorRewarder.sol";
import {ValidatorSummary} from "../activities/Activity.sol";
import {SubnetID} from "../structs/Subnet.sol";

contract ValidatorRewarderMap is IValidatorRewarder {

    mapping(address => uint64) public blocksCommitted;

    function disburseRewards(SubnetID calldata /*id*/, ValidatorSummary calldata summary) external {
        blocksCommitted[summary.validator] += summary.blocksCommitted;
    }
}