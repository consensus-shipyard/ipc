// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorRewarder} from "../activities/IValidatorRewarder.sol";
import {Consensus} from "../activities/Activity.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// An example validator rewarder implementation that only tracks the cumulative number of
/// blocks committed by each validator.
contract ValidatorRewarderMap is IValidatorRewarder, Ownable {
    SubnetID public subnetId;

    mapping(address => uint64) public blocksCommitted;

    constructor() Ownable(msg.sender) {}

    function setSubnet(SubnetID calldata id) external onlyOwner {
        require(id.route.length > 0, "root not allowed");
        subnetId = id;
    }

    function disburseRewards(SubnetID calldata id, Consensus.ValidatorDetail calldata detail) external {
        require(keccak256(abi.encode(id)) == keccak256(abi.encode(subnetId)), "not my subnet");

        address actor = id.route[id.route.length - 1];
        require(actor == msg.sender, "not from subnet");

        blocksCommitted[detail.validator] += detail.blocksCommitted;
    }
}
