// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorRewarder} from "../activities/IValidatorRewarder.sol";
import {ValidatorSummary} from "../activities/Activity.sol";
import {SubnetID} from "../structs/Subnet.sol";

/// An example validator rewarder implementation that tracks the accumulated 
/// reward for each valdiator only.
contract ValidatorRewarderMap is IValidatorRewarder {
    SubnetID public subnetId;
    address public owner;

    mapping(address => uint64) public blocksCommitted;

    constructor() {
        owner = msg.sender;
    }

    function setSubnet(SubnetID calldata id) external {
        require(msg.sender == owner, "not owner");
        require(id.route.length > 0, "root not allowed");

        subnetId = id;
    }

    function disburseRewards(SubnetID calldata id, ValidatorSummary calldata summary) external {
        require(keccak256(abi.encode(id)) == keccak256(abi.encode(subnetId)), "not my subnet");
        
        address actor = id.route[id.route.length - 1];
        require(actor == msg.sender, "not from subnet");

        blocksCommitted[summary.validator] += summary.blocksCommitted;
    }
}