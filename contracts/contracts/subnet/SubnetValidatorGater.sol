// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorGater} from "../interfaces/IValidatorGater.sol";
import {InvalidSubnet, NotAuthorized, PowerChangeRequestNotApproved} from "../errors/IPCErrors.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// This is a simple implementation of `IValidatorGater`. It makes sure the exact power change 
/// request is approved. This is a very strict requirement.
contract SubnetValidatorGater is IValidatorGater, Ownable {
    using SubnetIDHelper for SubnetID;

    address public caller;
    SubnetID public subnet;

    mapping(bytes32 => bool) public allowed;

    constructor(address _caller, SubnetID memory _subnet) Ownable(msg.sender) {
        caller = _caller;
        subnet = _subnet;
    }

    /// Only owner can approve the validator join request
    function approve(address validator, uint256 prevPower, uint256 newPower) external onlyOwner {
        allowed[genKey(validator, prevPower, newPower)] = true;
    }

    function interceptPowerDelta(SubnetID memory id, address validator, uint256 prevPower, uint256 newPower) external override {
        if (msg.sender != caller) {
            revert NotAuthorized(msg.sender);
        }

        if (id.equals(subnet)) {
            revert InvalidSubnet();
        }

        bytes32 key = genKey(validator, prevPower, newPower);
        if (!allowed[key]) {
            revert PowerChangeRequestNotApproved();
        }

        // remove the approved request
        delete allowed[key];
    }

    function genKey(address validator, uint256 prevPower, uint256 newPower) internal pure returns(bytes32) {
        return keccak256(abi.encode(validator, prevPower, newPower));
    }
}