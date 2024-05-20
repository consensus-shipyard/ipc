// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {SubnetID} from "../structs/Subnet.sol";

/// @title Subnet interface
interface ISubnet {
    /// @notice Checks if the subnet is now bootstrapped
    function bootstrapped() external view returns(bool);

    /// @notice Get the id of the subnet
    function id() external view returns(SubnetID memory);
}
