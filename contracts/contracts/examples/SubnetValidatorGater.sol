// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorGater} from "../interfaces/IValidatorGater.sol";
import {InvalidSubnet, NotAuthorized, ValidatorPowerChangeDenied} from "../errors/IPCErrors.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// The power range that an approved validator can have.
struct PowerRange {
    uint256 min;
    uint256 max;
}

/// This is a simple implementation of `IValidatorGater`. It makes sure the exact power change
/// request is approved. This is a very strict requirement.
/// See sample cli usage in tasks/validator-gater.ts
contract SubnetValidatorGater is IValidatorGater, Ownable {
    using SubnetIDHelper for SubnetID;

    SubnetID public subnet;
    mapping(address => PowerRange) public allowed;

    constructor() Ownable(msg.sender) {}

    function setSubnet(SubnetID calldata id) external onlyOwner {
        subnet = id;
    }

    function isAllow(address validator, uint256 power) public view returns (bool) {
        PowerRange memory range = allowed[validator];
        return range.min <= power && power <= range.max;
    }

    /// Only owner can approve the validator join request
    function approve(address validator, uint256 minPower, uint256 maxPower) external onlyOwner {
        allowed[validator] = PowerRange({min: minPower, max: maxPower});
    }

    /// Revoke approved power range
    function revoke(address validator) external onlyOwner {
        delete allowed[validator];
    }

    function interceptPowerDelta(
        SubnetID memory id,
        address validator,
        uint256 /*prevPower*/,
        uint256 newPower
    ) external view override {
        SubnetID memory targetSubnet = subnet;

        if (!id.equals(targetSubnet)) {
            revert InvalidSubnet();
        }

        if (msg.sender != targetSubnet.getAddress()) {
            revert NotAuthorized(msg.sender);
        }

        if (!isAllow(validator, newPower)) {
            revert ValidatorPowerChangeDenied();
        }
    }
}
