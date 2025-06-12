// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";

/// @title Validator Gater interface
/// This interface introduces the ability to intercept validator power updates before it's executed. Power updates could
/// come from staking, unstaking, and explicit validator membership adjustments (federated membership). With this interface,
/// it introduces an extra layer of checking to directly allow or deny the action, according to a user-defined policy.
interface IValidatorGater {
    /// This intercepts the power update call.
    /// @notice This method should revert if the power update is not allowed.
    function interceptPowerDelta(SubnetID memory id, address validator, uint256 prevPower, uint256 newPower) external;
}
