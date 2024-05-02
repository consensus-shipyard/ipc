// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

/// @title Subnet interface
interface ISubnet {
    /// @notice Checks if the subnet is now bootstrapped
    function bootstrapped() external view returns(bool);
}
