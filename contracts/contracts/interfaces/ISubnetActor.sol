// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {Asset} from "../structs/Subnet.sol";

/// @title Subnet actor interface
interface ISubnetActor {
    function supplySource() external view returns (Asset memory);

    /// @notice Returns the total amount of confirmed collateral across all validators.
    function getTotalCurrentPower() external view returns (uint256);

    /// @notice Checks if the validator address is in an active state.
    /// @param validator The address of the checked validator
    function getCurrentPower(address validator) external view returns (uint256);
}
